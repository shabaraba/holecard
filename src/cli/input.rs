use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Password};
use std::collections::HashMap;

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
        return Err(anyhow::anyhow!("Master password must be at least 12 characters"));
    }

    Ok(password)
}

pub fn prompt_entry_name() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Entry name")
        .interact_text()
        .context("Failed to read entry name")
}

pub fn prompt_custom_fields() -> Result<HashMap<String, String>> {
    let mut fields = HashMap::new();

    println!("\nEnter custom fields (leave key empty to finish):");

    loop {
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Field name")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read field name")?;

        if key.is_empty() {
            break;
        }

        let value: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("{} value", key))
            .allow_empty_password(true)
            .interact()
            .context("Failed to read field value")?;

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
