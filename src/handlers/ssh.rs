use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::cli::commands::SshCommands;
use crate::domain::{find_hand_by_name_or_alias, validate_private_key, Hand};
use crate::infrastructure::{KeyringManager, SshAgent};
use crate::multi_deck_context::MultiDeckContext;

pub fn handle_ssh(
    subcommand: SshCommands,
    deck_name: Option<&str>,
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
            deck_name,
            keyring,
            config_dir,
        ),
        SshCommands::Load { name, lifetime } => {
            handle_ssh_load(&name, lifetime, deck_name, keyring, config_dir)
        }
        SshCommands::Unload { name } => handle_ssh_unload(&name, deck_name, keyring, config_dir),
        SshCommands::List => handle_ssh_list(deck_name, keyring, config_dir),
        SshCommands::Connect { target, ssh_args } => {
            handle_ssh_connect(&target, ssh_args, deck_name, keyring, config_dir)
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
    deck_name: Option<&str>,
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

    let mut ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let mut cards = HashMap::new();

    if let Some(alias_value) = alias {
        // Pattern 1: Alias only (SSH config managed)
        cards.insert("alias".to_string(), alias_value);
        println!("✓ SSH hand '{}' created with alias authentication", name);
    } else {
        // Pattern 2 & 3: Direct management (username + hostname required)
        let username_value =
            username.context("Option --username is required when not using --alias")?;
        let hostname_value =
            hostname.context("Option --hostname is required when not using --alias")?;

        cards.insert("username".to_string(), username_value.clone());
        cards.insert("hostname".to_string(), hostname_value.clone());
        cards.insert(
            "host".to_string(),
            format!("{}@{}", username_value, hostname_value),
        );

        if let Some(password_value) = password {
            // Pattern 2: Password authentication
            cards.insert("password".to_string(), password_value);
            println!("✓ SSH hand '{}' created with password authentication", name);
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
            cards.insert("private_key".to_string(), private_key_content);

            if let Some(public_key_path_value) = public_key_path {
                let expanded_public_key_path = expand_tilde(&public_key_path_value)?;
                let public_key_content = std::fs::read_to_string(&expanded_public_key_path)
                    .with_context(|| {
                        format!(
                            "Failed to read public key file: {}",
                            expanded_public_key_path
                        )
                    })?;
                cards.insert("public_key".to_string(), public_key_content);
            }

            if let Some(passphrase_value) = passphrase {
                cards.insert("passphrase".to_string(), passphrase_value);
            }

            println!("✓ SSH hand '{}' created with key authentication", name);
        } else {
            anyhow::bail!("Either --password or --private-key is required when not using --alias");
        }
    }

    let hand = Hand::new(name.to_string(), cards, None);
    ctx.inner.deck.add_hand(hand)?;
    ctx.save()?;

    println!("✓ Hand '{}' saved to deck", name);

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
    hand_name: &str,
    lifetime: Option<u32>,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let hand = ctx
        .inner
        .deck
        .get_hand(hand_name)
        .map_err(|_| anyhow::anyhow!("Hand '{}' not found", hand_name))?;

    let private_key = hand
        .cards
        .get("private_key")
        .context("Hand does not contain 'private_key' card")?;

    validate_private_key(private_key)?;

    let passphrase: Option<&str> = hand.cards.get("passphrase").map(|s| s.as_str());

    let agent = SshAgent::connect()?;
    agent.add_identity(private_key, passphrase, lifetime)?;

    println!("✓ SSH key '{}' loaded into ssh-agent", hand_name);
    match lifetime {
        Some(0) => println!("  Lifetime: forever"),
        Some(sec) => println!("  Lifetime: {} seconds", sec),
        None => {}
    }

    Ok(())
}

fn handle_ssh_unload(
    identifier: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;

    let public_key = if let Ok(hand) = ctx.inner.deck.get_hand(identifier) {
        hand.cards
            .get("public_key")
            .context("Hand does not contain 'public_key' card")?
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
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let hands = ctx.inner.deck.list_hands();

    let ssh_hands: Vec<_> = hands.iter().filter(|hand| is_ssh_hand(hand)).collect();

    if ssh_hands.is_empty() {
        println!("No SSH hands found in deck");
    } else {
        println!("\nSSH Hands:\n");
        for hand in ssh_hands {
            let auth_type = get_auth_type(hand);
            let target = get_ssh_target(hand);
            println!("  {} ({})", hand.name(), auth_type);
            if let Some(t) = target {
                println!("    → {}", t);
            }
        }
    }

    Ok(())
}

fn is_ssh_hand(hand: &Hand) -> bool {
    hand.cards.contains_key("alias")
        || hand.cards.contains_key("private_key")
        || (hand.cards.contains_key("username") && hand.cards.contains_key("hostname"))
}

fn get_auth_type(hand: &Hand) -> &str {
    if hand.cards.contains_key("alias") {
        "alias"
    } else if hand.cards.contains_key("private_key") {
        "key"
    } else if hand.cards.contains_key("password") {
        "password"
    } else {
        "unknown"
    }
}

fn get_ssh_target(hand: &Hand) -> Option<String> {
    if let Some(alias) = hand.cards.get("alias") {
        Some(format!("alias: {}", alias))
    } else {
        hand.cards.get("host").cloned()
    }
}

fn handle_ssh_connect(
    target: &str,
    ssh_args: Vec<String>,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;

    let hand_name = find_hand_by_name_or_alias(&ctx.inner.deck, target)
        .ok_or_else(|| anyhow::anyhow!("No hand found with name or alias '{}'", target))?;

    let hand = ctx.inner.deck.get_hand(&hand_name)?;

    let ssh_target = if target.contains('@') {
        target.to_string()
    } else {
        // Get CSV list from host or alias card
        let csv_value = hand
            .cards
            .get("host")
            .or_else(|| hand.cards.get("alias"))
            .context("Hand has no 'host' or 'alias' card and target is not in user@host format")?;

        // Parse CSV and try to match the provided target exactly
        let aliases: Vec<String> = csv_value.split(',').map(|s| s.trim().to_string()).collect();

        // Try exact match first, otherwise use first entry
        aliases
            .iter()
            .find(|alias| *alias == target)
            .cloned()
            .or_else(|| aliases.first().cloned())
            .context("No valid alias found in CSV list")?
    };

    let has_alias = hand.cards.contains_key("alias");
    let has_private_key = hand.cards.contains_key("private_key");
    let has_password = hand.cards.contains_key("password");

    if !has_alias && !has_private_key && !has_password {
        anyhow::bail!(
            "Hand '{}' must have either 'alias', 'private_key', or 'password' card for SSH authentication",
            hand_name
        );
    }

    println!("Connecting to {}...", ssh_target);

    let status = if let Some(password) = hand.cards.get("password") {
        execute_ssh_with_password(&ssh_target, &ssh_args, password)?
    } else if let Some(private_key) = hand.cards.get("private_key") {
        validate_private_key(private_key)?;

        let passphrase = hand.cards.get("passphrase").map(|s| s.as_str());

        let agent = SshAgent::connect()?;
        agent.add_identity(private_key, passphrase, None)?;

        println!("✓ SSH key '{}' loaded into ssh-agent", hand_name);

        Command::new("ssh")
            .arg(&ssh_target)
            .args(&ssh_args)
            .status()
            .context("Failed to execute ssh command")?
    } else {
        // Alias mode: use ssh_target directly (managed by ~/.ssh/config)
        Command::new("ssh")
            .arg(&ssh_target)
            .args(&ssh_args)
            .status()
            .context("Failed to execute ssh command")?
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
