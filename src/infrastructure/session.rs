use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE_NAME: &str = "hc-session";
const DERIVED_KEY_USER: &str = "derived_key";

#[derive(Debug, Serialize, Deserialize)]
struct SessionMetadata {
    created_at: u64,
    last_accessed: u64,
    salt: String,
}

pub struct SessionData {
    pub derived_key: [u8; 32],
    pub salt: [u8; 16],
}

pub struct SessionManager {
    session_file: PathBuf,
    timeout_minutes: u64,
}

impl SessionManager {
    pub fn new(config_dir: &PathBuf, timeout_minutes: u64) -> Self {
        Self {
            session_file: config_dir.join("session.json"),
            timeout_minutes,
        }
    }

    pub fn save_session(&self, derived_key: &[u8; 32], salt: &[u8; 16]) -> Result<()> {
        let encoded_key = BASE64.encode(derived_key);
        let encoded_salt = BASE64.encode(salt);
        let now = current_timestamp();

        match Entry::new(SERVICE_NAME, DERIVED_KEY_USER) {
            Ok(entry) => {
                entry
                    .set_password(&encoded_key)
                    .context("Failed to save session key to keyring")?;
            }
            Err(_) => {
                return Err(anyhow::anyhow!("Keyring not available for session caching"));
            }
        }

        let metadata = SessionMetadata {
            created_at: now,
            last_accessed: now,
            salt: encoded_salt,
        };
        let json = serde_json::to_string(&metadata)?;
        fs::write(&self.session_file, json)?;

        Ok(())
    }

    pub fn load_session(&self) -> Result<Option<SessionData>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let metadata: SessionMetadata = {
            let content = fs::read_to_string(&self.session_file)?;
            serde_json::from_str(&content)?
        };

        let now = current_timestamp();
        let elapsed_minutes = (now - metadata.last_accessed) / 60;

        if elapsed_minutes >= self.timeout_minutes {
            self.clear_session()?;
            return Ok(None);
        }

        let encoded_key = match Entry::new(SERVICE_NAME, DERIVED_KEY_USER) {
            Ok(entry) => match entry.get_password() {
                Ok(key) => key,
                Err(_) => return Ok(None),
            },
            Err(_) => return Ok(None),
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

        Ok(Some(SessionData { derived_key, salt }))
    }

    pub fn clear_session(&self) -> Result<()> {
        if let Ok(entry) = Entry::new(SERVICE_NAME, DERIVED_KEY_USER) {
            let _ = entry.delete_password();
        }

        if self.session_file.exists() {
            fs::remove_file(&self.session_file)?;
        }

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.load_session().ok().flatten().is_some()
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
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
