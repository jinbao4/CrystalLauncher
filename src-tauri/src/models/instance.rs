use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstallState {
    NotInstalled,
    Installing,
    Installed,
    Failed,
}

impl Default for InstallState {
    fn default() -> Self {
        InstallState::NotInstalled
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub mc_version: String,
    pub memory_mb: u32,
    #[serde(default)]
    pub install_state: InstallState,
}

impl Instance {
    pub fn new(mc_version: impl Into<String>) -> Self {
        Self {
            mc_version: mc_version.into(),
            memory_mb: 2048,
            install_state: InstallState::NotInstalled,
        }
    }
}