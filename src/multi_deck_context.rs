use anyhow::Result;
use std::path::Path;

use crate::deck_context::DeckContext;
use crate::infrastructure::{DeckRegistry, KeyringManager};

pub struct MultiDeckContext {
    pub deck_name: String,
    pub inner: DeckContext,
}

impl MultiDeckContext {
    pub fn load(
        deck_name: Option<&str>,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<Self> {
        let registry = DeckRegistry::load(config_dir)?;

        let deck_metadata = if let Some(name) = deck_name {
            registry.get_deck(name)?
        } else {
            registry.get_active_deck()?
        };

        let deck_path = &deck_metadata.path;
        let inner = DeckContext::load(deck_path, &deck_metadata.name, keyring, config_dir)?;

        registry.touch_deck(&deck_metadata.name)?;

        Ok(Self {
            deck_name: deck_metadata.name,
            inner,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.inner.save()
    }
}
