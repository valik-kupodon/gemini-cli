use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub key_path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        let key_path = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).map(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir); // Ігноруємо помилку, обробимо пізніше при записі
            }
            config_dir.join("api_key.txt")
        });

        Self { key_path }
    }

    pub fn save_api_key(&self, api_key: &str) -> Result<PathBuf, String> {
        if let Some(ref path) = self.key_path {
            fs::write(path, api_key).map_err(|e| format!("Помилка запису файлу: {}", e))?;
            Ok(path.clone())
        } else {
            Err("Невизначений шлях для збереження API ключа".to_string())
        }
    }

    pub fn get_api_key(&self) -> Option<String> {
        if let Some(path) = &self.key_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        return Some(trimmed);
                    }
                }
            }
        }
        None
    }
}
