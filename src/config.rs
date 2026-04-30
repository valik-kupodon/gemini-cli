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

    #[cfg(test)]
    fn with_key_path(key_path: PathBuf) -> Self {
        Self {
            key_path: Some(key_path),
        }
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

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_path() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("gemini-cli-test-{suffix}"))
            .join("api_key.txt")
    }

    #[test]
    fn saves_and_reads_trimmed_api_key() {
        let key_path = unique_test_path();
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();

        let config = Config::with_key_path(key_path.clone());
        let saved_path = config.save_api_key("  test-key  \n").unwrap();

        assert_eq!(saved_path, key_path);
        assert_eq!(config.get_api_key().as_deref(), Some("test-key"));

        fs::remove_dir_all(key_path.parent().unwrap()).unwrap();
    }

    #[test]
    fn returns_none_when_key_file_is_missing() {
        let key_path = unique_test_path();
        let config = Config::with_key_path(key_path);

        assert_eq!(config.get_api_key(), None);
    }

    #[test]
    fn returns_none_for_empty_key_file() {
        let key_path = unique_test_path();
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "   \n").unwrap();

        let config = Config::with_key_path(key_path.clone());

        assert_eq!(config.get_api_key(), None);

        fs::remove_dir_all(key_path.parent().unwrap()).unwrap();
    }
}
