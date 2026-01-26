use crate::domain::{CryptoError, CryptoService, Vault};
use std::fs;
use std::path::Path;

const SALT_LEN: usize = 16;

pub struct VaultStorage<C: CryptoService> {
    crypto: C,
}

impl<C: CryptoService> VaultStorage<C> {
    pub fn new(crypto: C) -> Self {
        Self { crypto }
    }

    pub fn load_with_cached_key(
        &self,
        path: &Path,
        derived_key: &[u8; 32],
    ) -> Result<Vault, CryptoError> {
        if !path.exists() {
            return Ok(Vault::new());
        }

        let encrypted_data = fs::read(path)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to read vault file: {}", e)))?;

        if encrypted_data.len() < SALT_LEN {
            return Err(CryptoError::InvalidData("Vault file too short".to_string()));
        }

        let ciphertext = &encrypted_data[SALT_LEN..];
        let decrypted_data = self.crypto.decrypt_with_key(ciphertext, derived_key)?;

        let vault: Vault = serde_json::from_slice(&decrypted_data)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to deserialize vault: {}", e)))?;

        Ok(vault)
    }

    pub fn save_with_cached_key(
        &self,
        vault: &Vault,
        path: &Path,
        derived_key: &[u8; 32],
        salt: &[u8; 16],
    ) -> Result<(), CryptoError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CryptoError::InvalidData(format!("Failed to create vault directory: {}", e))
            })?;
        }

        let json_data = serde_json::to_vec(vault)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to serialize vault: {}", e)))?;

        let ciphertext = self.crypto.encrypt_with_key(&json_data, derived_key)?;

        let mut encrypted_data = Vec::with_capacity(SALT_LEN + ciphertext.len());
        encrypted_data.extend_from_slice(salt);
        encrypted_data.extend_from_slice(&ciphertext);

        self.write_vault_file(path, &encrypted_data)
    }

    pub fn derive_key_from_vault(
        &self,
        path: &Path,
        master_password: &str,
        secret_key: &str,
    ) -> Result<([u8; 32], [u8; 16]), CryptoError> {
        let salt = if path.exists() {
            let encrypted_data = fs::read(path).map_err(|e| {
                CryptoError::InvalidData(format!("Failed to read vault file: {}", e))
            })?;
            if encrypted_data.len() < SALT_LEN {
                return Err(CryptoError::InvalidData("Vault file too short".to_string()));
            }
            let mut salt = [0u8; 16];
            salt.copy_from_slice(&encrypted_data[..SALT_LEN]);
            salt
        } else {
            use rand::RngCore;
            let mut salt = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut salt);
            salt
        };

        let derived_key = self.crypto.derive_key(master_password, secret_key, &salt)?;
        Ok((derived_key, salt))
    }

    fn write_vault_file(&self, path: &Path, encrypted_data: &[u8]) -> Result<(), CryptoError> {
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, encrypted_data)
            .map_err(|e| CryptoError::InvalidData(format!("Failed to write vault file: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)
                .map_err(|e| {
                    CryptoError::InvalidData(format!("Failed to get file metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&temp_path, perms).map_err(|e| {
                CryptoError::InvalidData(format!("Failed to set file permissions: {}", e))
            })?;
        }

        fs::rename(&temp_path, path).map_err(|e| {
            CryptoError::InvalidData(format!("Failed to finalize vault file: {}", e))
        })?;

        Ok(())
    }
}
