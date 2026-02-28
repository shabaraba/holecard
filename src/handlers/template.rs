use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::domain::TemplateEngine;
use crate::infrastructure::KeyringManager;
use crate::multi_deck_context::MultiDeckContext;

pub fn handle_inject(
    card_name: &str,
    template: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let hand = ctx
        .inner
        .deck
        .get_hand(card_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let rendered = TemplateEngine::render(template, hand)?;
    println!("{}", rendered);

    Ok(())
}

pub fn handle_run(
    card_name: &str,
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
        .get_hand(card_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    for (key, value) in &hand.cards {
        cmd.env(key.to_uppercase(), value);
    }

    let status = cmd.status().context("Failed to execute command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
