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
            ttl: None,
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
            } else {
                // Optionally create a default config file if it doesn't exist
                let _ = fs::create_dir_all(config_dir);
                let config_toml = r#"# amnesia configuration file (v1.1)

# [ttl]
# Time to live in minutes.
# After this time, the application will automatically wipe memory and exit.
# Use 0.0 or comment out to disable.
# ttl = 10.0

# [idle]
# Idle timeout in seconds.
# The application will exit if no input is received for this duration.
# Default is 300.0 (5 minutes).
idle = 300.0

# [stealth_encryption]
# Enable stealth memory encryption (volatile-only).
# Encrypts the RAM buffer with a key derived from system state and ASLR.
# Note: Data is only accessible during the current session.
# Default is false.
stealth_encryption = false
"#;
                let _ = fs::write(config_path, config_toml);
            }
        }
        Self::default()
    }
}
