use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::infrastructure::SessionManager;

pub fn handle_lock(config_dir: &PathBuf) -> Result<()> {
    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, config.session_timeout_minutes);

    session.clear_session()?;
    println!("✓ Vault locked. Session cleared.");

    Ok(())
}

pub fn handle_status(config_dir: &PathBuf) -> Result<()> {
    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, config.session_timeout_minutes);

    if session.is_active() {
        println!("🔓 Vault is unlocked (session active)");
        println!(
            "   Session timeout: {} minutes",
            config.session_timeout_minutes
        );
    } else {
        println!("🔒 Vault is locked");
    }

    Ok(())
}
