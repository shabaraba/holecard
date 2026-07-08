use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct SessionMetadata {
    created_at: u64,
    last_accessed: u64,
    salt: String,
    derived_key: Option<String>,
    #[serde(default, alias = "entry_names")]
    card_names: Vec<String>,
}

pub struct SessionData {
    pub derived_key: [u8; 32],
    pub salt: [u8; 16],
    #[allow(dead_code)]
    pub hand_names: Vec<String>,
}

pub struct SessionManager {
    session_file: PathBuf,
    timeout_minutes: u64,
    // Legacy: service name used when migrating from keychain-based storage
    legacy_service_name: String,
}

impl SessionManager {
    pub fn new(config_dir: &Path, deck_name: &str, timeout_minutes: u64) -> Self {
        Self {
            session_file: config_dir.join(format!("session_{}.json", deck_name)),
            timeout_minutes,
            legacy_service_name: format!("hc-session-{}", deck_name),
        }
    }

    pub fn save_session(
        &self,
        derived_key: &[u8; 32],
        salt: &[u8; 16],
        card_names: Vec<String>,
    ) -> Result<()> {
        let encoded_key = BASE64.encode(derived_key);
        let encoded_salt = BASE64.encode(salt);
        let now = current_timestamp();

        let metadata = SessionMetadata {
            created_at: now,
            last_accessed: now,
            salt: encoded_salt,
            derived_key: Some(encoded_key),
            card_names,
        };
        let json = serde_json::to_string(&metadata)?;
        fs::write(&self.session_file, &json)?;
        set_private_permissions(&self.session_file)?;

        Ok(())
    }

    pub fn load_session(&self) -> Result<Option<SessionData>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.session_file)?;
        let metadata: SessionMetadata = serde_json::from_str(&content)?;

        let now = current_timestamp();
        let elapsed_minutes = (now - metadata.last_accessed) / 60;

        if elapsed_minutes >= self.timeout_minutes {
            self.clear_session()?;
            return Ok(None);
        }

        let encoded_key = match &metadata.derived_key {
            Some(k) => k.clone(),
            None => {
                // Migrate from legacy keychain storage
                match self.load_from_legacy_keychain() {
                    Some(k) => k,
                    None => return Ok(None),
                }
            }
        };

        let key_bytes = BASE64
            .decode(&encoded_key)
            .context("Failed to decode session key")?;

        let salt_bytes = BASE64
            .decode(&metadata.salt)
            .context("Failed to decode session salt")?;

        if key_bytes.len() != 32 || salt_bytes.len() != 16 {
            self.clear_session()?;
            return Ok(None);
        }

        let mut derived_key = [0u8; 32];
        derived_key.copy_from_slice(&key_bytes);

        let mut salt = [0u8; 16];
        salt.copy_from_slice(&salt_bytes);

        self.touch_session()?;

        Ok(Some(SessionData {
            derived_key,
            salt,
            hand_names: metadata.card_names,
        }))
    }

    pub fn clear_session(&self) -> Result<()> {
        self.delete_legacy_keychain_entry();

        if self.session_file.exists() {
            fs::remove_file(&self.session_file)?;
        }

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.load_session().ok().flatten().is_some()
    }

    pub fn load_card_names(&self) -> Result<Vec<String>> {
        if !self.session_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.session_file)?;
        let metadata: SessionMetadata = serde_json::from_str(&content)?;

        Ok(metadata.card_names)
    }

    fn touch_session(&self) -> Result<()> {
        if !self.session_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.session_file)?;
        let mut metadata: SessionMetadata = serde_json::from_str(&content)?;
        metadata.last_accessed = current_timestamp();

        let json = serde_json::to_string(&metadata)?;
        fs::write(&self.session_file, json)?;

        Ok(())
    }

    fn load_from_legacy_keychain(&self) -> Option<String> {
        use keyring::Entry;
        Entry::new(&self.legacy_service_name, "derived_key")
            .ok()?
            .get_password()
            .ok()
    }

    fn delete_legacy_keychain_entry(&self) {
        use keyring::Entry;
        if let Ok(entry) = Entry::new(&self.legacy_service_name, "derived_key") {
            let _ = entry.delete_password();
        }
    }
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms).context("Failed to set session file permissions")
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
