use std::path::PathBuf;

pub struct LauncherPaths {
    pub root: PathBuf,     
    pub instances: PathBuf,
}

impl LauncherPaths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            instances: root.join("instances"),
            root,
        }
    }

    pub fn official_mc() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").expect("Could not find APPDATA");
            PathBuf::from(appdata).join(".minecraft")
        }
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").expect("Could not find HOME");
            PathBuf::from(home).join("Library/Application Support/minecraft")
        }
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").expect("Could not find HOME");
            PathBuf::from(home).join(".minecraft")
        }
    }
}