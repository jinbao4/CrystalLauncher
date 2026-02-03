use tauri::Manager;
use std::sync::Mutex;

mod models;
mod commands; 
mod core;
mod utils;

use models::fs::LauncherPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let root = app.path().app_data_dir().expect("failed to resolve app data dir");
                let launcher_paths = LauncherPaths::new(root);
                app.manage(Mutex::new(launcher_paths));
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::list_instances,
                commands::create_instance,
                commands::install_instance,
                commands::launch_instance,
                commands::start_login,
                commands::refresh_login
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }