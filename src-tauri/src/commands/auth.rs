use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

use crate::error::{LauncherError, Result};

const CLIENT_ID: &str = "00000000402b5328";
const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
const SCOPES: &str = "service::user.auth.xboxlive.com::MBI_SSL";

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftAccount {
    pub uuid: String,
    pub name: String,
    pub mc_token: String,
    pub refresh_token: String,
    /// Unix timestamp (seconds) when `mc_token` expires
    pub expires_at: u64,
}

impl MinecraftAccount {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Refresh if within 5 minutes of expiry
        now + 300 >= self.expires_at
    }
}

// ── Internal types ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    refresh_token: String,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Opens a Microsoft login window and emits `login-success` / `login-error` events.
#[tauri::command]
pub async fn start_login(app: AppHandle) -> Result<()> {
    let auth_url = format!(
        "https://login.live.com/oauth20_authorize.srf\
         ?client_id={CLIENT_ID}\
         &response_type=code\
         &redirect_uri={REDIRECT_URI}\
         &scope={SCOPES}\
         &prompt=select_account"
    );

    let app_for_nav = app.clone();

    WebviewWindowBuilder::new(&app, "auth", WebviewUrl::External(auth_url.parse().unwrap()))
        .title("Sign in with Microsoft")
        .inner_size(500.0, 650.0)
        .resizable(false)
        .on_navigation(move |url| {
            let url_str = url.as_str();

            if !url_str.starts_with(REDIRECT_URI) {
                return true; // allow navigation
            }

            // Extract the auth code from the redirect
            if let Some(code) = Url::parse(url_str)
                .ok()
                .and_then(|u| {
                    u.query_pairs()
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.into_owned())
                })
            {
                let handle = app_for_nav.clone();

                // Close the auth window immediately
                if let Some(w) = handle.get_webview_window("auth") {
                    let _ = w.close();
                }
                let _ = handle.emit("auth-status", "Authenticating…");

                tauri::async_runtime::spawn(async move {
                    match perform_full_auth(&code).await {
                        Ok(account) => {
                            let _ = handle.emit("login-success", account);
                        }
                        Err(e) => {
                            log::error!("Login failed: {e}");
                            let _ = handle.emit("login-error", e.to_string());
                        }
                    }
                });
            }

            false // block the redirect page itself from loading
        })
        .build()
        .map_err(|e| LauncherError::Auth(e.to_string()))?;

    Ok(())
}

/// Refresh an existing session using the stored refresh token.
#[tauri::command]
pub async fn refresh_login(refresh_token: String) -> Result<MinecraftAccount> {
    let client = build_client()?;

    let params = [
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token.as_str()),
        ("grant_type", "refresh_token"),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPES),
    ];

    let res = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "Token refresh failed: {}",
            res.status()
        )));
    }

    let tokens: OAuthResponse = res.json().await?;
    let mut account = minecraft_login(&client, &tokens.access_token).await?;
    account.refresh_token = tokens.refresh_token;
    Ok(account)
}

// ── Internal auth flow ────────────────────────────────────────────────────────

async fn perform_full_auth(code: &str) -> Result<MinecraftAccount> {
    let client = build_client()?;

    let params = [
        ("client_id", CLIENT_ID),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPES),
    ];

    let res = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(LauncherError::Auth(format!("MSA token exchange failed: {body}")));
    }

    let tokens: OAuthResponse = res.json().await?;
    let mut account = minecraft_login(&client, &tokens.access_token).await?;
    account.refresh_token = tokens.refresh_token;
    Ok(account)
}

async fn minecraft_login(client: &Client, msa_token: &str) -> Result<MinecraftAccount> {
    let json_headers = json_headers();

    let xbl_res = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .headers(json_headers.clone())
        .json(&serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={msa_token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?;

    if !xbl_res.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "XBL auth failed: {}",
            xbl_res.text().await.unwrap_or_default()
        )));
    }

    let xbl: serde_json::Value = xbl_res.json().await?;
    let xbl_token = xbl["Token"].as_str().ok_or_else(|| LauncherError::Auth("No XBL token".into()))?;
    let uhs = xbl["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or_else(|| LauncherError::Auth("No UHS claim".into()))?;

    let xsts_res = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .headers(json_headers.clone())
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;

    if !xsts_res.status().is_success() {
        let status = xsts_res.status();
        let body: serde_json::Value = xsts_res.json().await.unwrap_or_default();
        let xerr = body["XErr"].as_u64().unwrap_or(0);
        let msg = match xerr {
            2148916233 => "This Microsoft account has no Xbox profile. Please create one at xbox.com.".into(),
            2148916238 => "Child accounts must be added to a family group to play Minecraft.".into(),
            _ => format!("XSTS auth failed (HTTP {status}, XErr {xerr})"),
        };
        return Err(LauncherError::Auth(msg));
    }

    let xsts: serde_json::Value = xsts_res.json().await?;
    let xsts_token = xsts["Token"].as_str().ok_or_else(|| LauncherError::Auth("No XSTS token".into()))?;

    let mc_res = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .headers(json_headers)
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={uhs};{xsts_token}")
        }))
        .send()
        .await?;

    if !mc_res.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "MC login failed: {}",
            mc_res.text().await.unwrap_or_default()
        )));
    }

    let mc: serde_json::Value = mc_res.json().await?;
    let mc_token = mc["access_token"].as_str().ok_or_else(|| LauncherError::Auth("No MC access token".into()))?;
    let expires_in = mc["expires_in"].as_u64().unwrap_or(86400);

    let profile_res = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token)
        .send()
        .await?;

    if !profile_res.status().is_success() {
        return Err(LauncherError::Auth(
            "Could not fetch Minecraft profile. Does this account own Minecraft?".into(),
        ));
    }

    let profile: serde_json::Value = profile_res.json().await?;
    let uuid = profile["id"].as_str().unwrap_or("").to_string();
    let name = profile["name"].as_str().unwrap_or("Player").to_string();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(MinecraftAccount {
        uuid,
        name,
        mc_token: mc_token.to_string(),
        refresh_token: String::new(), // filled by caller
        expires_at: now + expires_in,
    })
}

fn build_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| LauncherError::Network(e))
}

fn json_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    h.insert(ACCEPT, HeaderValue::from_static("application/json"));
    h
}