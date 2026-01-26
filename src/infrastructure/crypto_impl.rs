use crate::domain::{CryptoError, CryptoService};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroize;

const NONCE_LEN: usize = 12;
const MIN_ENCRYPTED_LEN_WITH_KEY: usize = NONCE_LEN + 16;

pub struct CryptoServiceImpl;

impl CryptoServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl CryptoService for CryptoServiceImpl {
    fn generate_secret_key(&self) -> String {
        let mut bytes = [0u8; 20];
        OsRng.fill_bytes(&mut bytes);

        let encoded = base32::encode(base32::Alphabet::Crockford, &bytes);

        format!(
            "A3-{}-{}-{}-{}-{}-{}",
            &encoded[0..6],
            &encoded[6..12],
            &encoded[12..17],
            &encoded[17..22],
            &encoded[22..27],
            &encoded[27..32]
        )
    }

    fn derive_key(&self, master_password: &str, secret_key: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
        let mut combined = Vec::new();
        combined.extend_from_slice(master_password.as_bytes());
        combined.extend_from_slice(b"|");
        combined.extend_from_slice(secret_key.as_bytes());

        let mut output_key = [0u8; 32];

        let params = Params::new(19 * 1024, 2, 1, Some(32))
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(&combined, salt, &mut output_key)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

        combined.zeroize();
        Ok(output_key)
    }

    fn encrypt_with_key(&self, data: &[u8], derived_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(derived_key)
            .map_err(|e| CryptoError::CipherInitFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt_with_key(&self, encrypted_data: &[u8], derived_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        if encrypted_data.len() < MIN_ENCRYPTED_LEN_WITH_KEY {
            return Err(CryptoError::InvalidData("too short".to_string()));
        }

        let nonce_bytes = &encrypted_data[..NONCE_LEN];
        let ciphertext = &encrypted_data[NONCE_LEN..];

        let cipher = Aes256Gcm::new_from_slice(derived_key)
            .map_err(|e| CryptoError::CipherInitFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        Ok(plaintext)
    }
}

impl Default for CryptoServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}
