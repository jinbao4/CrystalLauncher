use std::sync::Mutex;
use std::path::Path;
use tauri::{Emitter, State};

use crate::models::fs::LauncherPaths;
use crate::models::mc::{Manifest, VersionManifest};
use crate::utils;

#[tauri::command]
pub fn install_instance(
    instance_name: String,
    version: String,
    paths: State<'_, Mutex<LauncherPaths>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_handle = app.clone();
    let name_clone = instance_name.clone();
    let version_clone = version.clone();

    let (instances_dir, root_dir, jres_dir) = {
        let p = paths.lock().map_err(|e| e.to_string())?;
        (p.instances.clone(), p.root.clone(), p.jres.clone())
    };

    std::thread::spawn(move || {
        let run_install = |jres_dir: &Path| -> Result<(), String> {
            let instance_dir = instances_dir.join(&name_clone);
            let libraries_root = root_dir.join("libraries");
            let assets_root = root_dir.join("assets");

            let _ = app_handle.emit("install-status", "Fetching Manifest...");

            let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

            let manifest: Manifest = reqwest::blocking::get(manifest_url)
                .map_err(|e| e.to_string())?
                .json()
                .map_err(|e| e.to_string())?;

            let version_entry = manifest
                .versions
                .iter()
                .find(|v| v.id == version_clone)
                .ok_or_else(|| format!("Version {} not found", version_clone))?;

            let version_json_text = reqwest::blocking::get(&version_entry.url)
                .map_err(|e| e.to_string())?
                .text()
                .map_err(|e| e.to_string())?;

            std::fs::create_dir_all(&instance_dir).map_err(|e| e.to_string())?;
            std::fs::write(instance_dir.join("version.json"), &version_json_text)
                .map_err(|e| e.to_string())?;

            let _ = app_handle.emit("install-status", "Downloading Client...");
            utils::download_client_jar(&instance_dir, &version_json_text)?;

            let _ = app_handle.emit("install-status", "Downloading Libraries...");
            utils::download_libraries(&libraries_root, &version_json_text)?;

            let _ = app_handle.emit("install-status", "Downloading Assets...");
            utils::download_assets(&assets_root, &version_json_text)?;

            // Download JRE if specified
            let version_manifest: VersionManifest = serde_json::from_str(&version_json_text)
                .map_err(|e| e.to_string())?;
            if let Some(java_version) = version_manifest.java_version {
                let _ = app_handle.emit("install-status", &format!("Downloading JRE {}...", java_version.major_version));
                utils::download_jre(jres_dir, java_version.major_version)?;
            }

            let _ = app_handle.emit("install-status", "Installation Complete!");
            Ok(())
        };

        if let Err(e) = run_install(&jres_dir) {
            let _ = app_handle.emit("install-error", e);
        }
    });

    Ok(())
}