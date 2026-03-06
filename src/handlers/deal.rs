use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::domain::SecretResolver;
use crate::infrastructure::KeyringManager;
use crate::multi_deck_context::MultiDeckContext;

pub struct DealOptions {
    pub uppercase: bool,
    pub prefix: Option<String>,
    pub additional_env: Vec<(String, String)>,
}

fn validate_env_key(key: &str) -> Result<()> {
    if key.is_empty() {
        anyhow::bail!("Environment variable key cannot be empty");
    }
    if key.contains('\0') {
        anyhow::bail!("Environment variable key cannot contain NUL byte: {}", key);
    }
    if key.contains('=') {
        anyhow::bail!("Environment variable key cannot contain '=': {}", key);
    }
    #[cfg(target_os = "windows")]
    if key.chars().any(|c| c.is_control() || c == '"') {
        anyhow::bail!(
            "Environment variable key contains invalid Windows characters: {}",
            key
        );
    }
    Ok(())
}

fn build_env_key(key: &str, options: &DealOptions) -> String {
    let base = if options.uppercase {
        key.to_uppercase()
    } else {
        key.to_string()
    };

    match &options.prefix {
        Some(prefix) => format!("{}{}", prefix, base),
        None => base,
    }
}

fn resolve_value(
    value: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<String> {
    if SecretResolver::has_uri_references(value) {
        SecretResolver::resolve_template(value, deck_name, keyring, config_dir)
    } else {
        Ok(value.to_string())
    }
}

pub fn handle_deal(
    hand_name: &str,
    options: DealOptions,
    command: &[String],
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let hand = ctx
        .inner
        .deck
        .get_hand(hand_name)
        .map_err(|_| anyhow::anyhow!("Hand '{}' not found", hand_name))?;

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    for (key, value) in &hand.cards {
        let env_key = build_env_key(key, &options);
        validate_env_key(&env_key)?;
        cmd.env(env_key, value);
    }

    for (key, value) in options.additional_env {
        validate_env_key(&key)?;
        let resolved = resolve_value(&value, deck_name, keyring, config_dir)?;
        cmd.env(key, resolved);
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
