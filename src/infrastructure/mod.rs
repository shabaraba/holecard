pub mod biometric;
pub mod crypto_impl;
pub mod deck_registry;
#[cfg(target_os = "macos")]
pub mod keychain_macos;
pub mod keyring;
pub mod provider_storage;
pub mod session;
pub mod ssh_agent;
pub mod storage;
#[cfg(target_os = "macos")]
pub mod swift_runner;

pub use biometric::{get_biometric_auth, require_biometric_auth};
pub use crypto_impl::{decrypt_for_import, encrypt_for_export, CryptoServiceImpl};
pub use deck_registry::DeckRegistry;
pub use keyring::KeyringManager;
pub use provider_storage::{create_provider, ProviderStorage};
pub use session::{SessionData, SessionManager};
pub use ssh_agent::SshAgent;
pub use storage::DeckStorage;
