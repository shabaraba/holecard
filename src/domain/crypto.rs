use super::error::CryptoError;

pub trait CryptoService {
    fn generate_secret_key(&self) -> String;
    fn derive_key(&self, master_password: &str, secret_key: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError>;
    fn encrypt_with_key(&self, data: &[u8], derived_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt_with_key(&self, encrypted_data: &[u8], derived_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError>;
}
