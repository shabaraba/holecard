use anyhow::Result;
use std::process::Command;

const SERVICE_NAME: &str = "hc";
const MASTER_PASSWORD_PREFIX: &str = "master_password";

fn account_name(deck_name: &str) -> String {
    format!("{}-{}", MASTER_PASSWORD_PREFIX, deck_name)
}

pub fn save_master_password(deck_name: &str, master_password: &str) -> Result<()> {
    let account = account_name(deck_name);

    // Delete existing item first (old items may have app-specific ACL that causes prompts)
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE_NAME, "-a", &account])
        .output();

    // Add with -A (allow all applications) to prevent security prompts
    let status = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            SERVICE_NAME,
            "-a",
            &account,
            "-w",
            master_password,
            "-A",
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run security command: {}", e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to save password to keychain"));
    }
    Ok(())
}

pub fn load_master_password(deck_name: &str) -> Result<Option<String>> {
    let account = account_name(deck_name);

    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            SERVICE_NAME,
            "-a",
            &account,
            "-w",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run security command: {}", e))?;

    if output.status.success() {
        let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(password))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub fn delete_master_password(deck_name: &str) -> Result<()> {
    let account = account_name(deck_name);
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE_NAME, "-a", &account])
        .output();
    Ok(())
}
