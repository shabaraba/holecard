mod cli;
mod config;
mod domain;
mod infrastructure;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{commands::TotpCommands, input, Cli, Commands, ConfigCommands};
use config::{get_config_dir, Config};
use copypasta::{ClipboardContext, ClipboardProvider};
use domain::{CryptoService, Entry, TemplateEngine, TotpService, Vault};
use infrastructure::{
    decrypt_for_import, encrypt_for_export, CryptoServiceImpl, KeyringManager, SessionData,
    SessionManager, VaultStorage,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

struct VaultContext {
    vault: Vault,
    storage: VaultStorage<CryptoServiceImpl>,
    session_data: SessionData,
    config: Config,
    config_dir: PathBuf,
}

impl VaultContext {
    fn load(keyring: &KeyringManager, config_dir: &PathBuf) -> Result<Self> {
        let secret_key = keyring.load_secret_key()?;
        let config = Config::load(config_dir)?;
        let crypto = CryptoServiceImpl::new();
        let storage = VaultStorage::new(crypto);
        let session = SessionManager::new(config_dir, config.session_timeout_minutes);

        let (vault, session_data) = if let Some(cached) = session.load_session()? {
            let vault = storage
                .load_with_cached_key(&config.vault_path, &cached.derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            (vault, cached)
        } else {
            let master_password = input::prompt_master_password()?;
            let (derived_key, salt) = storage
                .derive_key_from_vault(&config.vault_path, &master_password, &secret_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let vault = storage
                .load_with_cached_key(&config.vault_path, &derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let session_data = SessionData { derived_key, salt };
            session.save_session(&derived_key, &salt)?;
            (vault, session_data)
        };

        Ok(Self {
            vault,
            storage,
            session_data,
            config,
            config_dir: config_dir.clone(),
        })
    }

    fn save(&self) -> Result<()> {
        self.storage
            .save_with_cached_key(
                &self.vault,
                &self.config.vault_path,
                &self.session_data.derived_key,
                &self.session_data.salt,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let session = SessionManager::new(&self.config_dir, self.config.session_timeout_minutes);
        session.save_session(&self.session_data.derived_key, &self.session_data.salt)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = get_config_dir()?;
    let keyring = KeyringManager::new(config_dir.clone());

    match cli.command {
        Commands::Init => handle_init(&keyring, &config_dir),
        Commands::Add { name, field } => handle_add(name, field, &keyring, &config_dir),
        Commands::Get { name, clip, totp } => {
            handle_get(&name, clip.clone(), totp, &keyring, &config_dir)
        }
        Commands::List => handle_list(&keyring, &config_dir),
        Commands::Edit { name } => handle_edit(&name, &keyring, &config_dir),
        Commands::Rm { name } => handle_rm(&name, &keyring, &config_dir),
        Commands::Config { subcommand } => handle_config(subcommand, &config_dir),
        Commands::Inject { entry, template } => {
            handle_inject(&entry, &template, &keyring, &config_dir)
        }
        Commands::Run { entry, command } => handle_run(&entry, &command, &keyring, &config_dir),
        Commands::Lock => handle_lock(&config_dir),
        Commands::Status => handle_status(&config_dir),
        Commands::Export { file } => handle_export(&file, &keyring, &config_dir),
        Commands::Import { file, overwrite } => {
            handle_import(&file, overwrite, &keyring, &config_dir)
        }
        Commands::Totp { subcommand } => handle_totp(subcommand, &keyring, &config_dir),
    }
}

fn handle_init(keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
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
    ctx.vault
        .add_entry(entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' added successfully!", entry_name);
    Ok(())
}

fn handle_get(
    name: &str,
    clip: Option<Option<String>>,
    totp: bool,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
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

fn handle_edit(
    name: &str,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
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

fn handle_rm(name: &str, keyring: &KeyringManager, config_dir: &std::path::PathBuf) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    ctx.vault
        .remove_entry(name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    ctx.save()?;

    println!("✓ Entry '{}' removed successfully!", name);
    Ok(())
}

fn handle_config(
    subcommand: Option<ConfigCommands>,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let mut config = Config::load(config_dir)?;

    match subcommand {
        None => {
            println!("\nCurrent Configuration:");
            println!("  Vault Path: {}", config.vault_path.display());
            println!(
                "  Session Timeout: {} minutes",
                config.session_timeout_minutes
            );
        }
        Some(ConfigCommands::VaultPath { path }) => {
            let new_path = PathBuf::from(path);
            let expanded_path = if new_path.starts_with("~") {
                let home = dirs::home_dir().context("Failed to get home directory")?;
                home.join(new_path.strip_prefix("~").unwrap())
            } else {
                new_path
            };

            config.vault_path = expanded_path.clone();
            config.save(config_dir)?;
            println!("✓ Vault path updated to: {}", expanded_path.display());
        }
        Some(ConfigCommands::SessionTimeout { minutes }) => {
            config.session_timeout_minutes = minutes;
            config.save(config_dir)?;
            println!("✓ Session timeout updated to: {} minutes", minutes);
        }
    }

    Ok(())
}

fn handle_inject(
    entry_name: &str,
    template: &str,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let entry = ctx
        .vault
        .get_entry(entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let rendered = TemplateEngine::render(template, entry)?;
    println!("{}", rendered);

    Ok(())
}

fn handle_run(
    entry_name: &str,
    command: &[String],
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let ctx = VaultContext::load(keyring, config_dir)?;
    let entry = ctx
        .vault
        .get_entry(entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Build environment variables: ENTRYNAME_FIELDNAME=value
    // Sanitize entry name for valid env var names
    // let sanitized_name = entry
    //     .name
    //     .to_uppercase()
    //     .replace(|c: char| !c.is_alphanumeric(), "_");

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    // Add vault fields as environment variables (inheriting parent environment)
    for (key, value) in &entry.custom_fields {
        // let env_key = format!("{}_{}", sanitized_name, key.to_uppercase());
        cmd.env(key.to_uppercase(), value);
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_lock(config_dir: &std::path::PathBuf) -> Result<()> {
    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, config.session_timeout_minutes);

    session.clear_session()?;
    println!("✓ Vault locked. Session cleared.");

    Ok(())
}

fn handle_status(config_dir: &std::path::PathBuf) -> Result<()> {
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

fn handle_export(
    file: &str,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
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

fn handle_import(
    file: &str,
    overwrite: bool,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
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

fn handle_totp(
    subcommand: TotpCommands,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    match subcommand {
        TotpCommands::Add { entry, secret } => {
            handle_totp_add(&entry, &secret, keyring, config_dir)
        }
        TotpCommands::Get { entry } => handle_totp_get(&entry, keyring, config_dir),
        TotpCommands::Rm { entry } => handle_totp_rm(&entry, keyring, config_dir),
    }
}

fn handle_totp_add(
    service_name: &str,
    secret: &str,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let totp_entry = ctx.vault.get_entry_mut("totp").map_err(|_| {
        anyhow::anyhow!("TOTP entry not found. Please reinitialize vault with 'hc init'")
    })?;

    if totp_entry.custom_fields.contains_key(service_name) {
        println!(
            "⚠ TOTP secret for '{}' already exists. Overwriting...",
            service_name
        );
    }

    totp_entry
        .custom_fields
        .insert(service_name.to_string(), secret.to_string());
    totp_entry.updated_at = chrono::Utc::now();

    ctx.save()?;
    println!("✓ TOTP secret for '{}' added", service_name);

    Ok(())
}

fn handle_totp_get(
    service_name: &str,
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let ctx = VaultContext::load(keyring, config_dir)?;
    let totp_entry = ctx.vault.get_entry("totp").map_err(|_| {
        anyhow::anyhow!("TOTP entry not found. Please reinitialize vault with 'hc init'")
    })?;

    if let Some(secret) = totp_entry.custom_fields.get(service_name) {
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
    keyring: &KeyringManager,
    config_dir: &std::path::PathBuf,
) -> Result<()> {
    let mut ctx = VaultContext::load(keyring, config_dir)?;

    let totp_entry = ctx.vault.get_entry_mut("totp").map_err(|_| {
        anyhow::anyhow!("TOTP entry not found. Please reinitialize vault with 'hc init'")
    })?;

    if totp_entry.custom_fields.remove(service_name).is_some() {
        totp_entry.updated_at = chrono::Utc::now();
        ctx.save()?;
        println!("✓ TOTP secret for '{}' removed", service_name);
    } else {
        println!("⚠ No TOTP secret found for service '{}'", service_name);
    }

    Ok(())
}
