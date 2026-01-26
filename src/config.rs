use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vault_path: PathBuf,
    pub session_timeout_minutes: u64,
}

impl Config {
    pub fn load(config_dir: &PathBuf) -> Result<Self> {
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let default_config = Self::default_with_dir(config_dir);
            default_config.save(config_dir)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }

    pub fn save(&self, config_dir: &PathBuf) -> Result<()> {
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    fn default_with_dir(config_dir: &PathBuf) -> Self {
        Self {
            vault_path: config_dir.join("vault.enc"),
            session_timeout_minutes: 60,
        }
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let config_dir = home.join(".holecard");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    Ok(config_dir)
}
