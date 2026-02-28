use anyhow::{Context, Result};
use rand::RngCore;
use std::path::Path;

use crate::cli::commands::DeckCommands;
use crate::domain::CryptoService;
use crate::infrastructure::{
    CryptoServiceImpl, DeckRegistry, DeckStorage, KeyringManager, SessionManager,
};
use crate::multi_deck_context::MultiDeckContext;
use crate::{cli::input, config::Config, domain::Deck};

pub fn handle_deck(
    subcommand: DeckCommands,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    match subcommand {
        DeckCommands::List => handle_list(config_dir),
        DeckCommands::Create { name } => handle_create(name, keyring, config_dir),
        DeckCommands::Delete { name, force } => handle_delete(name, force, config_dir),
        DeckCommands::Use { name } => handle_use(name, config_dir),
        DeckCommands::Move { card, to_hand } => handle_move(card, to_hand, keyring, config_dir),
        DeckCommands::Copy { card, to_hand } => handle_copy(card, to_hand, keyring, config_dir),
        DeckCommands::Passwd => handle_passwd(deck_name, keyring, config_dir),
    }
}

fn handle_list(config_dir: &Path) -> Result<()> {
    let registry = DeckRegistry::load(config_dir)?;
    let decks = registry.list_decks()?;

    if decks.is_empty() {
        println!("No decks found. Create one with 'hc deck create <name>'");
        return Ok(());
    }

    let active_deck = registry.get_active_deck().ok().map(|v| v.name);

    println!("\nDecks:");
    for deck in decks {
        let active_indicator = if Some(&deck.name) == active_deck.as_ref() {
            " (active)"
        } else {
            ""
        };
        println!("  • {}{}", deck.name, active_indicator);
        println!("    Path: {}", deck.path.display());
        println!(
            "    Last accessed: {}",
            deck.last_accessed.format("%Y-%m-%d %H:%M:%S")
        );
    }

    Ok(())
}

fn handle_create(name: String, keyring: &KeyringManager, config_dir: &Path) -> Result<()> {
    let registry = DeckRegistry::load(config_dir)?;

    println!("========================================");
    println!("     Creating Deck: {}", name);
    println!("========================================");
    println!("\nPlease set your Master Password.");
    println!("Requirements:");
    println!("  • At least 12 characters");
    println!("  • This will be needed to access your deck");
    println!("========================================\n");

    let master_password = input::prompt_master_password_confirm()?;

    let deck_path = config_dir.join(format!("{}.enc", name));

    if deck_path.exists() {
        anyhow::bail!("Deck file already exists at: {}", deck_path.display());
    }

    let crypto = CryptoServiceImpl::new();

    let secret_key = match keyring.load_secret_key() {
        Ok(existing_key) => existing_key,
        Err(_) => {
            let new_key = crypto.generate_secret_key();
            keyring.save_secret_key(&new_key)?;
            new_key
        }
    };

    let deck = Deck::new();
    let storage = DeckStorage::new(crypto);

    let (derived_key, salt) = storage
        .derive_key_from_deck(&deck_path, &master_password, &secret_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    storage
        .save_with_cached_key(&deck, &deck_path, &derived_key, &salt)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    registry.create_deck(&name, deck_path)?;

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &name, config.session_timeout_minutes);
    session.save_session(&derived_key, &salt, Vec::new())?;

    println!("\n========================================");
    println!("     Deck '{}' Created Successfully", name);
    println!("========================================");
    println!("\n✓ Master password set");
    println!("✓ Secret key stored in system keyring");
    println!("\nIMPORTANT:");
    println!("  • Use 'hc export' regularly to backup your deck");
    println!("  • Keep your export file and password safe");
    println!("  • You need BOTH the export file and its password to restore");
    println!("========================================\n");

    Ok(())
}

fn handle_delete(name: String, force: bool, config_dir: &Path) -> Result<()> {
    let registry = DeckRegistry::load(config_dir)?;
    let deck = registry.get_deck(&name)?;

    if !force {
        println!("⚠️  About to delete deck '{}'", name);
        println!("   Path: {}", deck.path.display());
        print!("\nAre you sure? (y/N): ");
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    if deck.path.exists() {
        std::fs::remove_file(&deck.path).context(format!(
            "Failed to delete deck file: {}",
            deck.path.display()
        ))?;
    }

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &name, config.session_timeout_minutes);
    let _ = session.clear_session();

    registry.delete_deck(&name)?;

    println!("✓ Deck '{}' deleted successfully", name);

    Ok(())
}

