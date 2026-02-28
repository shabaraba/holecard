use crate::cli::commands::{ProviderAddCommands, ProviderCommands, ProviderSecretsCommands};
use crate::deck_context::DeckContext;
use crate::domain::{
    card_to_secret_name, error::ProviderError, Deck, ProviderConfig, TemplateEngine,
};
use crate::infrastructure::{create_provider, CryptoServiceImpl, ProviderStorage};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::collections::HashMap;

struct ExtractedCredentials {
    provider_type: &'static str,
    provider_id: String,
    credentials: HashMap<String, String>,
}

fn extract_credentials(
    provider: &ProviderAddCommands,
    deck: &Deck,
) -> Result<ExtractedCredentials> {
    match provider {
        ProviderAddCommands::Github {
            provider_id,
            repo,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert(
                "repo".to_string(),
                TemplateEngine::resolve_value(repo, deck)?,
            );
            creds.insert(
                "token".to_string(),
                TemplateEngine::resolve_value(token, deck)?,
            );
            Ok(ExtractedCredentials {
                provider_type: "github",
                provider_id: provider_id.clone(),
                credentials: creds,
            })
        }
        ProviderAddCommands::Cloudflare {
            provider_id,
            account_id,
            worker_name,
            token,
        } => {
            let mut creds = HashMap::new();
            creds.insert(
                "account_id".to_string(),
                TemplateEngine::resolve_value(account_id, deck)?,
            );
            creds.insert(
                "worker_name".to_string(),
                TemplateEngine::resolve_value(worker_name, deck)?,
            );
            creds.insert(
                "token".to_string(),
                TemplateEngine::resolve_value(token, deck)?,
            );
            Ok(ExtractedCredentials {
                provider_type: "cloudflare",
                provider_id: provider_id.clone(),
                credentials: creds,
            })
        }
    }
}

pub fn handle_provider(ctx: &DeckContext, subcommand: &ProviderCommands) -> Result<()> {
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

fn handle_secrets_command(ctx: &DeckContext, subcommand: &ProviderSecretsCommands) -> Result<()> {
    match subcommand {
        ProviderSecretsCommands::List {
            provider_type,
            provider_id,
        } => handle_secrets_list(ctx, provider_type, provider_id),
        ProviderSecretsCommands::Add {
            provider_type,
            provider_id,
            card_field,
            as_name,
            expand,
        } => handle_secrets_add(
            ctx,
            provider_type,
            provider_id,
            card_field,
            as_name,
            *expand,
        ),
        ProviderSecretsCommands::Remove {
            provider_type,
            provider_id,
            secret_name,
        } => handle_secrets_remove(ctx, provider_type, provider_id, secret_name),
    }
}

fn create_storage() -> ProviderStorage<CryptoServiceImpl> {
    ProviderStorage::new(CryptoServiceImpl::new())
}

fn get_provider_path(ctx: &DeckContext) -> std::path::PathBuf {
    ctx.config_dir.join("providers.enc")
}

fn load_providers(ctx: &DeckContext) -> Result<HashMap<String, ProviderConfig>> {
    let storage = create_storage();
    storage.load(&get_provider_path(ctx), &ctx.session_data.derived_key)
}

fn save_providers(ctx: &DeckContext, configs: &HashMap<String, ProviderConfig>) -> Result<()> {
    let storage = create_storage();
    storage.save(
        configs,
        &get_provider_path(ctx),
        &ctx.session_data.derived_key,
        &ctx.session_data.salt,
    )
}

fn make_provider_key(provider_type: &str, provider_id: &str) -> String {
    format!("{}:{}", provider_type, provider_id)
}

fn get_provider_config<'a>(
    configs: &'a HashMap<String, ProviderConfig>,
    provider_type: &str,
    provider_id: &str,
) -> Result<&'a ProviderConfig> {
    let key = make_provider_key(provider_type, provider_id);
    configs
        .get(&key)
        .ok_or_else(|| ProviderError::ProviderNotFound(key).into())
}

fn confirm_action(prompt: &str) -> Result<bool> {
    Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .context("Failed to read confirmation")
}

fn handle_edit(
    ctx: &DeckContext,
    provider_type: &str,
    provider_id: &str,
    provider: &ProviderAddCommands,
) -> Result<()> {
    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    if !configs.contains_key(&key) {
        return Err(ProviderError::ProviderNotFound(key).into());
    }

    let extracted = extract_credentials(provider, &ctx.deck)?;

    if extracted.provider_type != provider_type {
        return Err(ProviderError::ConfigError(
            "Cannot change provider type. Remove and add a new provider instead.".to_string(),
        )
        .into());
    }

    let config = ProviderConfig {
        provider_type: extracted.provider_type.to_string(),
        provider_id: provider_id.to_string(),
        credentials: extracted.credentials,
    };

    configs.insert(key.clone(), config);
    save_providers(ctx, &configs)?;

    println!("✓ Provider updated: {} / {}", provider_type, provider_id);
    Ok(())
}

