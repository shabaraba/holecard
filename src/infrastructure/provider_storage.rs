use crate::domain::{
    error::ProviderError,
    provider::{Provider, ProviderConfig},
    providers::{cloudflare::CloudflareProvider, github::GitHubProvider},
    CryptoError, CryptoService,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SALT_LEN: usize = 16;

/// Provider storage with encryption
pub struct ProviderStorage<C: CryptoService> {
    crypto: C,
}

impl<C: CryptoService> ProviderStorage<C> {
    pub fn new(crypto: C) -> Self {
        Self { crypto }
    }

    /// Load all provider configurations
    pub fn load(
        &self,
        path: &Path,
        derived_key: &[u8; 32],
    ) -> Result<HashMap<String, ProviderConfig>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let encrypted_data = fs::read(path)
            .context("Failed to read provider config file")?;

        if encrypted_data.len() < SALT_LEN {
            return Err(CryptoError::InvalidData("Provider config file too short".to_string()).into());
        }

        let ciphertext = &encrypted_data[SALT_LEN..];
        let decrypted_data = self.crypto.decrypt_with_key(ciphertext, derived_key)?;

        let configs: HashMap<String, ProviderConfig> = serde_json::from_slice(&decrypted_data)
            .context("Failed to deserialize provider config")?;

        Ok(configs)
    }

    /// Save all provider configurations
    pub fn save(
        &self,
        configs: &HashMap<String, ProviderConfig>,
        path: &Path,
        derived_key: &[u8; 32],
        salt: &[u8; 16],
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create provider config directory")?;
        }

        let json_data = serde_json::to_vec(configs)
            .context("Failed to serialize provider config")?;

        let ciphertext = self.crypto.encrypt_with_key(&json_data, derived_key)?;

        let mut encrypted_data = Vec::with_capacity(SALT_LEN + ciphertext.len());
        encrypted_data.extend_from_slice(salt);
        encrypted_data.extend_from_slice(&ciphertext);

        self.write_config_file(path, &encrypted_data)
    }

    fn write_config_file(&self, path: &Path, encrypted_data: &[u8]) -> Result<()> {
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, encrypted_data)
            .context("Failed to write provider config file")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)
                .context("Failed to get file metadata")?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&temp_path, perms)
                .context("Failed to set file permissions")?;
        }

        fs::rename(&temp_path, path)
            .context("Failed to finalize provider config file")?;

        Ok(())
    }
}

/// Create provider instance from config
pub fn create_provider(config: &ProviderConfig) -> Result<Box<dyn Provider>> {
    match config.provider_type.as_str() {
        "github" => {
            let repo = config
                .credentials
                .get("repo")
                .ok_or_else(|| ProviderError::ConfigError("Missing 'repo' credential".to_string()))?;
            let token = config
                .credentials
                .get("token")
                .ok_or_else(|| ProviderError::ConfigError("Missing 'token' credential".to_string()))?;

            Ok(Box::new(GitHubProvider::new(repo.clone(), token.clone())))
        }
        "cloudflare" => {
            let account_id = config
                .credentials
                .get("account_id")
                .ok_or_else(|| ProviderError::ConfigError("Missing 'account_id' credential".to_string()))?;
            let worker_name = config
                .credentials
                .get("worker_name")
                .ok_or_else(|| ProviderError::ConfigError("Missing 'worker_name' credential".to_string()))?;
            let token = config
                .credentials
                .get("token")
                .ok_or_else(|| ProviderError::ConfigError("Missing 'token' credential".to_string()))?;

            Ok(Box::new(CloudflareProvider::new(
                account_id.clone(),
                worker_name.clone(),
                token.clone(),
            )))
        }
        _ => Err(ProviderError::ConfigError(format!(
            "Unknown provider type: {}",
            config.provider_type
        )).into()),
    }
}
