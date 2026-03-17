use std::sync::Mutex;
use tauri::{Emitter, State};

use crate::models::fs::LauncherPaths;
use crate::models::instance::InstallState;
use crate::models::mc::Manifest;
use crate::utils;
use crate::commands::instances::{get_or_create_instance, persist_instance};

#[tauri::command]
pub fn install_instance(
    version: String,
    paths: State<'_, Mutex<LauncherPaths>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let (instances_dir, root_dir) = {
        let p = paths.lock().map_err(|e| e.to_string())?;
        (p.instances.clone(), p.root.clone())
    };
    let mut instance = get_or_create_instance(&instances_dir, &version)?;

    if instance.install_state == InstallState::Installed {
        let _ = app.emit("install-status", "Already installed.");
        return Ok(());
    }

    instance.install_state = InstallState::Installing;
    persist_instance(&instances_dir, &instance)?;

    let app_handle = app.clone();
    let version_clone = version.clone();

    tauri::async_runtime::spawn(async move {
        let run_install = async || -> Result<(), String> {
            let instance_dir = instances_dir.join(&name_clone);
            let libraries_root = root_dir.join("libraries");
            let assets_root = root_dir.join("assets");

            let _ = app_handle.emit("install-status", "Fetching manifest…");

            let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

            let client = reqwest::Client::new();
            let manifest: Manifest = client.get(manifest_url)
                .send().await
                .map_err(|e| e.to_string())?
                .json()
                .await
                .map_err(|e| e.to_string())?;

            let version_entry = manifest
                .versions
                .iter()
                .find(|v| v.id == version_clone)
                .ok_or_else(|| format!("Version '{}' not found in manifest", version_clone))?;

            let version_json_text = client.get(&version_entry.url)
                .send().await
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?;

            std::fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;
            std::fs::write(instance_dir.join("version.json"), &version_json_text)
                .map_err(|e| e.to_string())?;

            let _ = app_handle.emit("install-status", "Downloading Client...");
            utils::download_client_jar(&instance_dir, &version_json_text).await?;

            let _ = app_handle.emit("install-status", "Downloading Libraries...");
            utils::download_libraries(&libraries_root, &version_json_text).await?;

            let _ = app_handle.emit("install-status", "Downloading Assets...");
            utils::download_assets(&assets_root, &version_json_text).await?;

            Ok(())
        };

        if let Err(e) = run_install().await {
            let _ = app_handle.emit("install-error", e);
        }
    });

    Ok(())
}