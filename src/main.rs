mod cli;
mod config;
mod domain;
mod infrastructure;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{input, Cli, Commands};
use config::{get_config_dir, Config};
use copypasta::{ClipboardContext, ClipboardProvider};
use domain::{CryptoService, Entry, Vault};
use infrastructure::{CryptoServiceImpl, KeyringManager, VaultStorage};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

struct VaultContext {
    vault: Vault,
    storage: VaultStorage<CryptoServiceImpl>,
    master_password: String,
    secret_key: String,
    config: Config,
}

impl VaultContext {
    fn load(keyring: &KeyringManager, config_dir: &PathBuf) -> Result<Self> {
        let secret_key = keyring.load_secret_key()?;
        let master_password = input::prompt_master_password()?;
        let config = Config::load(config_dir)?;
        let crypto = CryptoServiceImpl::new();
        let storage = VaultStorage::new(crypto);
        let vault = storage.load(&config.vault_path, &master_password, &secret_key)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Self {
            vault,
            storage,
            master_password,
            secret_key,
            config,
        })
    }

    fn save(&self) -> Result<()> {
        self.storage.save(&self.vault, &self.config.vault_path, &self.master_password, &self.secret_key)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = get_config_dir()?;
    let keyring = KeyringManager::new(config_dir.clone());

    match cli.command {
        Commands::Init => handle_init(&keyring, &config_dir),
        Commands::Add { name, field } => handle_add(name, field, &keyring, &config_dir),
        Commands::Get { name, clip } => handle_get(&name, clip, &keyring, &config_dir),
        Commands::List => handle_list(&keyring, &config_dir),
        Commands::Edit { name } => handle_edit(&name, &keyring, &config_dir),
        Commands::Rm { name } => handle_rm(&name, &keyring, &config_dir),
    }
}

fn handle_init(keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
    let secret_key = keyring.load_secret_key();
    if secret_key.is_ok() {
        println!("Vault already initialized.");
        return Ok(());
    }

    let crypto = CryptoServiceImpl::new();
    let secret_key = crypto.generate_secret_key();

    keyring.save_secret_key(&secret_key)?;

    let secret_key_path = config_dir.join("secret_key_backup.txt");
    std::fs::write(&secret_key_path, &secret_key)
        .context("Failed to write secret key backup")?;

    println!("\n========================================");
    println!("     Vault Initialization Complete");
    println!("========================================");
    println!("\nSecret Key has been saved to:");
    println!("  {}", secret_key_path.display());
    println!("\nIMPORTANT:");
    println!("  1. Copy this file to a secure backup location");
    println!("  2. Delete the file after backing up: rm {}", secret_key_path.display());
    println!("  3. You will need the Secret Key + Master Password to access your vault");
    println!("========================================\n");

    let master_password = input::prompt_master_password_confirm()?;

    let config = Config::load(config_dir)?;
    let vault = Vault::new();
    let storage = VaultStorage::new(crypto);

    storage.save(&vault, &config.vault_path, &master_password, &secret_key)?;

    println!("\n✓ Vault initialized successfully!");
    Ok(())
}

fn handle_add(
    name: Option<String>,
    fields: Vec<(String, String)>,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
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
    ctx.vault.add_entry(entry).map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' added successfully!", entry_name);
    Ok(())
}

fn handle_get(
    name: &str,
    clip: bool,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let entry = ctx.vault.get_entry(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("\nEntry: {}", entry.name);
    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S"));

    if !entry.custom_fields.is_empty() {
        println!("\nFields:");
        for (key, value) in &entry.custom_fields {
            println!("  {}: {}", key, value);
        }
    }

    if let Some(notes) = &entry.notes {
        println!("\nNotes: {}", notes);
    }

    if clip {
        let value_to_copy = entry
            .custom_fields
            .values()
            .next()
            .context("No fields to copy")?;

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

fn handle_list(keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
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
            println!("    Fields: {}", entry.custom_fields.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", "));
        }
    }

    Ok(())
}

fn handle_edit(name: &str, keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let entry = ctx.vault.get_entry_mut(name).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Editing entry: {}", entry.name);

    let new_fields = input::prompt_custom_fields()?;
    entry.update_fields(new_fields);

    let new_notes = input::prompt_notes()?;
    entry.update_notes(new_notes);

    ctx.save()?;

    println!("✓ Entry '{}' updated successfully!", name);
    Ok(())
}

fn handle_rm(name: &str, keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    ctx.vault.remove_entry(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' removed successfully!", name);
    Ok(())
}
