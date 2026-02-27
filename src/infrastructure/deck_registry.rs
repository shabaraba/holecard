use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckMetadata {
    pub name: String,
    pub path: PathBuf,
    pub created_at: DateTime<Local>,
    pub last_accessed: DateTime<Local>,
}

impl DeckMetadata {
    pub fn new(name: String, path: PathBuf) -> Self {
        let now = Local::now();
        Self {
            name,
            path,
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Local::now();
    }
}

// Note: Struct and field names retained for backward compatibility with vaults.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultsConfig {
    active_vault: String,
    vaults: Vec<DeckMetadata>,
}

pub struct DeckRegistry {
    config_dir: PathBuf,
}

impl DeckRegistry {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub fn load(config_dir: &Path) -> Result<Self> {
        let registry = Self::new(config_dir.to_path_buf());

        if !registry.registry_path().exists() {
            registry.migrate_legacy_vault()?;
        }

        Ok(registry)
    }

    fn registry_path(&self) -> PathBuf {
        self.config_dir.join("vaults.toml")
    }

    fn load_config(&self) -> Result<VaultsConfig> {
        if !self.registry_path().exists() {
            return Ok(VaultsConfig {
                active_vault: String::new(),
                vaults: Vec::new(),
            });
        }

        let content =
            fs::read_to_string(self.registry_path()).context("Failed to read vaults.toml")?;

        toml::from_str(&content).context("Failed to parse vaults.toml")
    }

    fn save_config(&self, config: &VaultsConfig) -> Result<()> {
        let content =
            toml::to_string_pretty(config).context("Failed to serialize vaults config")?;

        fs::write(self.registry_path(), content).context("Failed to write vaults.toml")?;
        Ok(())
    }

    pub fn create_deck(&self, name: &str, path: PathBuf) -> Result<DeckMetadata> {
        let mut config = self.load_config()?;

        if config.vaults.iter().any(|v| v.name == name) {
            anyhow::bail!("Deck '{}' already exists", name);
        }

        let metadata = DeckMetadata::new(name.to_string(), path);
        config.vaults.push(metadata.clone());

        if config.active_vault.is_empty() {
            config.active_vault = name.to_string();
        }

        self.save_config(&config)?;
        Ok(metadata)
    }

    pub fn delete_deck(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        let initial_len = config.vaults.len();
        config.vaults.retain(|v| v.name != name);

        if config.vaults.len() == initial_len {
            anyhow::bail!("Vault '{}' not found", name);
        }

        if config.active_vault == name {
            config.active_vault = config
                .vaults
                .first()
                .map(|v| v.name.clone())
                .unwrap_or_default();
        }

        self.save_config(&config)?;
        Ok(())
    }

    pub fn set_active(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        if !config.vaults.iter().any(|v| v.name == name) {
            anyhow::bail!("Vault '{}' not found", name);
        }

        config.active_vault = name.to_string();
        self.save_config(&config)?;
        Ok(())
    }

    pub fn get_deck(&self, name: &str) -> Result<DeckMetadata> {
        let config = self.load_config()?;

        config
            .vaults
            .into_iter()
            .find(|v| v.name == name)
            .ok_or_else(|| anyhow::anyhow!("Vault '{}' not found", name))
    }

    pub fn get_active_deck(&self) -> Result<DeckMetadata> {
        let config = self.load_config()?;

        if config.active_vault.is_empty() {
            anyhow::bail!("No active vault set. Use 'hc vault use <name>' to set one.");
        }

        self.get_deck(&config.active_vault)
    }

    pub fn list_decks(&self) -> Result<Vec<DeckMetadata>> {
        let config = self.load_config()?;
        let mut vaults = config.vaults;
        vaults.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        Ok(vaults)
    }

    pub fn touch_deck(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;

        if let Some(vault) = config.vaults.iter_mut().find(|v| v.name == name) {
            vault.touch();
            self.save_config(&config)?;
            Ok(())
        } else {
            anyhow::bail!("Vault '{}' not found", name);
        }
    }

    fn migrate_legacy_vault(&self) -> Result<()> {
        let legacy_vault_path = self.config_dir.join("vault.enc");

        if legacy_vault_path.exists() {
            println!("🔄 Migrating existing vault to 'default' vault...");

            let metadata = DeckMetadata::new("default".to_string(), legacy_vault_path);
            let config = VaultsConfig {
                active_vault: "default".to_string(),
                vaults: vec![metadata],
            };

            self.save_config(&config)?;
            println!("✓ Migration complete. Your vault is now named 'default'.");
        }

        Ok(())
    }
}
