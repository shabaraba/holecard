use anyhow::Result;
use std::path::PathBuf;

use crate::cli::input;
use crate::config::Config;
use crate::domain::Vault;
use crate::infrastructure::{CryptoServiceImpl, KeyringManager, SessionData, SessionManager, VaultStorage};

pub struct VaultContext {
    pub vault: Vault,
    pub storage: VaultStorage<CryptoServiceImpl>,
    pub session_data: SessionData,
    pub config: Config,
    pub config_dir: PathBuf,
}

impl VaultContext {
    pub fn load(keyring: &KeyringManager, config_dir: &PathBuf) -> Result<Self> {
        let secret_key = keyring.load_secret_key()?;
        let config = Config::load(config_dir)?;
        let crypto = CryptoServiceImpl::new();
        let storage = VaultStorage::new(crypto);
        let session = SessionManager::new(config_dir, config.session_timeout_minutes);

        let (vault, session_data) = if let Some(cached) = session.load_session()? {
            let vault = storage
                .load_with_cached_key(&config.vault_path, &cached.derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            (vault, cached)
        } else {
            let master_password = input::prompt_master_password()?;
            let (derived_key, salt) = storage
                .derive_key_from_vault(&config.vault_path, &master_password, &secret_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let vault = storage
                .load_with_cached_key(&config.vault_path, &derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let session_data = SessionData { derived_key, salt };
            session.save_session(&derived_key, &salt)?;
            (vault, session_data)
        };

        Ok(Self {
            vault,
            storage,
            session_data,
            config,
            config_dir: config_dir.clone(),
        })
    }

    pub fn save(&self) -> Result<()> {
        self.storage
            .save_with_cached_key(
                &self.vault,
                &self.config.vault_path,
                &self.session_data.derived_key,
                &self.session_data.salt,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let session = SessionManager::new(&self.config_dir, self.config.session_timeout_minutes);
        session.save_session(&self.session_data.derived_key, &self.session_data.salt)?;

        Ok(())
    }
}
