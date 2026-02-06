use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ttl: Option<f64>,
    pub idle: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ttl: None,
            idle: Some(300.0),
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
            } else {
                // Optionally create a default config file if it doesn't exist
                let _ = fs::create_dir_all(config_dir);
                let default_config = Self::default();
                if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
                    let _ = fs::write(config_path, toml_str);
                }
            }
        }
        Self::default()
    }
}
