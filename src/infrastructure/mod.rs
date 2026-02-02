pub mod biometric;
pub mod crypto_impl;
#[cfg(target_os = "macos")]
pub mod keychain_macos;
pub mod keyring;
pub mod provider_storage;
pub mod session;
pub mod ssh_agent;
pub mod storage;
#[cfg(target_os = "macos")]
pub mod swift_runner;
pub mod vault_registry;

pub use biometric::{get_biometric_auth, require_biometric_auth};
pub use crypto_impl::{decrypt_for_import, encrypt_for_export, CryptoServiceImpl};
pub use keyring::KeyringManager;
pub use provider_storage::{create_provider, ProviderStorage};
pub use session::{SessionData, SessionManager};
pub use ssh_agent::SshAgent;
pub use storage::VaultStorage;
pub use vault_registry::VaultRegistry;
