use anyhow::{Context, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::cli::input;
use crate::config::Config;
use crate::context::VaultContext;
use crate::domain::{CryptoService, Entry, Vault};
use crate::infrastructure::{CryptoServiceImpl, KeyringManager, SessionManager, VaultStorage};

pub fn handle_init(keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let secret_key_exists = keyring.load_secret_key().is_ok();
    let config = Config::load(config_dir)?;
    let vault_exists = config.vault_path.exists();

    if secret_key_exists || vault_exists {
        println!("\n⚠ Vault already exists!");
        if !input::prompt_confirm_reinit()? {
            println!("Initialization cancelled.");
            return Ok(());
        }

        println!("\n🗑️  Clearing existing vault data...");

        keyring.delete_secret_key()?;

        if vault_exists {
            std::fs::remove_file(&config.vault_path).context("Failed to delete vault file")?;
        }

        let session = SessionManager::new(config_dir, config.session_timeout_minutes);
        session.clear_session()?;

        println!("✓ Existing vault data cleared\n");
    }

    println!("========================================");
    println!("     Vault Initialization");
    println!("========================================");
    println!("\nPlease set your Master Password.");
    println!("Requirements:");
    println!("  • At least 12 characters");
    println!("  • This will be needed to access your vault");
    println!("========================================\n");

    let master_password = input::prompt_master_password_confirm()?;

    let crypto = CryptoServiceImpl::new();
    let secret_key = crypto.generate_secret_key();

    keyring.save_secret_key(&secret_key)?;

    let config = Config::load(config_dir)?;
    let mut vault = Vault::new();
    let storage = VaultStorage::new(crypto);

    let totp_entry = Entry::new(
        "totp".to_string(),
        HashMap::new(),
        Some("TOTP secrets storage".to_string()),
    );
    vault
        .add_entry(totp_entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let (derived_key, salt) = storage
        .derive_key_from_vault(&config.vault_path, &master_password, &secret_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    storage.save_with_cached_key(&vault, &config.vault_path, &derived_key, &salt)?;

    println!("\n========================================");
    println!("     Vault Initialization Complete");
    println!("========================================");
    println!("\n✓ Master password set");
    println!("✓ Secret key stored in system keyring");
    println!("✓ TOTP entry created");
    println!("\nIMPORTANT:");
    println!("  • Use 'hc export' regularly to backup your vault");
    println!("  • Keep your export file and password safe");
    println!("  • You need BOTH the export file and its password to restore");
    println!("========================================\n");

    Ok(())
}

pub fn handle_add(
    name: Option<String>,
    fields: Vec<(String, String)>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let entry_name = name.unwrap_or_else(|| input::prompt_entry_name().unwrap());

    let custom_fields = if fields.is_empty() {
        input::prompt_custom_fields()?
    } else {
        fields.into_iter().collect()
    };

    let notes = input::prompt_notes()?;

    let entry = Entry::new(entry_name.clone(), custom_fields, notes);
    ctx.vault
        .add_entry(entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' added successfully!", entry_name);
    Ok(())
}

pub fn handle_get(
    name: &str,
    clip: Option<Option<String>>,
    totp: bool,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let entry = ctx
        .vault
        .get_entry(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("\nEntry: {}", entry.name);
    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S"));

    if !entry.custom_fields.is_empty() {
        println!("\nFields:");
        for (key, value) in &entry.custom_fields {
            println!("  {}: {}", key, value);
        }
    }

    if totp && name == "totp" {
        println!("\n⚠ Use 'hc totp get <service>' to generate TOTP codes");
    }

    if let Some(notes) = &entry.notes {
        println!("\nNotes: {}", notes);
    }

    if let Some(field_name) = clip {
        let value_to_copy = match field_name {
            Some(field) => entry
                .custom_fields
                .get(&field)
                .context(format!("Field '{}' not found", field))?
                .clone(),
            None => {
                if let Some(password) = entry.custom_fields.get("password") {
                    password.clone()
                } else {
                    entry
                        .custom_fields
                        .values()
                        .next()
                        .context("No fields to copy")?
                        .clone()
                }
            }
        };

        let mut ctx = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {:?}", e))?;
        ctx.set_contents(value_to_copy.clone())
            .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {:?}", e))?;

        println!("\n✓ Copied to clipboard (will clear in 30 seconds)");

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(30));
            if let Ok(mut ctx) = ClipboardContext::new() {
                let _ = ctx.set_contents(String::new());
            }
        });
    }

    Ok(())
}

pub fn handle_list(keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let entries = ctx.vault.list_entries();

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    println!("\nEntries:");
    for entry in entries {
        println!("  • {}", entry.name);
        if !entry.custom_fields.is_empty() {
            println!(
                "    Fields: {}",
                entry
                    .custom_fields
                    .keys()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    Ok(())
}

pub fn handle_edit(name: &str, keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let entry = ctx
        .vault
        .get_entry_mut(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Editing entry: {}", entry.name);

    let new_fields = input::prompt_custom_fields()?;
    entry.update_fields(new_fields);

    let new_notes = input::prompt_notes()?;
    entry.update_notes(new_notes);

    ctx.save()?;

    println!("✓ Entry '{}' updated successfully!", name);
    Ok(())
}

pub fn handle_rm(name: &str, keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    ctx.vault
        .remove_entry(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' removed successfully!", name);
    Ok(())
}
