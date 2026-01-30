use anyhow::{bail, Result};

const KEY_FORMATS: &[(&str, SshKeyType)] = &[
    ("OPENSSH PRIVATE KEY", SshKeyType::OpenSsh),
    ("RSA PRIVATE KEY", SshKeyType::Rsa),
    ("EC PRIVATE KEY", SshKeyType::Ecdsa),
    ("PRIVATE KEY", SshKeyType::Ed25519Pem),
];

pub fn validate_private_key(key: &str) -> Result<SshKeyType> {
    let key = key.trim();

    for (label, key_type) in KEY_FORMATS {
        let begin = format!("-----BEGIN {}-----", label);
        let end = format!("-----END {}-----", label);
        if key.contains(&begin) && key.contains(&end) {
            return Ok(key_type.clone());
        }
    }

    bail!("Invalid SSH private key format. Supported formats: OpenSSH, RSA (PEM), ECDSA, Ed25519");
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SshKeyType {
    OpenSsh,
    Rsa,
    Ecdsa,
    Ed25519Pem,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openssh_private_key() {
        let key = "-----BEGIN OPENSSH PRIVATE KEY-----\nAAA...\n-----END OPENSSH PRIVATE KEY-----";
        assert!(matches!(
            validate_private_key(key),
            Ok(SshKeyType::OpenSsh)
        ));
    }

    #[test]
    fn test_validate_rsa_private_key() {
        let key = "-----BEGIN RSA PRIVATE KEY-----\nAAA...\n-----END RSA PRIVATE KEY-----";
        assert!(matches!(
            validate_private_key(key),
            Ok(SshKeyType::Rsa)
        ));
    }

    #[test]
    fn test_invalid_private_key() {
        let key = "invalid key content";
        assert!(validate_private_key(key).is_err());
    }
}
