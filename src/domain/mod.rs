pub mod crypto;
pub mod entry;
pub mod error;
pub mod password_gen;
pub mod provider;
pub mod providers;
pub mod secret_resolver;
pub mod ssh_key;
pub mod template;
pub mod totp;
pub mod uri;
pub mod vault;

pub use crypto::CryptoService;
pub use entry::Entry;
pub use error::CryptoError;
#[allow(unused_imports)]
pub use error::VaultError;
pub use password_gen::PasswordService;
pub use provider::{field_to_secret_name, ProviderConfig};
pub use secret_resolver::SecretResolver;
pub use ssh_key::{find_entry_by_name_or_alias, validate_private_key};
pub use template::TemplateEngine;
pub use totp::TotpService;
pub use uri::SecretUri;
pub use vault::Vault;
