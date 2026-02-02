use anyhow::{Context, Result};
use rand::RngCore;
use std::path::Path;

use crate::cli::commands::VaultCommands;
use crate::domain::CryptoService;
use crate::infrastructure::{
    CryptoServiceImpl, KeyringManager, SessionManager, VaultRegistry, VaultStorage,
};
use crate::multi_vault_context::MultiVaultContext;
use crate::{cli::input, config::Config, domain::Vault};

pub fn handle_vault(
    subcommand: VaultCommands,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    match subcommand {
        VaultCommands::List => handle_list(config_dir),
        VaultCommands::Create { name } => handle_create(name, keyring, config_dir),
        VaultCommands::Delete { name, force } => handle_delete(name, force, config_dir),
        VaultCommands::Use { name } => handle_use(name, config_dir),
        VaultCommands::Move { entry, to_vault } => {
            handle_move(entry, to_vault, keyring, config_dir)
        }
        VaultCommands::Copy { entry, to_vault } => {
            handle_copy(entry, to_vault, keyring, config_dir)
        }
        VaultCommands::Passwd => handle_passwd(vault_name, keyring, config_dir),
    }
}

fn handle_list(config_dir: &Path) -> Result<()> {
    let registry = VaultRegistry::load(config_dir)?;
    let vaults = registry.list_vaults()?;

    if vaults.is_empty() {
        println!("No vaults found. Create one with 'hc vault create <name>'");
        return Ok(());
    }

    let active_vault = registry.get_active_vault().ok().map(|v| v.name);

    println!("\nVaults:");
    for vault in vaults {
        let active_indicator = if Some(&vault.name) == active_vault.as_ref() {
            " (active)"
        } else {
            ""
        };
        println!("  • {}{}", vault.name, active_indicator);
        println!("    Path: {}", vault.path.display());
        println!(
            "    Last accessed: {}",
            vault.last_accessed.format("%Y-%m-%d %H:%M:%S")
        );
    }

    Ok(())
}

fn handle_create(name: String, keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let registry = VaultRegistry::load(config_dir)?;

    println!("========================================");
    println!("     Creating Vault: {}", name);
    println!("========================================");
    println!("\nPlease set your Master Password.");
    println!("Requirements:");
    println!("  • At least 12 characters");
    println!("  • This will be needed to access your vault");
    println!("========================================\n");

    let master_password = input::prompt_master_password_confirm()?;

    let vault_path = config_dir.join(format!("{}.enc", name));

    if vault_path.exists() {
        anyhow::bail!("Vault file already exists at: {}", vault_path.display());
    }

    let crypto = CryptoServiceImpl::new();

    let secret_key = match keyring.load_secret_key() {
        Ok(existing_key) => existing_key,
        Err(_) => {
            let new_key = crypto.generate_secret_key();
            keyring.save_secret_key(&new_key)?;
            new_key
        }
    };

    let vault = Vault::new();
    let storage = VaultStorage::new(crypto);

    let (derived_key, salt) = storage
        .derive_key_from_vault(&vault_path, &master_password, &secret_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    storage
        .save_with_cached_key(&vault, &vault_path, &derived_key, &salt)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    registry.create_vault(&name, vault_path)?;

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &name, config.session_timeout_minutes);
    session.save_session(&derived_key, &salt, Vec::new())?;

    println!("\n========================================");
    println!("     Vault '{}' Created Successfully", name);
    println!("========================================");
    println!("\n✓ Master password set");
    println!("✓ Secret key stored in system keyring");
    println!("\nIMPORTANT:");
    println!("  • Use 'hc export' regularly to backup your vault");
    println!("  • Keep your export file and password safe");
    println!("  • You need BOTH the export file and its password to restore");
    println!("========================================\n");

    Ok(())
}

