use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::domain::SecretResolver;
use crate::infrastructure::KeyringManager;

pub fn handle_run(
    env_vars: Vec<(String, String)>,
    command: &[String],
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    for (key, value) in env_vars {
        let resolved_value = if SecretResolver::has_uri_references(&value) {
            SecretResolver::resolve_template(&value, vault_name, keyring, config_dir)?
        } else {
            value
        };

        cmd.env(key, resolved_value);
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
