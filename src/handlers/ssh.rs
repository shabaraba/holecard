use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::cli::commands::SshCommands;
use crate::domain::{find_entry_by_name_or_alias, validate_private_key, Entry};
use crate::infrastructure::{KeyringManager, SshAgent};
use crate::multi_vault_context::MultiVaultContext;

pub fn handle_ssh(
    subcommand: SshCommands,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    match subcommand {
        SshCommands::Add {
            name,
            alias,
            username,
            hostname,
            password,
            private_key,
            public_key,
            passphrase,
        } => handle_ssh_add(
            &name,
            SshAddOptions {
                alias,
                username,
                hostname,
                password,
                private_key_path: private_key,
                public_key_path: public_key,
                passphrase,
            },
            vault_name,
            keyring,
            config_dir,
        ),
        SshCommands::Load { name, lifetime } => {
            handle_ssh_load(&name, lifetime, vault_name, keyring, config_dir)
        }
        SshCommands::Unload { name } => handle_ssh_unload(&name, vault_name, keyring, config_dir),
        SshCommands::List => handle_ssh_list(vault_name, keyring, config_dir),
        SshCommands::Connect { target, ssh_args } => {
            handle_ssh_connect(&target, ssh_args, vault_name, keyring, config_dir)
        }
    }
}

struct SshAddOptions {
    alias: Option<String>,
    username: Option<String>,
    hostname: Option<String>,
    password: Option<String>,
    private_key_path: Option<String>,
    public_key_path: Option<String>,
    passphrase: Option<String>,
}

fn handle_ssh_add(
    name: &str,
    options: SshAddOptions,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let SshAddOptions {
        alias,
        username,
        hostname,
        password,
        private_key_path,
        public_key_path,
        passphrase,
    } = options;
    // Validation: --alias is mutually exclusive with other options
    if alias.is_some()
        && (username.is_some()
            || hostname.is_some()
            || password.is_some()
            || private_key_path.is_some())
    {
        anyhow::bail!(
            "Option --alias cannot be used with --username, --hostname, --password, or --private-key.\n\
             When using --alias, the SSH connection details are managed by ~/.ssh/config."
        );
    }

    // Validation: --password and --private-key are mutually exclusive
    if password.is_some() && private_key_path.is_some() {
        anyhow::bail!(
            "Options --password and --private-key are mutually exclusive.\n\
             Use --password for password authentication or --private-key for key-based authentication."
        );
    }

    let mut ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let mut custom_fields = HashMap::new();

    if let Some(alias_value) = alias {
        // Pattern 1: Alias only (SSH config managed)
        custom_fields.insert("alias".to_string(), alias_value);
        println!("✓ SSH entry '{}' created with alias authentication", name);
    } else {
        // Pattern 2 & 3: Direct management (username + hostname required)
        let username_value =
            username.context("Option --username is required when not using --alias")?;
        let hostname_value =
            hostname.context("Option --hostname is required when not using --alias")?;

        custom_fields.insert("username".to_string(), username_value.clone());
        custom_fields.insert("hostname".to_string(), hostname_value.clone());
        custom_fields.insert(
            "host".to_string(),
            format!("{}@{}", username_value, hostname_value),
        );

        if let Some(password_value) = password {
            // Pattern 2: Password authentication
            custom_fields.insert("password".to_string(), password_value);
            println!(
                "✓ SSH entry '{}' created with password authentication",
                name
            );
        } else if let Some(private_key_path_value) = private_key_path {
            // Pattern 3: Key authentication
            let expanded_private_key_path = expand_tilde(&private_key_path_value)?;
            let private_key_content = std::fs::read_to_string(&expanded_private_key_path)
                .with_context(|| {
                    format!(
                        "Failed to read private key file: {}",
                        expanded_private_key_path
                    )
                })?;

            validate_private_key(&private_key_content)?;
            custom_fields.insert("private_key".to_string(), private_key_content);

            if let Some(public_key_path_value) = public_key_path {
                let expanded_public_key_path = expand_tilde(&public_key_path_value)?;
                let public_key_content = std::fs::read_to_string(&expanded_public_key_path)
                    .with_context(|| {
                        format!(
                            "Failed to read public key file: {}",
                            expanded_public_key_path
                        )
                    })?;
                custom_fields.insert("public_key".to_string(), public_key_content);
            }

            if let Some(passphrase_value) = passphrase {
                custom_fields.insert("passphrase".to_string(), passphrase_value);
            }

            println!("✓ SSH entry '{}' created with key authentication", name);
        } else {
            anyhow::bail!("Either --password or --private-key is required when not using --alias");
        }
    }

    let entry = Entry::new(name.to_string(), custom_fields, None);
    ctx.inner.vault.add_entry(entry)?;
    ctx.save()?;

    println!("✓ Entry '{}' saved to vault", name);

    Ok(())
}

fn expand_tilde(path: &str) -> Result<String> {
    if path.starts_with('~') {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(path.replacen('~', &home, 1))
    } else {
        Ok(path.to_string())
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

fn handle_ssh_list(
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entries = ctx.inner.vault.list_entries();

    let ssh_entries: Vec<_> = entries.iter().filter(|entry| is_ssh_entry(entry)).collect();

    if ssh_entries.is_empty() {
        println!("No SSH entries found in vault");
    } else {
        println!("\nSSH Entries:\n");
        for entry in ssh_entries {
            let auth_type = get_auth_type(entry);
            let target = get_ssh_target(entry);
            println!("  {} ({})", entry.name, auth_type);
            if let Some(t) = target {
                println!("    → {}", t);
            }
        }
    }

    Ok(())
}

fn is_ssh_entry(entry: &Entry) -> bool {
    entry.custom_fields.contains_key("alias")
        || entry.custom_fields.contains_key("private_key")
        || (entry.custom_fields.contains_key("username")
            && entry.custom_fields.contains_key("hostname"))
}

fn get_auth_type(entry: &Entry) -> &str {
    if entry.custom_fields.contains_key("alias") {
        "alias"
    } else if entry.custom_fields.contains_key("private_key") {
        "key"
    } else if entry.custom_fields.contains_key("password") {
        "password"
    } else {
        "unknown"
    }
}

fn get_ssh_target(entry: &Entry) -> Option<String> {
    if let Some(alias) = entry.custom_fields.get("alias") {
        Some(format!("alias: {}", alias))
    } else {
        entry.custom_fields.get("host").cloned()
    }
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
