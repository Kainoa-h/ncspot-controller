use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub hook_script: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if !config_path.exists() {
            return Config::default();
        }

        match fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to parse config file: {}", e);
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
                Config::default()
            }
        }
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("ncspot-controller")
            .join("config.toml")
    }
}
