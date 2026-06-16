use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ttl: Option<f64>,
    pub idle: Option<f64>,
    pub stealth_encryption: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ttl: Some(100.0),
            idle: Some(300.0),
            stealth_encryption: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("com", "laticee", "amnesia") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.toml");

            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    } else {
                        eprintln!(
                            "Warning: Failed to parse config file at {:?}. Using defaults.",
                            config_path
                        );
                    }
                }
            }
        }
        Self::default()
    }
}
