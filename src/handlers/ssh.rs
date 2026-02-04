use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::cli::commands::SshCommands;
use crate::domain::{find_entry_by_name_or_alias, validate_private_key};
use crate::infrastructure::{KeyringManager, SshAgent};
use crate::multi_vault_context::MultiVaultContext;

pub fn handle_ssh(
    subcommand: SshCommands,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    match subcommand {
        SshCommands::Load { name, lifetime } => {
            handle_ssh_load(&name, lifetime, vault_name, keyring, config_dir)
        }
        SshCommands::Unload { name } => handle_ssh_unload(&name, vault_name, keyring, config_dir),
        SshCommands::List => handle_ssh_list(),
        SshCommands::Connect { target, ssh_args } => {
            handle_ssh_connect(&target, ssh_args, vault_name, keyring, config_dir)
        }
    }
}

fn handle_ssh_load(
    entry_name: &str,
    lifetime: Option<u32>,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entry = ctx
        .inner
        .vault
        .get_entry(entry_name)
        .map_err(|_| anyhow::anyhow!("Entry '{}' not found", entry_name))?;

    let private_key = entry
        .custom_fields
        .get("private_key")
        .context("Entry does not contain 'private_key' field")?;

    validate_private_key(private_key)?;

    let passphrase = entry.custom_fields.get("passphrase").map(|s| s.as_str());

    let agent = SshAgent::connect()?;
    agent.add_identity(private_key, passphrase, lifetime)?;

    println!("✓ SSH key '{}' loaded into ssh-agent", entry_name);
    match lifetime {
        Some(0) => println!("  Lifetime: forever"),
        Some(sec) => println!("  Lifetime: {} seconds", sec),
        None => {}
    }

    Ok(())
}

fn handle_ssh_unload(
    identifier: &str,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    let public_key = if let Ok(entry) = ctx.inner.vault.get_entry(identifier) {
        entry
            .custom_fields
            .get("public_key")
            .context("Entry does not contain 'public_key' field")?
            .clone()
    } else {
        identifier.to_string()
    };

    let agent = SshAgent::connect()?;
    agent.remove_identity(&public_key)?;

    println!("✓ SSH key removed from ssh-agent");
    Ok(())
}

fn handle_ssh_list() -> Result<()> {
    let agent = SshAgent::connect()?;
    let keys = agent.list_identities()?;

    if keys.is_empty() {
        println!("No SSH keys loaded in ssh-agent");
    } else {
        println!("\nLoaded SSH keys:\n");
        for key in keys {
            println!("  {}", key);
        }
    }

    Ok(())
}

fn handle_ssh_connect(
    target: &str,
    ssh_args: Vec<String>,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

    let entry_name = find_entry_by_name_or_alias(&ctx.inner.vault, target)
        .ok_or_else(|| anyhow::anyhow!("No entry found with name or alias '{}'", target))?;

    let entry = ctx.inner.vault.get_entry(&entry_name)?;

    let ssh_target = if target.contains('@') {
        target.to_string()
    } else {
        entry
            .custom_fields
            .get("host")
            .or_else(|| entry.custom_fields.get("alias"))
            .and_then(|value| value.split(',').next().map(|s| s.trim().to_string()))
            .context("Entry has no 'host' or 'alias' field and target is not in user@host format")?
    };

    let has_private_key = entry.custom_fields.contains_key("private_key");
    let has_password = entry.custom_fields.contains_key("password");

    if !has_private_key && !has_password {
        anyhow::bail!(
            "Entry '{}' must have either 'private_key' or 'password' field for SSH authentication",
            entry_name
        );
    }

    println!("Connecting to {}...", ssh_target);

    let status = if let Some(password) = entry.custom_fields.get("password") {
        execute_ssh_with_password(&ssh_target, &ssh_args, password)?
    } else if let Some(private_key) = entry.custom_fields.get("private_key") {
        validate_private_key(private_key)?;

        let passphrase = entry.custom_fields.get("passphrase").map(|s| s.as_str());

        let agent = SshAgent::connect()?;
        agent.add_identity(private_key, passphrase, None)?;

        println!("✓ SSH key '{}' loaded into ssh-agent", entry_name);

        Command::new("ssh")
            .arg(&ssh_target)
            .args(&ssh_args)
            .status()
            .context("Failed to execute ssh command")?
    } else {
        unreachable!("Either private_key or password should exist")
    };

    if !status.success() {
        anyhow::bail!("SSH connection failed");
    }

    Ok(())
}

fn execute_ssh_with_password(
    ssh_target: &str,
    ssh_args: &[String],
    password: &str,
) -> Result<std::process::ExitStatus> {
    if !is_sshpass_available() {
        anyhow::bail!(
            "Password authentication requires 'sshpass' to be installed.\n\
             Install it with: brew install sshpass (macOS) or apt-get install sshpass (Linux)"
        );
    }

    println!("✓ Using password authentication");

    Command::new("sshpass")
        .arg("-p")
        .arg(password)
        .arg("ssh")
        .arg(ssh_target)
        .args(ssh_args)
        .status()
        .context("Failed to execute sshpass command")
}

fn is_sshpass_available() -> bool {
    Command::new("which")
        .arg("sshpass")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
