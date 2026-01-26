use super::error::CryptoError;

pub trait CryptoService {
    fn encrypt(&self, data: &[u8], master_password: &str, secret_key: &str) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, encrypted_data: &[u8], master_password: &str, secret_key: &str) -> Result<Vec<u8>, CryptoError>;
    fn generate_secret_key(&self) -> String;
}
