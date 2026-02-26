use anyhow::{Context, Result};
use chrono::Utc;

const TOTP_PERIOD: u64 = 30;
const TOTP_DIGITS: u32 = 6;

pub struct TotpService;

impl TotpService {
    /// Generate a TOTP code from a base32-encoded secret
    pub fn generate_code(secret: &str) -> Result<String> {
        let normalized = Self::normalize_secret(secret);
        Self::validate_secret(&normalized)?;

        let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &normalized)
            .context("Failed to decode base32 secret")?;

        let current_time = Utc::now().timestamp() as u64;

        let totp = totp_lite::totp_custom::<totp_lite::Sha1>(
            TOTP_PERIOD,
            TOTP_DIGITS,
            &decoded,
            current_time,
        );

        Ok(format!("{:0width$}", totp, width = TOTP_DIGITS as usize))
    }

    /// Get remaining seconds in current TOTP window
    pub fn get_remaining_seconds() -> u64 {
        let current_time = Utc::now().timestamp() as u64;
        TOTP_PERIOD - (current_time % TOTP_PERIOD)
    }

    /// Normalize TOTP secret (remove spaces, convert to uppercase)
    fn normalize_secret(secret: &str) -> String {
        secret
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect::<String>()
            .to_uppercase()
    }

    /// Validate a TOTP secret
    pub fn validate_secret(secret: &str) -> Result<()> {
        let normalized = Self::normalize_secret(secret);

        let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &normalized)
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
    fn test_normalize_secret() {
        // Spaces and dashes
        assert_eq!(
            TotpService::normalize_secret("JBSW Y3DP-EHPK 3PXP"),
            "JBSWY3DPEHPK3PXP"
        );

        // Lowercase
        assert_eq!(
            TotpService::normalize_secret("jbswy3dpehpk3pxp"),
            "JBSWY3DPEHPK3PXP"
        );

        // Mixed case with spaces
        assert_eq!(
            TotpService::normalize_secret("jBsW y3dP ehPk 3pXp"),
            "JBSWY3DPEHPK3PXP"
        );
    }

    #[test]
    fn test_validate_secret() {
        // Valid secret (RFC 4648 base32)
        assert!(TotpService::validate_secret("JBSWY3DPEHPK3PXP").is_ok());

        // Valid with spaces (should normalize)
        assert!(TotpService::validate_secret("JBSW Y3DP EHPK 3PXP").is_ok());

        // Valid lowercase (should normalize)
        assert!(TotpService::validate_secret("jbswy3dpehpk3pxp").is_ok());

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

    #[test]
    fn test_generate_code() {
        // Valid test secret from RFC 6238
        let secret = "JBSWY3DPEHPK3PXP";

        let code = TotpService::generate_code(secret).unwrap();

        // Should be 6 digits
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}
