use anyhow::{Context, Result};
use keyring::Entry;
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "hc";
const USERNAME: &str = "secret_key";
const MASTER_PASSWORD_PREFIX: &str = "master_password";

pub struct KeyringManager {
    fallback_path: PathBuf,
}

impl KeyringManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let fallback_path = config_dir.join("secret_key");
        Self { fallback_path }
    }

    pub fn save_secret_key(&self, secret_key: &str) -> Result<()> {
        match Entry::new(SERVICE_NAME, USERNAME) {
            Ok(entry) => {
                entry
                    .set_password(secret_key)
                    .context("Failed to save secret key to OS keyring")?;
                Ok(())
            }
            Err(_) => {
                fs::write(&self.fallback_path, secret_key)
                    .context("Failed to save secret key to fallback file")?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&self.fallback_path)?.permissions();
                    perms.set_mode(0o600);
                    fs::set_permissions(&self.fallback_path, perms)?;
                }
                Ok(())
            }
        }
    }

    pub fn load_secret_key(&self) -> Result<String> {
        let try_keyring = || -> Option<String> {
            Entry::new(SERVICE_NAME, USERNAME)
                .ok()?
                .get_password()
                .ok()
                .map(|s| s.trim().to_string())
        };

        if let Some(key) = try_keyring() {
            return Ok(key);
        }

        if self.fallback_path.exists() {
            fs::read_to_string(&self.fallback_path)
                .map(|s| s.trim().to_string())
                .context("Failed to read secret key from fallback file")
        } else {
            Err(anyhow::anyhow!(
                "Secret key not found. Please run 'hc init' first."
            ))
        }
    }

    #[allow(dead_code)]
    pub fn delete_secret_key(&self) -> Result<()> {
        if let Ok(entry) = Entry::new(SERVICE_NAME, USERNAME) {
            let _ = entry.delete_password();
        }

        if self.fallback_path.exists() {
            fs::remove_file(&self.fallback_path)
                .context("Failed to delete fallback secret key file")?;
        }

        Ok(())
    }

    pub fn save_master_password(&self, vault_name: &str, master_password: &str) -> Result<()> {
        let username = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        match Entry::new(SERVICE_NAME, &username) {
            Ok(entry) => {
                entry
                    .set_password(master_password)
                    .context("Failed to save master password to OS keyring")?;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to access keyring for master password: {}",
                e
            )),
        }
    }

    pub fn load_master_password(&self, vault_name: &str) -> Result<Option<String>> {
        let username = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        match Entry::new(SERVICE_NAME, &username) {
            Ok(entry) => match entry.get_password() {
                Ok(pwd) => Ok(Some(pwd.trim().to_string())),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn delete_master_password(&self, vault_name: &str) -> Result<()> {
        let username = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        if let Ok(entry) = Entry::new(SERVICE_NAME, &username) {
            let _ = entry.delete_password();
        }
        Ok(())
    }
}
