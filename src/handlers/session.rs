use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::infrastructure::{DeckRegistry, SessionManager};

pub fn handle_lock(config_dir: &Path) -> Result<()> {
    let config = Config::load(config_dir)?;
    let registry = DeckRegistry::load(config_dir)?;

    let active_deck = registry.get_active_deck()?;
    let session = SessionManager::new(
        config_dir,
        &active_deck.name,
        config.session_timeout_minutes,
    );

    session.clear_session()?;
    println!("✓ Deck '{}' locked. Session cleared.", active_deck.name);

    Ok(())
}

pub fn handle_status(config_dir: &Path) -> Result<()> {
    let config = Config::load(config_dir)?;
    let registry = DeckRegistry::load(config_dir)?;

    let active_deck = registry.get_active_deck()?;
    let session = SessionManager::new(
        config_dir,
        &active_deck.name,
        config.session_timeout_minutes,
    );

    if session.is_active() {
        println!(
            "🔓 Deck '{}' is unlocked (session active)",
            active_deck.name
        );
        println!(
            "   Session timeout: {} minutes",
            config.session_timeout_minutes
        );
    } else {
        println!("🔒 Deck '{}' is locked", active_deck.name);
    }

    Ok(())
}
