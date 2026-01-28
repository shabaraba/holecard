use anyhow::Result;
use std::path::Path;

use crate::context::VaultContext;
use crate::infrastructure::{KeyringManager, VaultRegistry};

pub struct MultiVaultContext {
    pub vault_name: String,
    pub inner: VaultContext,
}

impl MultiVaultContext {
    pub fn load(
        vault_name: Option<&str>,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<Self> {
        let registry = VaultRegistry::load(config_dir)?;

        let vault_metadata = if let Some(name) = vault_name {
            registry.get_vault(name)?
        } else {
            registry.get_active_vault()?
        };

        let vault_path = &vault_metadata.path;
        let inner = VaultContext::load(
            vault_path,
            &vault_metadata.name,
            keyring,
            config_dir,
        )?;

        registry.touch_vault(&vault_metadata.name)?;

        Ok(Self {
            vault_name: vault_metadata.name,
            inner,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.inner.save()
    }
}
