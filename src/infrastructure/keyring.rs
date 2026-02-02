use anyhow::{Context, Result};
use keyring::Entry;
use std::fs;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

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

    #[cfg(target_os = "macos")]
    pub fn save_master_password(&self, vault_name: &str, master_password: &str) -> Result<()> {
        use std::io::Write;

        let label = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        let service = SERVICE_NAME;

        let script = format!(
            r#"
import Foundation
import Security

let service = "{}" as CFString
let account = "{}" as CFString
let password = "{}".data(using: .utf8)!

var query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
    kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    kSecValueData as String: password
]

SecItemDelete(query as CFDictionary)

let status = SecItemAdd(query as CFDictionary, nil)
if status == errSecSuccess {{
    print("success")
    exit(0)
}} else {{
    print("error: \(status)")
    exit(1)
}}
"#,
            service,
            label,
            master_password.replace("\"", "\\\"").replace("\\", "\\\\")
        );

        let mut child = Command::new("swift")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn swift")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Failed to save password to keychain.\nstdout: {}\nstderr: {}",
                stdout,
                stderr
            ));
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
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

    #[cfg(target_os = "macos")]
    pub fn load_master_password(&self, vault_name: &str) -> Result<Option<String>> {
        use std::io::Write;

        let label = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        let service = SERVICE_NAME;

        let script = format!(
            r#"
import Foundation
import Security

let service = "{}" as CFString
let account = "{}" as CFString

let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
    kSecReturnData as String: true
]

var item: CFTypeRef?
let status = SecItemCopyMatching(query as CFDictionary, &item)

if status == errSecSuccess, let data = item as? Data, let password = String(data: data, encoding: .utf8) {{
    print(password)
    exit(0)
}} else {{
    exit(1)
}}
"#,
            service, label
        );

        let mut child = Command::new("swift")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(password))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(target_os = "macos"))]
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

    #[cfg(target_os = "macos")]
    #[allow(dead_code)]
    pub fn delete_master_password(&self, vault_name: &str) -> Result<()> {
        use std::io::Write;

        let label = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        let service = SERVICE_NAME;

        let script = format!(
            r#"
import Foundation
import Security

let service = "{}" as CFString
let account = "{}" as CFString

let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: account,
]

SecItemDelete(query as CFDictionary)
exit(0)
"#,
            service, label
        );

        let mut child = Command::new("swift")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes())?;
        }

        let _ = child.wait_with_output()?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    #[allow(dead_code)]
    pub fn delete_master_password(&self, vault_name: &str) -> Result<()> {
        let username = format!("{}-{}", MASTER_PASSWORD_PREFIX, vault_name);
        if let Ok(entry) = Entry::new(SERVICE_NAME, &username) {
            let _ = entry.delete_password();
        }
        Ok(())
    }
}
