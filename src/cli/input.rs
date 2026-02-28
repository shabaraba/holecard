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

pub fn prompt_card_name() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Card name")
        .interact_text()
        .context("Failed to read card name")
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
        .with_prompt("⚠ Hand already exists. Reinitialize? This will DELETE ALL existing data!")
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
    EditField(String),
    AddField,
    DeleteField(String),
    EditNotes,
    Done,
}

pub fn prompt_edit_menu(hand: &Hand) -> Result<EditAction> {
    let mut options = vec!["Add new card", "Edit notes", "Done"];
    let mut field_keys: Vec<String> = hand.cards.keys().cloned().collect();
    field_keys.sort();

    if !field_keys.is_empty() {
        println!("\nCurrent cards: {}", field_keys.join(", "));
        options.insert(0, "Edit existing card");
        options.insert(1, "Delete card");
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to read menu selection")?;

    let offset = if field_keys.is_empty() { 0 } else { 2 };

    match selection {
        0 if !field_keys.is_empty() => {
            let field_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select field to edit")
                .items(&field_keys)
                .interact()
                .context("Failed to read field selection")?;
            Ok(EditAction::EditField(field_keys[field_selection].clone()))
        }
        1 if !field_keys.is_empty() => {
            let field_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select field to delete")
                .items(&field_keys)
                .interact()
                .context("Failed to read field selection")?;
            Ok(EditAction::DeleteField(field_keys[field_selection].clone()))
        }
        n if n == offset => Ok(EditAction::AddField),
        n if n == offset + 1 => Ok(EditAction::EditNotes),
        _ => Ok(EditAction::Done),
    }
}

pub fn prompt_field_value(key: &str) -> Result<String> {
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
            .context("Failed to read field value")
    }
}

pub fn prompt_new_field() -> Result<(String, String)> {
    let key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Field name")
        .interact_text()
        .context("Failed to read field name")?;

    let value: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} value", key))
        .allow_empty_password(true)
        .interact()
        .context("Failed to read field value")?;

    Ok((key, value))
}