fn handle_delete(name: String, force: bool, config_dir: &Path) -> Result<()> {
    let registry = VaultRegistry::load(config_dir)?;
    let vault = registry.get_vault(&name)?;

    if !force {
        println!("⚠️  About to delete vault '{}'", name);
        println!("   Path: {}", vault.path.display());
        print!("\nAre you sure? (y/N): ");
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    if vault.path.exists() {
        std::fs::remove_file(&vault.path).context(format!(
            "Failed to delete vault file: {}",
            vault.path.display()
        ))?;
    }

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &name, config.session_timeout_minutes);
    let _ = session.clear_session();

    registry.delete_vault(&name)?;

    println!("✓ Vault '{}' deleted successfully", name);

    Ok(())
}

fn handle_use(name: String, config_dir: &Path) -> Result<()> {
    let registry = VaultRegistry::load(config_dir)?;

    registry.get_vault(&name)?;

    registry.set_active(&name)?;

    println!("✓ Active vault set to '{}'", name);

    Ok(())
}

fn handle_move(
    entry_name: String,
    to_vault: String,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut source_ctx = MultiVaultContext::load(None, keyring, config_dir)?;
    let source_vault_name = source_ctx.vault_name.clone();

    if source_vault_name == to_vault {
        anyhow::bail!("Source and target vault are the same");
    }

    let entry = source_ctx
        .inner
        .vault
        .get_entry(&entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .clone();

    source_ctx
        .inner
        .vault
        .remove_entry(&entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    source_ctx.save()?;

    let mut target_ctx = MultiVaultContext::load(Some(&to_vault), keyring, config_dir)?;

    target_ctx
        .inner
        .vault
        .add_entry(entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    target_ctx.save()?;

    println!(
        "✓ Entry '{}' moved from '{}' to '{}'",
        entry_name, source_vault_name, to_vault
    );

    Ok(())
}

fn handle_copy(
    entry_name: String,
    to_vault: String,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let source_ctx = MultiVaultContext::load(None, keyring, config_dir)?;
    let source_vault_name = source_ctx.vault_name.clone();

    if source_vault_name == to_vault {
        anyhow::bail!("Source and target vault are the same");
    }

    let entry = source_ctx
        .inner
        .vault
        .get_entry(&entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .clone();

    let mut target_ctx = MultiVaultContext::load(Some(&to_vault), keyring, config_dir)?;

    target_ctx
        .inner
        .vault
        .add_entry(entry)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    target_ctx.save()?;

    println!(
        "✓ Entry '{}' copied from '{}' to '{}'",
        entry_name, source_vault_name, to_vault
    );

    Ok(())
}

fn handle_passwd(
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    println!("\n========================================");
    println!("     Change Master Password");
    println!("========================================");
    println!("\nFirst, verify your current master password:");

    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let vault_name = ctx.vault_name.clone();

    let registry = VaultRegistry::load(config_dir)?;
    let vault_path = registry.get_vault(&vault_name)?.path.clone();

    let backup_path = vault_path.with_extension("enc.backup");
    std::fs::copy(&vault_path, &backup_path).context("Failed to create vault backup")?;

    println!("\n========================================");
    println!("     Set New Master Password");
    println!("========================================");
    println!("\nEnter new master password:");
    let new_password = input::prompt_master_password_confirm()?;

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let secret_key = keyring.load_secret_key()?;
    let crypto = CryptoServiceImpl::new();
    let derived_key = crypto
        .derive_key(&new_password, &secret_key, &salt)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Err(e) =
        ctx.inner
            .storage
            .save_with_cached_key(&ctx.inner.vault, &vault_path, &derived_key, &salt)
    {
        std::fs::rename(&backup_path, &vault_path).context("Failed to restore vault backup")?;
        anyhow::bail!(
            "Failed to re-encrypt vault: {}. Vault restored from backup.",
            e
        );
    }

    let entry_names: Vec<String> = ctx
        .inner
        .vault
        .list_entries()
        .iter()
        .map(|e| e.name.clone())
        .collect();

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &vault_name, config.session_timeout_minutes);

    let clear_result = session.clear_session();
    let save_result = session.save_session(&derived_key, &salt, entry_names);
    std::fs::remove_file(&backup_path).ok();

    clear_result?;
    save_result?;

    println!("\n✓ Master password changed successfully");
    println!("✓ Session renewed");
    println!("\nNext vault access will use the new password.");

    Ok(())
}
