use crate::domain::{CryptoError, CryptoService, Vault};
use std::fs;
use std::path::Path;

pub struct VaultStorage<C: CryptoService> {
    crypto: C,
}

impl<C: CryptoService> VaultStorage<C> {
    pub fn new(crypto: C) -> Self {
        Self { crypto }
    }

    pub fn load(&self, path: &Path, master_password: &str, secret_key: &str) -> Result<Vault, CryptoError> {
        if !path.exists() {
            return Ok(Vault::new());
        }

        let encrypted_data = fs::read(path)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to read vault file: {}", e)))?;

        let decrypted_data = self
            .crypto
            .decrypt(&encrypted_data, master_password, secret_key)?;

        let vault: Vault = serde_json::from_slice(&decrypted_data)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to deserialize vault: {}", e)))?;

        Ok(vault)
    }

    pub fn save(&self, vault: &Vault, path: &Path, master_password: &str, secret_key: &str) -> Result<(), CryptoError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CryptoError::InvalidData(format!("Failed to create vault directory: {}", e)))?;
        }

        let json_data = serde_json::to_vec(vault)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to serialize vault: {}", e)))?;

        let encrypted_data = self
            .crypto
            .encrypt(&json_data, master_password, secret_key)?;

        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, encrypted_data)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to write vault file: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)
                .map_err(|e| CryptoError::InvalidData(format!("Failed to get file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&temp_path, perms)
                .map_err(|e| CryptoError::InvalidData(format!("Failed to set file permissions: {}", e)))?;
        }

        fs::rename(&temp_path, path)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to finalize vault file: {}", e)))?;

        Ok(())
    }
}
