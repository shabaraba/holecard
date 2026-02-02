use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cli::input;
use crate::config::Config;
use crate::domain::Vault;
use crate::infrastructure::{
    CryptoServiceImpl, KeyringManager, SessionData, SessionManager, VaultStorage,
};

pub struct VaultContext {
    pub vault: Vault,
    pub storage: VaultStorage<CryptoServiceImpl>,
    pub session_data: SessionData,
    pub config: Config,
    pub config_dir: PathBuf,
    vault_path: PathBuf,
    vault_name: String,
}

fn resolve_master_password(
    config: &Config,
    keyring: &KeyringManager,
    vault_name: &str,
) -> Result<String> {
    if !config.enable_biometric {
        return input::prompt_master_password();
    }

    let biometric = crate::infrastructure::get_biometric_auth();
    if !biometric.is_available() {
        return input::prompt_master_password();
    }

    println!("🔐 Authenticating...");
    match biometric.authenticate("Unlock your vault") {
        Ok(true) => {
            println!("✅ Authentication successful");
            match keyring.load_master_password(vault_name)? {
                Some(pwd) => {
                    println!("🔓 Unlocking vault...");
                    Ok(pwd)
                }
                None => {
                    println!("⚠️  No cached password found. Please enter your master password.");
                    let pwd = input::prompt_master_password()?;
                    keyring.save_master_password(vault_name, &pwd)?;
                    Ok(pwd)
                }
            }
        }
        Ok(false) => {
            println!("⚠️  Authentication failed. Falling back to password.");
            input::prompt_master_password()
        }
        Err(e) => {
            eprintln!("⚠️  Authentication error: {}. Falling back to password.", e);
            input::prompt_master_password()
        }
    }
}

impl VaultContext {
    pub fn load(
        vault_path: &Path,
        vault_name: &str,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<Self> {
        let secret_key = keyring.load_secret_key()?;
        let config = Config::load(config_dir)?;
        let crypto = CryptoServiceImpl::new();
        let storage = VaultStorage::new(crypto);
        let session = SessionManager::new(config_dir, vault_name, config.session_timeout_minutes);

        let (vault, session_data) = if let Some(cached) = session.load_session()? {
            let vault = storage
                .load_with_cached_key(vault_path, &cached.derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            (vault, cached)
        } else {
            let master_password = resolve_master_password(&config, keyring, vault_name)?;

            let (derived_key, salt) = storage
                .derive_key_from_vault(vault_path, &master_password, &secret_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let vault = storage
                .load_with_cached_key(vault_path, &derived_key)
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
            config_dir: config_dir.to_path_buf(),
            vault_path: vault_path.to_path_buf(),
            vault_name: vault_name.to_string(),
        })
    }

    pub fn save(&self) -> Result<()> {
        self.storage
            .save_with_cached_key(
                &self.vault,
                &self.vault_path,
                &self.session_data.derived_key,
                &self.session_data.salt,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let session = SessionManager::new(
            &self.config_dir,
            &self.vault_name,
            self.config.session_timeout_minutes,
        );
        session.save_session(&self.session_data.derived_key, &self.session_data.salt)?;

        Ok(())
    }
}
