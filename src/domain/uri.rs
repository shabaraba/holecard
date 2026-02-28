use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

static URI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:hc|op)://(?:([^/]+)/)?([^/]+)/(.+)$").expect("Failed to compile URI regex")
});

static ENV_VAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)(?::-([^}]+))?\}").expect("Failed to compile env var regex")
});

#[derive(Debug, Clone, PartialEq)]
pub struct SecretUri {
    pub deck: Option<String>,
    pub hand: String,
    pub card: String,
}

impl SecretUri {
    pub fn parse(uri: &str) -> Result<Self> {
        let uri = uri.trim();

        if !uri.starts_with("hc://") && !uri.starts_with("op://") {
            anyhow::bail!(
                "Invalid URI scheme. Expected 'hc://' or 'op://', got: {}",
                uri
            );
        }

        let caps = URI_REGEX
            .captures(uri)
            .ok_or_else(|| anyhow::anyhow!("Invalid URI format: {}", uri))?;

        let deck = caps.get(1).map(|m| m.as_str().to_string());
        let hand = caps.get(2).unwrap().as_str().to_string();
        let card = caps.get(3).unwrap().as_str().to_string();

        if hand.is_empty() {
            anyhow::bail!("Hand name cannot be empty in URI: {}", uri);
        }
        if card.is_empty() {
            anyhow::bail!("Card name cannot be empty in URI: {}", uri);
        }

        Ok(Self { deck, hand, card })
    }

    #[allow(dead_code)]
    pub fn is_uri(s: &str) -> bool {
        let trimmed = s.trim();
        trimmed.starts_with("hc://") || trimmed.starts_with("op://")
    }

    pub fn expand_env_vars(value: &str) -> String {
        ENV_VAR_REGEX
            .replace_all(value, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_value = caps.get(2).map(|m| m.as_str());

                std::env::var(var_name)
                    .ok()
                    .or_else(|| default_value.map(String::from))
                    .unwrap_or_else(|| format!("${{{}}}", var_name))
            })
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_uri() {
        let uri = SecretUri::parse("hc://production/database/password").unwrap();
        assert_eq!(uri.deck, Some("production".to_string()));
        assert_eq!(uri.hand, "database");
        assert_eq!(uri.card, "password");
    }

    #[test]
    fn test_parse_uri_without_deck() {
        let uri = SecretUri::parse("hc://github/token").unwrap();
        assert_eq!(uri.deck, None);
        assert_eq!(uri.hand, "github");
        assert_eq!(uri.card, "token");
    }

    #[test]
    fn test_parse_uri_with_nested_card() {
        let uri = SecretUri::parse("hc://aws/credentials/access_key").unwrap();
        assert_eq!(uri.deck, Some("aws".to_string()));
        assert_eq!(uri.hand, "credentials");
        assert_eq!(uri.card, "access_key");
    }

    #[test]
    fn test_parse_invalid_scheme() {
        let result = SecretUri::parse("http://vault/item/field");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid URI scheme"));
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = SecretUri::parse("hc://invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_uri() {
        assert!(SecretUri::is_uri("hc://deck/hand/card"));
        assert!(SecretUri::is_uri("  hc://deck/hand/card  "));
        assert!(!SecretUri::is_uri("http://example.com"));
        assert!(!SecretUri::is_uri("plain text"));
    }

    #[test]
    fn test_expand_env_vars_with_default() {
        std::env::remove_var("UNDEFINED_VAR");
        let result = SecretUri::expand_env_vars("hc://${UNDEFINED_VAR:-default}/hand/card");
        assert_eq!(result, "hc://default/hand/card");
    }

    #[test]
    fn test_expand_env_vars_with_existing() {
        std::env::set_var("TEST_DECK", "production");
        let result = SecretUri::expand_env_vars("hc://${TEST_DECK:-default}/hand/card");
        assert_eq!(result, "hc://production/hand/card");
        std::env::remove_var("TEST_DECK");
    }

    #[test]
    fn test_expand_env_vars_no_default() {
        std::env::remove_var("MISSING");
        let result = SecretUri::expand_env_vars("hc://${MISSING}/hand/card");
        assert_eq!(result, "hc://${MISSING}/hand/card");
    }

    #[test]
    fn test_parse_op_scheme_compatibility() {
        // op:// should also work (1Password compatibility)
        let uri = SecretUri::parse("op://production/database/password").unwrap();
        assert_eq!(uri.deck, Some("production".to_string()));
        assert_eq!(uri.hand, "database");
        assert_eq!(uri.card, "password");
    }

    #[test]
    fn test_is_uri_op_scheme() {
        assert!(SecretUri::is_uri("op://deck/hand/card"));
        assert!(SecretUri::is_uri("  op://deck/hand/card  "));
    }
}