fn handle_use(name: String, config_dir: &Path) -> Result<()> {
    let registry = DeckRegistry::load(config_dir)?;

    registry.get_deck(&name)?;

    registry.set_active(&name)?;

    println!("✓ Active deck set to '{}'", name);

    Ok(())
}

fn handle_move(
    hand_name: String,
    to_deck: String,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let mut source_ctx = MultiDeckContext::load(None, keyring, config_dir)?;
    let source_deck_name = source_ctx.deck_name.clone();

    if source_deck_name == to_deck {
        anyhow::bail!("Source and target deck are the same");
    }

    let hand = source_ctx
        .inner
        .deck
        .get_hand(&hand_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .clone();

    // Load target first to ensure it exists and is accessible
    let mut target_ctx = MultiDeckContext::load(Some(&to_deck), keyring, config_dir)?;

    // Add to target and save before removing from source (safe order)
    target_ctx
        .inner
        .deck
        .add_hand(hand.clone())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    target_ctx.save().context("Failed to save to target deck")?;

    // Only remove from source after successful target save
    source_ctx
        .inner
        .deck
        .remove_hand(&hand_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // If source save fails, attempt rollback
    if let Err(e) = source_ctx.save() {
        // Attempt to rollback: remove from target
        let _ = target_ctx.inner.deck.remove_hand(&hand_name);
        let _ = target_ctx.save();
        return Err(e).context("Failed to save source deck after move");
    }

    println!(
        "✓ Hand '{}' moved from deck '{}' to deck '{}'",
        hand_name, source_deck_name, to_deck
    );

    Ok(())
}

fn handle_copy(
    hand_name: String,
    to_deck: String,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let source_ctx = MultiDeckContext::load(None, keyring, config_dir)?;
    let source_deck_name = source_ctx.deck_name.clone();

    if source_deck_name == to_deck {
        anyhow::bail!("Source and target deck are the same");
    }

    let hand = source_ctx
        .inner
        .deck
        .get_hand(&hand_name)
        .map_err(|e| anyhow::anyhow!("{}", e))?
        .clone();

    let mut target_ctx = MultiDeckContext::load(Some(&to_deck), keyring, config_dir)?;

    target_ctx
        .inner
        .deck
        .add_hand(hand)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    target_ctx.save()?;

    println!(
        "✓ Hand '{}' copied from deck '{}' to deck '{}'",
        hand_name, source_deck_name, to_deck
    );

    Ok(())
}

fn handle_passwd(
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    println!("\n========================================");
    println!("     Change Master Password");
    println!("========================================");
    println!("\nFirst, verify your current master password:");

    let ctx = MultiDeckContext::load(deck_name, keyring, config_dir)?;
    let deck_name_str = ctx.deck_name.clone();

    let registry = DeckRegistry::load(config_dir)?;
    let deck_path = registry.get_deck(&deck_name_str)?.path.clone();

    let backup_path = deck_path.with_extension("enc.backup");
    std::fs::copy(&deck_path, &backup_path).context("Failed to create deck backup")?;

    println!("\n========================================");
    println!("     Set New Master Password");
    println!("========================================");
    println!("\nEnter new master password:");
    let new_password = input::prompt_master_password_confirm()?;

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let secret_key = keyring.load_secret_key()?;
    let crypto = CryptoServiceImpl::new();
    let derived_key = crypto
        .derive_key(&new_password, &secret_key, &salt)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Err(e) =
        ctx.inner
            .storage
            .save_with_cached_key(&ctx.inner.deck, &deck_path, &derived_key, &salt)
    {
        std::fs::rename(&backup_path, &deck_path).context("Failed to restore deck backup")?;
        anyhow::bail!(
            "Failed to re-encrypt deck: {}. Deck restored from backup.",
            e
        );
    }

    let hand_names: Vec<String> = ctx
        .inner
        .deck
        .list_hands()
        .iter()
        .map(|e| e.name().to_string())
        .collect();

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &deck_name_str, config.session_timeout_minutes);

    let clear_result = session.clear_session();
    let save_result = session.save_session(&derived_key, &salt, hand_names);
    std::fs::remove_file(&backup_path).ok();

    clear_result?;
    save_result?;

    println!("\n✓ Master password changed successfully");
    println!("✓ Session renewed");
    println!("\nNext deck access will use the new password.");

    Ok(())
}
