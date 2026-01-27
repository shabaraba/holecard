use crate::cli::commands::{ProviderAddCommands, ProviderCommands, ProviderSecretsCommands};
use crate::context::VaultContext;
use crate::domain::{error::ProviderError, field_to_secret_name, ProviderConfig, TemplateEngine};
use crate::infrastructure::{create_provider, CryptoServiceImpl, ProviderStorage};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::collections::HashMap;

pub fn handle_provider(ctx: &VaultContext, subcommand: &ProviderCommands) -> Result<()> {
    match subcommand {
        ProviderCommands::List => handle_list(ctx),
        ProviderCommands::Add { provider } => handle_add(ctx, provider),
        ProviderCommands::Edit {
            provider_type,
            provider_id,
            provider,
        } => handle_edit(ctx, provider_type, provider_id, provider),
        ProviderCommands::Remove {
            provider_type,
            provider_id,
        } => handle_remove(ctx, provider_type, provider_id),
        ProviderCommands::Secrets { subcommand } => handle_secrets_command(ctx, subcommand),
    }
}

fn handle_secrets_command(ctx: &VaultContext, subcommand: &ProviderSecretsCommands) -> Result<()> {
    match subcommand {
        ProviderSecretsCommands::List {
            provider_type,
            provider_id,
        } => handle_secrets_list(ctx, provider_type, provider_id),
        ProviderSecretsCommands::Add {
            provider_type,
            provider_id,
            entry_field,
            as_name,
            expand,
        } => handle_secrets_add(ctx, provider_type, provider_id, entry_field, as_name, *expand),
        ProviderSecretsCommands::Remove {
            provider_type,
            provider_id,
            secret_name,
        } => handle_secrets_remove(ctx, provider_type, provider_id, secret_name),
    }
}

fn get_provider_path(ctx: &VaultContext) -> std::path::PathBuf {
    ctx.config_dir.join("providers.enc")
}

fn load_providers(ctx: &VaultContext) -> Result<HashMap<String, ProviderConfig>> {
    let crypto = CryptoServiceImpl::new();
    let storage = ProviderStorage::new(crypto);
    let path = get_provider_path(ctx);
    storage.load(&path, &ctx.session_data.derived_key)
}

fn save_providers(ctx: &VaultContext, configs: &HashMap<String, ProviderConfig>) -> Result<()> {
    let crypto = CryptoServiceImpl::new();
    let storage = ProviderStorage::new(crypto);
    let path = get_provider_path(ctx);
    storage.save(configs, &path, &ctx.session_data.derived_key, &ctx.session_data.salt)
}

fn make_provider_key(provider_type: &str, provider_id: &str) -> String {
    format!("{}:{}", provider_type, provider_id)
}

fn handle_edit(
    ctx: &VaultContext,
    provider_type: &str,
    provider_id: &str,
    provider: &ProviderAddCommands,
) -> Result<()> {
    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    if !configs.contains_key(&key) {
        return Err(ProviderError::ProviderNotFound(key).into());
    }

    let (new_provider_type, new_provider_id, credentials) = match provider {
        ProviderAddCommands::Github {
            provider_id: _,
            repo,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert("repo".to_string(), TemplateEngine::resolve_value(repo, &ctx.vault)?);
            creds.insert("token".to_string(), TemplateEngine::resolve_value(token, &ctx.vault)?);
            ("github", provider_id, creds)
        }
        ProviderAddCommands::Cloudflare {
            provider_id: _,
            account_id,
            worker_name,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert("account_id".to_string(), TemplateEngine::resolve_value(account_id, &ctx.vault)?);
            creds.insert("worker_name".to_string(), TemplateEngine::resolve_value(worker_name, &ctx.vault)?);
            creds.insert("token".to_string(), TemplateEngine::resolve_value(token, &ctx.vault)?);
            ("cloudflare", provider_id, creds)
        }
    };

    if new_provider_type != provider_type {
        return Err(ProviderError::ConfigError(
            "Cannot change provider type. Remove and add a new provider instead.".to_string(),
        )
        .into());
    }

    let config = ProviderConfig {
        provider_type: new_provider_type.to_string(),
        provider_id: new_provider_id.to_string(),
        credentials,
    };

    configs.insert(key.clone(), config);
    save_providers(ctx, &configs)?;

    println!("✓ Provider updated: {} / {}", provider_type, provider_id);
    Ok(())
}

fn handle_add(ctx: &VaultContext, provider: &ProviderAddCommands) -> Result<()> {
    let (provider_type, provider_id, credentials) = match provider {
        ProviderAddCommands::Github {
            provider_id,
            repo,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert("repo".to_string(), TemplateEngine::resolve_value(repo, &ctx.vault)?);
            creds.insert("token".to_string(), TemplateEngine::resolve_value(token, &ctx.vault)?);
            ("github", provider_id, creds)
        }
        ProviderAddCommands::Cloudflare {
            provider_id,
            account_id,
            worker_name,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert("account_id".to_string(), TemplateEngine::resolve_value(account_id, &ctx.vault)?);
            creds.insert("worker_name".to_string(), TemplateEngine::resolve_value(worker_name, &ctx.vault)?);
            creds.insert("token".to_string(), TemplateEngine::resolve_value(token, &ctx.vault)?);
            ("cloudflare", provider_id, creds)
        }
    };

    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    if configs.contains_key(&key) {
        return Err(ProviderError::ProviderAlreadyExists(
            provider_type.to_string(),
            provider_id.to_string(),
        )
        .into());
    }

    let config = ProviderConfig {
        provider_type: provider_type.to_string(),
        provider_id: provider_id.to_string(),
        credentials,
    };

    configs.insert(key, config);
    save_providers(ctx, &configs)?;

    println!("✓ Provider added: {} / {}", provider_type, provider_id);
    Ok(())
}

