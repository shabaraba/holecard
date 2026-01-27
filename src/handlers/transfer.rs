use anyhow::{Context, Result};
use std::path::Path;

use crate::cli::input;
use crate::context::VaultContext;
use crate::domain::Entry;
use crate::infrastructure::{decrypt_for_import, encrypt_for_export, KeyringManager};

pub fn handle_export(file: &str, keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let entries = ctx.vault.list_entries();

    let export_data: Vec<&Entry> = entries.into_iter().collect();
    let json = serde_json::to_string_pretty(&export_data).context("Failed to serialize entries")?;

    println!("\nSet a password to encrypt the export file:");
    let password = input::prompt_export_password()?;

    let encrypted = encrypt_for_export(json.as_bytes(), &password)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt export: {}", e))?;

    std::fs::write(file, &encrypted).context("Failed to write export file")?;

    println!(
        "\n✓ Exported {} entries to {} (encrypted)",
        export_data.len(),
        file
    );

    Ok(())
}

pub fn handle_import(
    file: &str,
    overwrite: bool,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let encrypted_data = std::fs::read(file).context("Failed to read import file")?;

    println!("\nEnter the password used to encrypt this export:");
    let password = input::prompt_import_password()?;

    let decrypted = decrypt_for_import(&encrypted_data, &password)
        .map_err(|_| anyhow::anyhow!("Failed to decrypt: wrong password or corrupted file"))?;

    let json = String::from_utf8(decrypted).context("Failed to decode decrypted data as UTF-8")?;
    let entries: Vec<Entry> = serde_json::from_str(&json).context("Failed to parse import file")?;

    let mut imported = 0;
    let mut overwritten = 0;
    let mut skipped = 0;

    for entry in entries {
        match ctx.vault.import_entry(entry.clone(), overwrite) {
            Ok(was_overwritten) => {
                if was_overwritten {
                    overwritten += 1;
                } else {
                    imported += 1;
                }
            }
            Err(_) => {
                println!("  Skipped '{}' (already exists)", entry.name);
                skipped += 1;
            }
        }
    }

    ctx.save()?;

    println!("\n✓ Import complete:");
    println!("  • {} entries imported", imported);
    if overwritten > 0 {
        println!("  • {} entries overwritten", overwritten);
    }
    if skipped > 0 {
        println!(
            "  • {} entries skipped (use --overwrite to replace)",
            skipped
        );
    }

    Ok(())
}
