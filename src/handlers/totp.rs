use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::cli::commands::TotpCommands;
use crate::domain::TotpService;
use crate::infrastructure::KeyringManager;
use crate::multi_deck_context::MultiDeckContext;

pub fn handle_totp(
    subcommand: TotpCommands,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    match subcommand {
        TotpCommands::Add { card, secret } => {
            handle_totp_add(&card, &secret, deck_name, keyring, config_dir)
        }
        TotpCommands::Get { card } => handle_totp_get(&card, deck_name, keyring, config_dir),
        TotpCommands::Rm { card } => handle_totp_rm(&card, deck_name, keyring, config_dir),
    }
}

fn handle_totp_add(
    service_name: &str,
    secret: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    TotpService::validate_secret(secret)?;

    let mut ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;

    let totp_entry = ctx.inner.deck.get_hand_mut("totp").map_err(|_| {
        anyhow::anyhow!("TOTP hand not found. Please reinitialize deck with 'hc init'")
    })?;

    if totp_entry.cards.contains_key(service_name) {
        println!(
            "⚠ TOTP secret for '{}' already exists. Overwriting...",
            service_name
        );
    }

    totp_entry
        .cards
        .insert(service_name.to_string(), secret.to_string());
    totp_entry.updated_at = chrono::Utc::now();

    ctx.save()?;
    println!("✓ TOTP secret for '{}' added", service_name);

    Ok(())
}

fn handle_totp_get(
    service_name: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let totp_entry = ctx.inner.deck.get_hand("totp").map_err(|_| {
        anyhow::anyhow!("TOTP hand not found. Please reinitialize deck with 'hc init'")
    })?;

    if let Some(secret) = totp_entry.cards.get(service_name) {
        if secret.is_empty() {
            anyhow::bail!("TOTP secret for '{}' is empty", service_name);
        }

        match TotpService::generate_code(secret) {
            Ok(code) => {
                let remaining = TotpService::get_remaining_seconds();
                println!("\nTOTP Code: {} (valid for {} seconds)", code, remaining);

                let mut clipboard_ctx = ClipboardContext::new()
                    .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {:?}", e))?;
                clipboard_ctx
                    .set_contents(code.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {:?}", e))?;

                println!("✓ Copied to clipboard (will clear in 30 seconds)");

                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(30));
                    if let Ok(mut ctx) = ClipboardContext::new() {
                        let _ = ctx.set_contents(String::new());
                    }
                });
            }
            Err(e) => {
                anyhow::bail!("Failed to generate TOTP code: {}", e);
            }
        }
    } else {
        anyhow::bail!("No TOTP secret found for service '{}'", service_name);
    }

    Ok(())
}

fn handle_totp_rm(
    service_name: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;

    let totp_entry = ctx.inner.deck.get_hand_mut("totp").map_err(|_| {
        anyhow::anyhow!("TOTP hand not found. Please reinitialize deck with 'hc init'")
    })?;

    if totp_entry.cards.remove(service_name).is_some() {
        totp_entry.updated_at = chrono::Utc::now();
        ctx.save()?;
        println!("✓ TOTP secret for '{}' removed", service_name);
    } else {
        println!("⚠ No TOTP secret found for service '{}'", service_name);
    }

    Ok(())
}
