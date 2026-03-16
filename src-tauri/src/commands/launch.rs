use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{Emitter, State};

use crate::models::fs::LauncherPaths;
use crate::models::instance::Instance;
use crate::models::mc::VersionManifest;
use crate::utils;

#[tauri::command]
pub fn launch_instance(
    instance_name: String,
    uuid: String,
    name: String,
    access_token: String,
    paths: State<'_, Mutex<LauncherPaths>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let p = paths.lock().map_err(|e| e.to_string())?;

    let instance_dir = p.instances.join(&instance_name);
    let libraries_root = p.root.join("libraries");
    let assets_root = p.root.join("assets");
    let natives_dir = instance_dir.join("natives");
    let client_jar = instance_dir.join("client.jar");
    let version_file = instance_dir.join("version.json");
    let instance_file = instance_dir.join("instance.json");
    let official_mc = LauncherPaths::official_mc();

    let instance: Instance = {
        let raw = std::fs::read_to_string(&instance_file)
            .map_err(|e| format!("Could not read instance.json for '{}': {}", instance_name, e))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("Malformed instance.json: {}", e))?
    };

    let memory_mb = if instance.memory_mb >= 512 {
        instance.memory_mb
    } else {
        return Err(format!(
            "Instance '{}' has memory set to {} MB, which is below the minimum of 512 MB.",
            instance_name, instance.memory_mb
        ));
    };

    if !version_file.exists() {
        return Err(format!(
            "version.json is missing for '{}'. Please install it first.",
            instance_name
        ));
    }

    let version_text = std::fs::read_to_string(&version_file)
        .map_err(|e| format!("Failed to read version.json: {}", e))?;

    let manifest: VersionManifest = serde_json::from_str(&version_text)
        .map_err(|e| format!("Invalid version.json: {}", e))?;

    if instance.mc_version != manifest.id {
        return Err(format!(
            "Version mismatch: instance.json says '{}' but the installed version.json is '{}'. \
             Re-install the instance to fix this.",
            instance.mc_version, manifest.id
        ));
    }
    utils::extract_natives(&libraries_root, &natives_dir, &manifest)?;

    let mut jar_list: Vec<String> = manifest
        .libraries
        .iter()
        .filter(|lib| utils::is_library_allowed(&lib.rules))
        .filter_map(|lib| lib.downloads.artifact.as_ref())
        .map(|artifact| {
            libraries_root
                .join(&artifact.path)
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    jar_list.push(client_jar.to_string_lossy().into_owned());

    let missing: Vec<&str> = jar_list
        .iter()
        .filter(|p| !PathBuf::from(p).exists())
        .map(String::as_str)
        .collect();

    if !missing.is_empty() {
        return Err(format!(
            "{} librar{} missing. Try reinstalling the instance.\nMissing:\n  {}",
            missing.len(),
            if missing.len() == 1 { "y" } else { "ies" },
            missing.join("\n  ")
        ));
    }

    let classpath = jar_list.join(utils::get_classpath_separator());
    let main_class = &manifest.main_class;

    println!(
        "[Launch] '{}' | version: {} | main_class: {} | memory: {}M",
        instance_name, manifest.id, main_class, memory_mb
    );

    let mut child = Command::new("java")
        .arg(format!("-Xms512M"))
        .arg(format!("-Xmx{}M", memory_mb))
        .arg(format!(
            "-Djava.library.path={}",
            natives_dir.to_string_lossy()
        ))
        .arg("-cp")
        .arg(&classpath)
        .arg(main_class)
        .arg("--version")
        .arg(&manifest.id)
        .arg("--accessToken")
        .arg(&access_token)
        .arg("--uuid")
        .arg(&uuid)
        .arg("--username")
        .arg(&name)
        .arg("--userType")
        .arg("msa")
        .arg("--assetsDir")
        .arg(&assets_root)
        .arg("--assetIndex")
        .arg(&manifest.asset_index.id)
        .arg("--gameDir")
        .arg(&official_mc)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Java was not found. Please install Java and make sure it's on your PATH.".to_string()
            } else {
                format!("Failed to launch: {}", e)
            }
        })?;

    let pid = child.id();
    let _ = app.emit("game-started", serde_json::json!({ "instance": instance_name, "pid": pid }));

    let app_handle = app.clone();
    let name_clone = instance_name.clone();
    let log_path = instance_dir.join("last_launch.log");

    std::thread::spawn(move || {
        let mut log_lines: Vec<String> = Vec::new();

        if let Some(stderr) = child.stderr.take() {
            for line in BufReader::new(stderr).lines().flatten() {
                log_lines.push(line);
            }
        }
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().flatten() {
                log_lines.push(line);
            }
        }
        let _ = std::fs::write(&log_path, log_lines.join("\n"));

        match child.wait() {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                if status.success() {
                    let _ = app_handle.emit(
                        "game-stopped",
                        serde_json::json!({ "instance": name_clone, "exit_code": code }),
                    );
                } else {
                    let tail: Vec<&str> = log_lines
                        .iter()
                        .rev()
                        .take(20)
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();

                    let _ = app_handle.emit(
                        "game-crashed",
                        serde_json::json!({
                            "instance": name_clone,
                            "exit_code": code,
                            "log_tail": tail,
                        }),
                    );
                }
            }
            Err(e) => {
                let _ = app_handle.emit(
                    "game-crashed",
                    serde_json::json!({
                        "instance": name_clone,
                        "exit_code": -1,
                        "log_tail": [format!("Failed to wait on process: {}", e)],
                    }),
                );
            }
        }
    });
    Ok(())
}