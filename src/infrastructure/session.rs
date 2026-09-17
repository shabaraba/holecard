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
    #[serde(default, alias = "entry_names", alias = "card_names")]
    hand_names: Vec<String>,
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
        hand_names: Vec<String>,
    ) -> Result<()> {
        let now = current_timestamp();
        let metadata = SessionMetadata {
            created_at: now,
            last_accessed: now,
            salt: BASE64.encode(salt),
            derived_key: Some(BASE64.encode(derived_key)),
            hand_names,
        };

        self.write_metadata(&metadata)
    }

    pub fn load_session(&self) -> Result<Option<SessionData>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let mut metadata: SessionMetadata = {
            let content = fs::read_to_string(&self.session_file)?;
            serde_json::from_str(&content)?
        };

        let now = current_timestamp();
        let elapsed_minutes = now.saturating_sub(metadata.last_accessed) / 60;

        if elapsed_minutes >= self.timeout_minutes {
            self.clear_session()?;
            return Ok(None);
        }

        let mut migrated_from_legacy = false;
        let encoded_key = match &metadata.derived_key {
            Some(k) => k.clone(),
            None => {
                // Migrate from legacy keychain storage
                match self.load_from_legacy_keychain() {
                    Some(k) => {
                        metadata.derived_key = Some(k.clone());
                        migrated_from_legacy = true;
                        k
                    }
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

        metadata.last_accessed = now;
        self.write_metadata(&metadata)?;

        if migrated_from_legacy {
            self.delete_legacy_keychain_entry();
        }

        Ok(Some(SessionData {
            derived_key,
            salt,
            hand_names: metadata.hand_names,
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

    pub fn load_hand_names(&self) -> Result<Vec<String>> {
        if !self.session_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.session_file)?;
        let metadata: SessionMetadata = serde_json::from_str(&content)?;

        Ok(metadata.hand_names)
    }

    fn write_metadata(&self, metadata: &SessionMetadata) -> Result<()> {
        let json = serde_json::to_string(metadata)?;
        write_private_file(&self.session_file, &json)
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
fn write_private_file(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    // Create with 0600 up front so the derived key is never briefly
    // readable under the process umask; re-assert it afterwards in case
    // the file already existed with looser permissions.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .context("Failed to create session file")?;
    file.write_all(contents.as_bytes())
        .context("Failed to write session file")?;

    set_private_permissions(path)
}

#[cfg(not(unix))]
fn write_private_file(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).context("Failed to write session file")
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms).context("Failed to set session file permissions")
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
