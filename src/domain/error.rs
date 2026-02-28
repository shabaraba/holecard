use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeckError {
    #[error("Hand '{0}' already exists")]
    HandAlreadyExists(String),

    #[error("Hand '{0}' not found")]
    HandNotFound(String),
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

    #[error("Hand '{0}' not found in deck")]
    CardNotFound(String),

    #[error("Invalid hand name format: {0}")]
    InvalidCardFormat(String),
}
