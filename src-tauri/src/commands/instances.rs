use crate::models::fs::LauncherPaths;
use crate::models::instance::Instance;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn list_instances(paths: State<Mutex<LauncherPaths>>) -> Result<Vec<String>, String> {
    let launcher_paths = paths.lock().map_err(|e| format!("Failed to lock state: {}", e))?;

    Ok(std::fs::read_dir(&launcher_paths.instances)
        .map_err(|e| format!("Failed to read instances dir: {}", e))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().is_dir() {
                    e.file_name().into_string().ok()
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
pub fn create_instance(instance: Instance, paths: State<Mutex<LauncherPaths>>) -> Result<(), String> {
    let launcher_paths = paths.lock().map_err(|e| format!("Failed to lock state: {}", e))?;

    let instance_dir = launcher_paths.instances.join(&instance.name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("Failed to create instance dir: {}", e))?;

    let metadata = serde_json::to_string_pretty(&instance)
        .map_err(|e| format!("Failed to serialize instance: {}", e))?;

    std::fs::write(instance_dir.join("instance.json"), metadata)
        .map_err(|e| format!("Failed to write instance.json: {}", e))?;

    Ok(())
}