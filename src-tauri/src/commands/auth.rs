use tauri::{AppHandle, WebviewWindowBuilder, WebviewUrl, Emitter, Manager};
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ACCEPT};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

const CLIENT_ID: &str = "00000000402b5328"; //
const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf"; 
const SCOPES: &str = "service::user.auth.xboxlive.com::MBI_SSL";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftAccount {
    pub uuid: String,
    pub name: String,
    pub mc_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
    refresh_token: String,
}

#[tauri::command]
pub async fn start_login(app: AppHandle) -> Result<(), String> {
    let app_handle = app.clone();
    
    let auth_url = format!(
        "https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri={}&scope={}&prompt=select_account",
        CLIENT_ID, REDIRECT_URI, SCOPES
    );

    WebviewWindowBuilder::new(&app, "auth", WebviewUrl::External(auth_url.parse().unwrap()))
        .title("Microsoft Login")
        .inner_size(500.0, 650.0)
        .on_navigation(move |url| {
            let url_str = url.as_str();
            
            if url_str.starts_with(REDIRECT_URI) {
                if let Ok(parsed_url) = Url::parse(url_str) {
                    if let Some((_, code)) = parsed_url.query_pairs().find(|(key, _)| key == "code") {
                        let code_clean = code.to_string();
                        let handle = app_handle.clone();

                        if let Some(w) = handle.get_webview_window("auth") { let _ = w.close(); }
                        let _ = handle.emit("install-status", "Authenticating...");
                        
                        tauri::async_runtime::spawn(async move {
                            match perform_handshake(&code_clean).await {
                                Ok(account) => { let _ = handle.emit("login-success", account); }
                                Err(e) => { 
                                    println!("LOGIN ERROR: {}", e);
                                    let _ = handle.emit("login-error", e); 
                                }
                            }
                        });
                    }
                }
                return false; 
            }
            true
        })
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn refresh_login(refresh_token: String) -> Result<MinecraftAccount, String> {
    let client = Client::new();
    
    // Using explicit params map like the reference
    let params = [
        ("client_id", CLIENT_ID),
        ("refresh_token", &refresh_token),
        ("grant_type", "refresh_token"),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPES)
    ];

    let msa_res = client.post("https://login.live.com/oauth20_token.srf")
        .form(&params)
        .send().await.map_err(|e| e.to_string())?;

    if !msa_res.status().is_success() {
        return Err(format!("Refresh Token Failed: {}", msa_res.status()));
    }

    let tokens: OAuthResponse = msa_res.json().await.map_err(|e| e.to_string())?;

    let mut account = perform_minecraft_login(&client, &tokens.access_token).await?;
    account.refresh_token = tokens.refresh_token;
    Ok(account)
}

async fn perform_handshake(code: &str) -> Result<MinecraftAccount, String> {
    let client = Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPES)
    ];

    let msa_res = client.post("https://login.live.com/oauth20_token.srf")
        .form(&params)
        .send().await.map_err(|e| e.to_string())?;

    if !msa_res.status().is_success() {
        return Err(format!("MSA Token Failed: {}", msa_res.text().await.unwrap_or_default()));
    }

    let tokens: OAuthResponse = msa_res.json().await.map_err(|e| e.to_string())?;
    
    let mut account = perform_minecraft_login(&client, &tokens.access_token).await?;
    account.refresh_token = tokens.refresh_token;
    Ok(account)
}

// shit code 
async fn perform_minecraft_login(client: &Client, msa_token: &str) -> Result<MinecraftAccount, String> {
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let xbl_payload = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": msa_token 
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let xbl_res = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .headers(headers.clone())
        .json(&xbl_payload)
        .send().await.map_err(|e| e.to_string())?;

    if !xbl_res.status().is_success() {
        return Err(format!("XBL Auth Failed: {}", xbl_res.text().await.unwrap_or_default()));
    }

    let xbl_data: serde_json::Value = xbl_res.json().await.map_err(|e| e.to_string())?;
    let xbl_token = xbl_data["Token"].as_str().ok_or("No XBL Token")?;
    let uhs = xbl_data["DisplayClaims"]["xui"][0]["uhs"].as_str().ok_or("No UHS")?;

    let xsts_payload = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let xsts_res = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .headers(headers.clone())
        .json(&xsts_payload)
        .send().await.map_err(|e| e.to_string())?;

    if !xsts_res.status().is_success() {
        return Err(format!("XSTS Auth Failed: {}", xsts_res.text().await.unwrap_or_default()));
    }

    let xsts_data: serde_json::Value = xsts_res.json().await.map_err(|e| e.to_string())?;
    let xsts_token = xsts_data["Token"].as_str().ok_or("No XSTS Token")?;

    let mc_payload = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });

    let mc_res = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .headers(headers.clone())
        .json(&mc_payload)
        .send().await.map_err(|e| e.to_string())?;

    if !mc_res.status().is_success() {
        return Err(format!("MC Login Failed: {}", mc_res.text().await.unwrap_or_default()));
    }

    let mc_data: serde_json::Value = mc_res.json().await.map_err(|e| e.to_string())?;
    let mc_token = mc_data["access_token"].as_str().ok_or("No MC Token")?;
    let expires_in = mc_data["expires_in"].as_u64().unwrap_or(86400);

    let profile = client.get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token)
        .send().await.map_err(|e| e.to_string())?
        .json::<serde_json::Value>().await.map_err(|e| e.to_string())?;

    let uuid = profile["id"].as_str().unwrap_or("error").to_string();
    let name = profile["name"].as_str().unwrap_or("Player").to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    Ok(MinecraftAccount {
        uuid,
        name,
        mc_token: mc_token.to_string(),
        refresh_token: String::new(), // filled by caller
        expires_at: now + expires_in,
    })
}