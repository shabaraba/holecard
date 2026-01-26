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
