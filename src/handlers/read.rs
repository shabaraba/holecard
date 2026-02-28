use anyhow::Result;
use std::path::Path;

use crate::domain::SecretResolver;
use crate::infrastructure::KeyringManager;

pub fn handle_read(
    uri: &str,
    deck_name: Option<&str>,
    keyring: &KeyringManager,
    config_dir: &Path,
) -> Result<()> {
    let value = SecretResolver::resolve(uri, deck_name, keyring, config_dir)?;
    println!("{}", value);
    Ok(())
}
