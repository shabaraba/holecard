use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::domain::TemplateEngine;
use crate::infrastructure::KeyringManager;
use crate::multi_vault_context::MultiVaultContext;

pub fn handle_inject(
    entry_name: &str,
    template: &str,
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entry = ctx
        .inner
        .vault
        .get_entry(entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let rendered = TemplateEngine::render(template, entry)?;
    println!("{}", rendered);

    Ok(())
}

pub fn handle_run(
    entry_name: &str,
    command: &[String],
    vault_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;
    let entry = ctx
        .inner
        .vault
        .get_entry(entry_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    for (key, value) in &entry.custom_fields {
        cmd.env(key.to_uppercase(), value);
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
