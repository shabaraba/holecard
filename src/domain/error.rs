use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Entry '{0}' already exists")]
    EntryAlreadyExists(String),

    #[error("Entry '{0}' not found")]
    EntryNotFound(String),
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: incorrect password or corrupted data")]
    DecryptionFailed,

    #[error("Cipher initialization failed: {0}")]
    CipherInitFailed(String),

    #[error("Invalid encrypted data: {0}")]
    InvalidData(String),
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider '{0}' not found")]
    ProviderNotFound(String),

    #[error("Provider '{0}' with ID '{1}' already exists")]
    ProviderAlreadyExists(String, String),

    #[error("Provider configuration error: {0}")]
    ConfigError(String),

    #[error("Field '{0}' not found in entry")]
    FieldNotFound(String),

    #[error("Invalid field name format: {0}")]
    InvalidFieldFormat(String),
}
