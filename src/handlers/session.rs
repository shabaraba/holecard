use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::infrastructure::{SessionManager, VaultRegistry};

pub fn handle_lock(config_dir: &Path) -> Result<()> {
    let config = Config::load(config_dir)?;
    let registry = VaultRegistry::load(config_dir)?;

    let active_vault = registry.get_active_vault()?;
    let session = SessionManager::new(
        config_dir,
        &active_vault.name,
        config.session_timeout_minutes,
    );

    session.clear_session()?;
    println!("✓ Vault '{}' locked. Session cleared.", active_vault.name);

    Ok(())
}

pub fn handle_status(config_dir: &Path) -> Result<()> {
    let config = Config::load(config_dir)?;
    let registry = VaultRegistry::load(config_dir)?;

    let active_vault = registry.get_active_vault()?;
    let session = SessionManager::new(
        config_dir,
        &active_vault.name,
        config.session_timeout_minutes,
    );

    if session.is_active() {
        println!(
            "🔓 Vault '{}' is unlocked (session active)",
            active_vault.name
        );
        println!(
            "   Session timeout: {} minutes",
            config.session_timeout_minutes
        );
    } else {
        println!("🔒 Vault '{}' is locked", active_vault.name);
    }

    Ok(())
}