fn handle_secrets_add(
    ctx: &VaultContext,
    provider_type: &str,
    provider_id: &str,
    entry_field: &str,
    as_name: &Option<String>,
    expand: bool,
) -> Result<()> {
    let configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    let config = configs
        .get(&key)
        .ok_or_else(|| ProviderError::ProviderNotFound(key.clone()))?;

    let provider = create_provider(config)?;

    let parts: Vec<&str> = entry_field.split('.').collect();
    let (entry_name, field_name) = if parts.len() == 2 {
        (parts[0], Some(parts[1]))
    } else if parts.len() == 1 {
        (parts[0], None)
    } else {
        return Err(ProviderError::InvalidFieldFormat(entry_field.to_string()).into());
    };

    let entry = ctx
        .vault
        .get_entry(entry_name)
        .map_err(|_| ProviderError::FieldNotFound(entry_name.to_string()))?;

    if expand {
        if field_name.is_some() {
            return Err(ProviderError::ConfigError(
                "Cannot use --expand with specific field".to_string(),
            )
            .into());
        }

        println!(
            "⚠️  About to push {} field(s) to {} / {}:",
            entry.custom_fields.len(),
            provider_type,
            provider_id
        );
        for (field, value) in &entry.custom_fields {
            let secret_name = field_to_secret_name(field);
            println!("   {} = {} (masked)", secret_name, mask_value(value));
        }

        if !Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()?
        {
            println!("Cancelled.");
            return Ok(());
        }

        for (field, value) in &entry.custom_fields {
            let secret_name = field_to_secret_name(field);
            provider
                .push_secret(&secret_name, value)
                .with_context(|| format!("Failed to push secret: {}", secret_name))?;
            println!("✓ Pushed: {}", secret_name);
        }
    } else {
        let field = field_name.ok_or_else(|| {
            ProviderError::ConfigError(
                "Must specify field name (e.g., entry.field) or use --expand".to_string(),
            )
        })?;

        let value = entry
            .custom_fields
            .get(field)
            .ok_or_else(|| ProviderError::FieldNotFound(field.to_string()))?;

        let secret_name = as_name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| field_to_secret_name(field));

        println!("⚠️  About to push secret to {} / {}:", provider_type, provider_id);
        println!("   Secret name: {}", secret_name);
        println!("   Value: {} (masked)", mask_value(value));

        if !Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()?
        {
            println!("Cancelled.");
            return Ok(());
        }

        provider
            .push_secret(&secret_name, value)
            .with_context(|| format!("Failed to push secret: {}", secret_name))?;
        println!("✓ Pushed: {}", secret_name);
    }

    Ok(())
}

fn handle_list(ctx: &VaultContext) -> Result<()> {
    let configs = load_providers(ctx)?;

    if configs.is_empty() {
        println!("No providers configured.");
        return Ok(());
    }

    println!("Configured providers:");
    for (_key, config) in &configs {
        println!("  {} / {}", config.provider_type, config.provider_id);
        for (cred_key, _) in &config.credentials {
            println!("    {}: ***", cred_key);
        }
    }

    Ok(())
}

fn handle_secrets_list(ctx: &VaultContext, provider_type: &str, provider_id: &str) -> Result<()> {
    let configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    let config = configs
        .get(&key)
        .ok_or_else(|| ProviderError::ProviderNotFound(key.clone()))?;

    let provider = create_provider(config)?;
    let secrets = provider.list_secrets()?;

    if secrets.is_empty() {
        println!("No secrets found in {} / {}", provider_type, provider_id);
        return Ok(());
    }

    println!("Secrets in {} / {}:", provider_type, provider_id);
    for secret in secrets {
        println!("  {}", secret);
    }

    Ok(())
}

fn handle_secrets_remove(
    ctx: &VaultContext,
    provider_type: &str,
    provider_id: &str,
    secret_name: &str,
) -> Result<()> {
    let configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    let config = configs
        .get(&key)
        .ok_or_else(|| ProviderError::ProviderNotFound(key.clone()))?;

    let provider = create_provider(config)?;

    let confirm = Confirm::new()
        .with_prompt(format!(
            "Delete secret '{}' from {} / {}?",
            secret_name, provider_type, provider_id
        ))
        .default(false)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    provider
        .delete_secret(secret_name)
        .with_context(|| format!("Failed to delete secret: {}", secret_name))?;

    println!("✓ Deleted secret: {}", secret_name);
    Ok(())
}

fn handle_remove(ctx: &VaultContext, provider_type: &str, provider_id: &str) -> Result<()> {
    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    if !configs.contains_key(&key) {
        return Err(ProviderError::ProviderNotFound(key).into());
    }

    let confirm = Confirm::new()
        .with_prompt(format!(
            "Remove provider {} / {}?",
            provider_type, provider_id
        ))
        .default(false)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    configs.remove(&key);
    save_providers(ctx, &configs)?;

    println!("✓ Provider removed: {} / {}", provider_type, provider_id);
    Ok(())
}

fn mask_value(value: &str) -> String {
    if value.len() <= 4 {
        "****".to_string()
    } else {
        format!("{}****", &value[..4])
    }
}
