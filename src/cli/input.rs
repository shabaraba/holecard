use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Editor, Input, Password, Select};
use std::collections::HashMap;

use crate::domain::Hand;

pub fn prompt_master_password() -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Master Password")
        .interact()
        .context("Failed to read master password")
}

pub fn prompt_master_password_confirm() -> Result<String> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Master Password")
        .with_confirmation("Confirm Master Password", "Passwords do not match")
        .interact()
        .context("Failed to read master password")?;

    if password.len() < 12 {
        return Err(anyhow::anyhow!(
            "Master password must be at least 12 characters"
        ));
    }

    Ok(password)
}

pub fn prompt_hand_name() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Hand name")
        .interact_text()
        .context("Failed to read hand name")
}

pub fn prompt_cards() -> Result<HashMap<String, String>> {
    let mut fields = HashMap::new();

    println!("\nEnter cards (leave name empty to finish):");

    loop {
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Card name")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read card name")?;

        if key.is_empty() {
            break;
        }

        let value: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} value", key))
            .allow_empty_password(true)
            .interact()
            .context("Failed to read card value")?;

        fields.insert(key, value);
    }

    Ok(fields)
}

pub fn prompt_notes() -> Result<Option<String>> {
    let notes: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Notes (optional)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to read notes")?;

    if notes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(notes))
    }
}

#[allow(dead_code)]
pub fn prompt_confirm_reinit() -> Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("⚠ Deck already exists. Reinitialize? This will DELETE ALL existing data!")
        .default(false)
        .interact()
        .context("Failed to read confirmation")
}

pub fn prompt_export_password() -> Result<String> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Export Password")
        .with_confirmation("Confirm Export Password", "Passwords do not match")
        .interact()
        .context("Failed to read export password")?;

    if password.is_empty() {
        return Err(anyhow::anyhow!("Export password cannot be empty"));
    }

    Ok(password)
}

pub fn prompt_import_password() -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Import Password")
        .interact()
        .context("Failed to read import password")
}

pub enum EditAction {
    EditCard(String),
    AddCard,
    DeleteCard(String),
    EditNotes,
    Done,
}

pub fn prompt_edit_menu(hand: &Hand) -> Result<EditAction> {
    let mut options = vec!["Add new card", "Edit notes", "Done"];
    let mut card_keys: Vec<String> = hand.cards.keys().cloned().collect();
    card_keys.sort();

    if !card_keys.is_empty() {
        println!("\nCurrent cards: {}", card_keys.join(", "));
        options.insert(0, "Edit existing card");
        options.insert(1, "Delete card");
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to read menu selection")?;

    let offset = if card_keys.is_empty() { 0 } else { 2 };

    match selection {
        0 if !card_keys.is_empty() => {
            let card_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select card to edit")
                .items(&card_keys)
                .interact()
                .context("Failed to read card selection")?;
            Ok(EditAction::EditCard(card_keys[card_selection].clone()))
        }
        1 if !card_keys.is_empty() => {
            let card_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select card to delete")
                .items(&card_keys)
                .interact()
                .context("Failed to read card selection")?;
            Ok(EditAction::DeleteCard(card_keys[card_selection].clone()))
        }
        n if n == offset => Ok(EditAction::AddCard),
        n if n == offset + 1 => Ok(EditAction::EditNotes),
        _ => Ok(EditAction::Done),
    }
}

pub fn prompt_card_value(key: &str) -> Result<String> {
    if key == "private_key" {
        let value = Editor::new()
            .edit("# Paste your SSH private key here (including BEGIN/END lines)\n# Lines starting with # will be removed")
            .context("Failed to open editor")?
            .unwrap_or_default();

        let cleaned: String = value
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(cleaned.trim().to_string())
    } else {
        Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("New value for '{}'", key))
            .allow_empty_password(true)
            .interact()
            .context("Failed to read card value")
    }
}

pub fn prompt_new_card() -> Result<(String, String)> {
    let key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Card name")
        .interact_text()
        .context("Failed to read card name")?;

    let value: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} value", key))
        .allow_empty_password(true)
        .interact()
        .context("Failed to read card value")?;

    Ok((key, value))
}
