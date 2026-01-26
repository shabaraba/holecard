use anyhow::{Context, Result};
use chrono::Utc;

const TOTP_PERIOD: u64 = 30;
const TOTP_DIGITS: u32 = 6;

pub struct TotpService;

impl TotpService {
    /// Generate a TOTP code from a base32-encoded secret
    pub fn generate_code(secret: &str) -> Result<String> {
        Self::validate_secret(secret)?;

        let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .context("Failed to decode base32 secret")?;

        let current_time = Utc::now().timestamp() as u64;
        let counter = current_time / TOTP_PERIOD;

        let totp =
            totp_lite::totp_custom::<totp_lite::Sha1>(TOTP_PERIOD, TOTP_DIGITS, &decoded, counter);

        Ok(format!("{:0width$}", totp, width = TOTP_DIGITS as usize))
    }

    /// Get remaining seconds in current TOTP window
    pub fn get_remaining_seconds() -> u64 {
        let current_time = Utc::now().timestamp() as u64;
        TOTP_PERIOD - (current_time % TOTP_PERIOD)
    }

    /// Validate a TOTP secret
    pub fn validate_secret(secret: &str) -> Result<()> {
        let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
            .context("Invalid base32 encoding")?;

        if decoded.len() < 10 {
            anyhow::bail!("TOTP secret too short (minimum 10 bytes)");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_secret() {
        // Valid secret (RFC 4648 base32)
        assert!(TotpService::validate_secret("JBSWY3DPEHPK3PXP").is_ok());

        // Invalid base32
        assert!(TotpService::validate_secret("invalid!@#").is_err());

        // Too short
        assert!(TotpService::validate_secret("AB").is_err());
    }

    #[test]
    fn test_remaining_seconds() {
        let remaining = TotpService::get_remaining_seconds();
        assert!(remaining > 0 && remaining <= TOTP_PERIOD);
    }
}
