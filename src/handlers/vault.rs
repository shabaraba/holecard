use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::cli::input;
use crate::domain::{Entry, PasswordService};
use crate::handlers::password::copy_to_clipboard_with_clear;
use crate::infrastructure::KeyringManager;
use crate::multi_vault_context::MultiVaultContext;

pub fn handle_init(keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    println!("⚠️  'hc init' is deprecated.");
    println!("    Use 'hc vault create default' instead.\n");
    println!("Proceeding with vault creation...\n");

    crate::handlers::vault_management::handle_vault(
        crate::cli::commands::VaultCommands::Create {
            name: "default".to_string(),
        },
        keyring,
        config_dir,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_add(
    name: Option<String>,
    fields: Vec<(String, String)>,
    file_fields: Vec<(String, String)>,
    generate: bool,
    gen_length: Option<usize>,
    gen_memorable: bool,
    gen_words: Option<usize>,
    gen_no_uppercase: bool,
    gen_no_lowercase: bool,
    gen_no_digits: bool,
    gen_no_symbols: bool,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    let entry_name = name.unwrap_or_else(|| input::prompt_entry_name().unwrap());

    let mut custom_fields: HashMap<String, String> = if fields.is_empty() && file_fields.is_empty()
    {
        input::prompt_custom_fields()?
    } else {
        let mut combined = HashMap::new();
        combined.extend(fields);
        combined.extend(file_fields);
        combined
    };

    if generate {
        let password = PasswordService::generate_from_cli(
            gen_memorable,
            gen_words,
            gen_length,
            gen_no_uppercase,
            gen_no_lowercase,
            gen_no_digits,
            gen_no_symbols,
        )?;

        custom_fields.insert("password".to_string(), password);
        println!("Generated password for 'password' field (hidden)");
    }

    let notes = input::prompt_notes()?;

    let entry = Entry::new(entry_name.clone(), custom_fields, notes);
    ctx.inner
        .vault
        .add_entry(entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("Entry '{}' added successfully!", entry_name);
    Ok(())
}

pub fn handle_get(
    name: &str,
    clip: Option<Option<String>>,
    totp: bool,
    show: bool,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entry = ctx
        .inner
        .vault
        .get_entry(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("\nEntry: {}", entry.name);
    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S"));

    if !entry.custom_fields.is_empty() {
        println!("\nFields:");
        if show {
            let _password = input::prompt_master_password()?;
            for (key, value) in &entry.custom_fields {
                println!("  {}: {}", key, value);
            }
        } else {
            for key in entry.custom_fields.keys() {
                println!("  {}: ******", key);
            }
        }
    }

    if totp && name == "totp" {
        println!("\n⚠ Use 'hc totp get <service>' to generate TOTP codes");
    }

    if let Some(notes) = &entry.notes {
        if show {
            println!("\nNotes: {}", notes);
        } else {
            println!("\nNotes: ******");
        }
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

        copy_to_clipboard_with_clear(&value_to_copy)?;
        println!("\nCopied to clipboard (will clear in 30 seconds)");
    }

    Ok(())
}

pub fn handle_list(
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entries = ctx.inner.vault.list_entries();

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

pub fn handle_edit(
    name: &str,
    fields: Vec<(String, String)>,
    file_fields: Vec<(String, String)>,
    rm_fields: Vec<String>,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    let entry = ctx
        .inner
        .vault
        .get_entry_mut(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if !fields.is_empty() || !file_fields.is_empty() || !rm_fields.is_empty() {
        for (key, value) in fields {
            entry.custom_fields.insert(key.clone(), value);
            println!("✓ Field '{}' updated", key);
        }

        for (key, value) in file_fields {
            entry.custom_fields.insert(key.clone(), value);
            println!("✓ Field '{}' updated from file", key);
        }

        for key in rm_fields {
            if entry.custom_fields.remove(&key).is_some() {
                println!("✓ Field '{}' removed", key);
            } else {
                println!("⚠ Field '{}' not found", key);
            }
        }

        entry.touch();
        ctx.save()?;
        println!("✓ Entry '{}' updated successfully!", name);
    } else {
        println!(
            "⚠ No changes specified. Use -f to add/update fields, --file to add from file, or --rm-field to remove fields."
        );
    }

    Ok(())
}

pub fn handle_edit_interactive(
    name: &str,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    let entry = ctx
        .inner
        .vault
        .get_entry_mut(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Editing entry: {}", entry.name);

    loop {
        match input::prompt_edit_menu(entry)? {
            input::EditAction::Done => break,
            input::EditAction::EditField(key) => {
                let value = input::prompt_field_value(&key)?;
                entry.custom_fields.insert(key.clone(), value);
                println!("✓ Field '{}' updated", key);
            }
            input::EditAction::AddField => {
                let (key, value) = input::prompt_new_field()?;
                entry.custom_fields.insert(key.clone(), value);
                println!("✓ Field '{}' added", key);
            }
            input::EditAction::DeleteField(key) => {
                if entry.custom_fields.remove(&key).is_some() {
                    println!("✓ Field '{}' removed", key);
                } else {
                    println!("⚠ Field '{}' not found", key);
                }
            }
            input::EditAction::EditNotes => {
                let new_notes = input::prompt_notes()?;
                entry.update_notes(new_notes);
                println!("✓ Notes updated");
            }
        }
    }

    entry.touch();
    ctx.save()?;

    println!("✓ Entry '{}' updated successfully!", name);
    Ok(())
}

pub fn handle_rm(
    name: &str,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    ctx.inner
        .vault
        .remove_entry(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' removed successfully!", name);
    Ok(())
}
