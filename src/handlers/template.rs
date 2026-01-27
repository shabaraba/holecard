use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::context::VaultContext;
use crate::domain::TemplateEngine;
use crate::infrastructure::KeyringManager;

pub fn handle_inject(
    entry_name: &str,
    template: &str,
    keyring: &KeyringManager,
    config_dir: &Path,
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

pub fn handle_run(
    entry_name: &str,
    command: &[String],
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let ctx = VaultContext::load(keyring, config_dir)?;
    let entry = ctx
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