fn handle_add(ctx: &DeckContext, provider: &ProviderAddCommands) -> Result<()> {
    let extracted = extract_credentials(provider, &ctx.deck)?;
    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(extracted.provider_type, &extracted.provider_id);

    if configs.contains_key(&key) {
        return Err(ProviderError::ProviderAlreadyExists(
            extracted.provider_type.to_string(),
            extracted.provider_id.clone(),
        )
        .into());
    }

    let config = ProviderConfig {
        provider_type: extracted.provider_type.to_string(),
        provider_id: extracted.provider_id.clone(),
        credentials: extracted.credentials,
    };

    configs.insert(key, config);
    save_providers(ctx, &configs)?;

    println!(
        "✓ Provider added: {} / {}",
        extracted.provider_type, extracted.provider_id
    );
    Ok(())
}

fn handle_secrets_add(
    ctx: &DeckContext,
    provider_type: &str,
    provider_id: &str,
    card_field: &str,
    as_name: &Option<String>,
    expand: bool,
) -> Result<()> {
    let configs = load_providers(ctx)?;
    let config = get_provider_config(&configs, provider_type, provider_id)?;
    let provider = create_provider(config)?;

    let parts: Vec<&str> = card_field.split('.').collect();
    let (hand_name, card_name) = if parts.len() == 2 {
        (parts[0], Some(parts[1]))
    } else if parts.len() == 1 {
        (parts[0], None)
    } else {
        return Err(ProviderError::InvalidCardFormat(card_field.to_string()).into());
    };

    let hand = ctx
        .deck
        .get_hand(hand_name)
        .map_err(|_| ProviderError::CardNotFound(hand_name.to_string()))?;

    if expand {
        if card_name.is_some() {
            return Err(ProviderError::ConfigError(
                "Cannot use --expand with specific card".to_string(),
            )
            .into());
        }

        println!(
            "About to push {} card(s) to {} / {}:",
            hand.cards.len(),
            provider_type,
            provider_id
        );
        for (card_key, value) in &hand.cards {
            let secret_name = card_to_secret_name(card_key);
            println!("   {} = {} (masked)", secret_name, mask_value(value));
        }

        if !confirm_action("Continue?")? {
            println!("Cancelled.");
            return Ok(());
        }

        for (card_key, value) in &hand.cards {
            let secret_name = card_to_secret_name(card_key);
            provider
                .push_secret(&secret_name, value)
                .with_context(|| format!("Failed to push secret: {}", secret_name))?;
            println!("Pushed: {}", secret_name);
        }
    } else {
        let card = card_name.ok_or_else(|| {
            ProviderError::ConfigError(
                "Must specify card name (e.g., hand.card) or use --expand".to_string(),
            )
        })?;

        let value = hand.cards.get(card).ok_or_else(|| {
            ProviderError::ConfigError(format!("Card '{}' not found in hand '{}'", card, hand_name))
        })?;

        let secret_name = as_name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| card_to_secret_name(card));

        println!(
            "About to push secret to {} / {}:",
            provider_type, provider_id
        );
        println!("   Secret name: {}", secret_name);
        println!("   Value: {} (masked)", mask_value(value));

        if !confirm_action("Continue?")? {
            println!("Cancelled.");
            return Ok(());
        }

        provider
            .push_secret(&secret_name, value)
            .with_context(|| format!("Failed to push secret: {}", secret_name))?;
        println!("Pushed: {}", secret_name);
    }

    Ok(())
}

fn handle_list(ctx: &DeckContext) -> Result<()> {
    let configs = load_providers(ctx)?;

    if configs.is_empty() {
        println!("No providers configured.");
        return Ok(());
    }

    println!("Configured providers:");
    for config in configs.values() {
        println!("  {} / {}", config.provider_type, config.provider_id);
        for cred_key in config.credentials.keys() {
            println!("    {}: ***", cred_key);
        }
    }

    Ok(())
}

fn handle_secrets_list(ctx: &DeckContext, provider_type: &str, provider_id: &str) -> Result<()> {
    let configs = load_providers(ctx)?;
    let config = get_provider_config(&configs, provider_type, provider_id)?;
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
    ctx: &DeckContext,
    provider_type: &str,
    provider_id: &str,
    secret_name: &str,
) -> Result<()> {
    let configs = load_providers(ctx)?;
    let config = get_provider_config(&configs, provider_type, provider_id)?;
    let provider = create_provider(config)?;

    let prompt = format!(
        "Delete secret '{}' from {} / {}?",
        secret_name, provider_type, provider_id
    );
    if !confirm_action(&prompt)? {
        println!("Cancelled.");
        return Ok(());
    }

    provider
        .delete_secret(secret_name)
        .with_context(|| format!("Failed to delete secret: {}", secret_name))?;

    println!("✓ Deleted secret: {}", secret_name);
    Ok(())
}

fn handle_remove(ctx: &DeckContext, provider_type: &str, provider_id: &str) -> Result<()> {
    let mut configs = load_providers(ctx)?;
    let key = make_provider_key(provider_type, provider_id);

    if !configs.contains_key(&key) {
        return Err(ProviderError::ProviderNotFound(key).into());
    }

    let prompt = format!("Remove provider {} / {}?", provider_type, provider_id);
    if !confirm_action(&prompt)? {
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
