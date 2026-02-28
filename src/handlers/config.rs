use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::cli::ConfigCommands;
use crate::config::Config;

pub fn handle_config(subcommand: Option<ConfigCommands>, config_dir: &Path) -> Result<()> {
    let mut config = Config::load(config_dir)?;

    match subcommand {
        None => {
            println!("\nCurrent Configuration:");
            println!("  Deck Path: {}", config.deck_path.display());
            println!(
                "  Session Timeout: {} minutes",
                config.session_timeout_minutes
            );
        }
        Some(ConfigCommands::DeckPath { path }) => {
            let new_path = PathBuf::from(path);
            let expanded_path = if new_path.starts_with("~") {
                let home = dirs::home_dir().context("Failed to get home directory")?;
                home.join(new_path.strip_prefix("~").unwrap())
            } else {
                new_path
            };

            config.deck_path = expanded_path.clone();
            config.save(config_dir)?;
            println!("✓ Deck path updated to: {}", expanded_path.display());
        }
        Some(ConfigCommands::SessionTimeout { minutes }) => {
            config.session_timeout_minutes = minutes;
            config.save(config_dir)?;
            println!("✓ Session timeout updated to: {} minutes", minutes);
        }
    }

    Ok(())
}
