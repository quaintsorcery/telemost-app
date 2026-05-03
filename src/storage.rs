use crate::models::Conference;
use directories::ProjectDirs;
use keyring::Entry;
use std::fs;

pub struct Storage;

impl Storage {
    const APP_NAME: &'static str = "telemost-app";
    const SERVICE: &'static str = "telemost-app-auth";
    const ACCOUNT: &'static str = "current_user_token";

    fn get_entry() -> Result<Entry, String> {
        Entry::new(Self::SERVICE, Self::ACCOUNT).map_err(|e| e.to_string())
    }

    pub fn save_token(token: &str) {
        if let Ok(entry) = Self::get_entry() {
            let _ = entry.set_password(token);
        }
    }

    pub fn get_token() -> Option<String> {
        Self::get_entry().ok()?.get_password().ok()
    }

    pub fn save_login(login: &str) {
        if let Some(dirs) = ProjectDirs::from("com", "quaintsorcery", Self::APP_NAME) {
            let path = dirs.config_dir();
            let _ = fs::create_dir_all(path);
            let _ = fs::write(path.join("user.txt"), login);
        }
    }

    pub fn get_login() -> Option<String> {
        ProjectDirs::from("com", "quaintsorcery", Self::APP_NAME)
            .and_then(|dirs| fs::read_to_string(dirs.config_dir().join("user.txt")).ok())
    }

    pub fn save_conferences(list: &[Conference]) {
        if let Some(dirs) = ProjectDirs::from("com", "quaintsorcery", Self::APP_NAME) {
            let _ = fs::create_dir_all(dirs.config_dir());
            if let Ok(json) = serde_json::to_string(list) {
                let _ = fs::write(dirs.config_dir().join("conferences.json"), json);
            }
        }
    }

    pub fn get_conferences() -> Vec<Conference> {
        ProjectDirs::from("com", "quaintsorcery", Self::APP_NAME)
            .and_then(|dirs| fs::read_to_string(dirs.config_dir().join("conferences.json")).ok())
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn clear_all() {
        if let Ok(entry) = Self::get_entry() {
            let _ = entry.delete_credential();
        }
        if let Some(dirs) = ProjectDirs::from("com", "quaintsorcery", Self::APP_NAME) {
            let _ = fs::remove_file(dirs.config_dir().join("user.txt"));
        }
    }
}
