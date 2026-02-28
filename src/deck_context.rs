use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::cli::input;
use crate::config::Config;
use crate::domain::Deck;
use crate::infrastructure::{
    CryptoServiceImpl, DeckStorage, KeyringManager, SessionData, SessionManager,
};

pub struct DeckContext {
    pub deck: Deck,
    pub storage: DeckStorage<CryptoServiceImpl>,
    pub session_data: SessionData,
    pub config: Config,
    pub config_dir: PathBuf,
    deck_path: PathBuf,
    deck_name: String,
}

fn resolve_master_password(
    config: &Config,
    keyring: &KeyringManager,
    deck_name: &str,
) -> Result<String> {
    if !config.enable_biometric {
        return input::prompt_master_password();
    }

    let biometric = crate::infrastructure::get_biometric_auth();
    if !biometric.is_available() {
        return input::prompt_master_password();
    }

    println!("🔐 Authenticating...");
    match biometric.authenticate("Unlock your deck") {
        Ok(true) => {
            println!("✅ Authentication successful");
            match keyring.load_master_password(deck_name)? {
                Some(pwd) => {
                    println!("🔓 Unlocking deck...");
                    Ok(pwd)
                }
                None => {
                    println!("⚠️  No cached password found. Please enter your master password.");
                    let pwd = input::prompt_master_password()?;
                    keyring.save_master_password(deck_name, &pwd)?;
                    Ok(pwd)
                }
            }
        }
        Ok(false) => {
            println!("⚠️  Authentication failed. Falling back to password.");
            input::prompt_master_password()
        }
        Err(e) => {
            eprintln!("⚠️  Authentication error: {}. Falling back to password.", e);
            input::prompt_master_password()
        }
    }
}

impl DeckContext {
    pub fn load(
        deck_path: &Path,
        deck_name: &str,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<Self> {
        let secret_key = keyring.load_secret_key()?;
        let config = Config::load(config_dir)?;
        let crypto = CryptoServiceImpl::new();
        let storage = DeckStorage::new(crypto);
        let session = SessionManager::new(config_dir, deck_name, config.session_timeout_minutes);

        let (deck, session_data) = if let Some(cached) = session.load_session()? {
            let deck = storage
                .load_with_cached_key(deck_path, &cached.derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            (deck, cached)
        } else {
            let master_password = resolve_master_password(&config, keyring, deck_name)?;

            let (derived_key, salt) = storage
                .derive_key_from_deck(deck_path, &master_password, &secret_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let deck = storage
                .load_with_cached_key(deck_path, &derived_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let hand_names: Vec<String> =
                deck.list_hands().iter().map(|e| e.name.clone()).collect();

            session.save_session(&derived_key, &salt, hand_names.clone())?;
            let session_data = SessionData {
                derived_key,
                salt,
                hand_names,
            };
            (deck, session_data)
        };

        Ok(Self {
            deck,
            storage,
            session_data,
            config,
            config_dir: config_dir.to_path_buf(),
            deck_path: deck_path.to_path_buf(),
            deck_name: deck_name.to_string(),
        })
    }

    pub fn save(&self) -> Result<()> {
        self.storage
            .save_with_cached_key(
                &self.deck,
                &self.deck_path,
                &self.session_data.derived_key,
                &self.session_data.salt,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let hand_names: Vec<String> = self
            .deck
            .list_hands()
            .iter()
            .map(|e| e.name.clone())
            .collect();

        let session = SessionManager::new(
            &self.config_dir,
            &self.deck_name,
            self.config.session_timeout_minutes,
        );
        session.save_session(
            &self.session_data.derived_key,
            &self.session_data.salt,
            hand_names,
        )?;

        Ok(())
    }
}
