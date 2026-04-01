use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub default_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_url: "https://start.duckduckgo.com".to_string(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("light")
}

fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load() -> Settings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => {
            let settings = Settings::default();
            save(&settings);
            settings
        }
    }
}

pub fn save(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(settings).unwrap();
    let _ = fs::write(path, json);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_has_url() {
        let s = Settings::default();
        assert!(!s.default_url.is_empty());
    }

    #[test]
    fn round_trip_serialize() {
        let s = Settings {
            default_url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s2.default_url, "https://example.com");
    }
}
