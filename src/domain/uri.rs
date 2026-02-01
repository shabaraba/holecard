use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

static URI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^hc://(?:([^/]+)/)?([^/]+)/(.+)$").expect("Failed to compile URI regex")
});

static ENV_VAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)(?::-([^}]+))?\}").expect("Failed to compile env var regex")
});

#[derive(Debug, Clone, PartialEq)]
pub struct SecretUri {
    pub vault: Option<String>,
    pub item: String,
    pub field: String,
}

impl SecretUri {
    pub fn parse(uri: &str) -> Result<Self> {
        let uri = uri.trim();

        if !uri.starts_with("hc://") {
            anyhow::bail!("Invalid URI scheme. Expected 'hc://', got: {}", uri);
        }

        let caps = URI_REGEX
            .captures(uri)
            .ok_or_else(|| anyhow::anyhow!("Invalid URI format: {}", uri))?;

        let vault = caps.get(1).map(|m| m.as_str().to_string());
        let item = caps.get(2).unwrap().as_str().to_string();
        let field = caps.get(3).unwrap().as_str().to_string();

        if item.is_empty() {
            anyhow::bail!("Item name cannot be empty in URI: {}", uri);
        }
        if field.is_empty() {
            anyhow::bail!("Field name cannot be empty in URI: {}", uri);
        }

        Ok(Self { vault, item, field })
    }

    pub fn is_uri(s: &str) -> bool {
        s.trim().starts_with("hc://")
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
        assert_eq!(uri.vault, Some("production".to_string()));
        assert_eq!(uri.item, "database");
        assert_eq!(uri.field, "password");
    }

    #[test]
    fn test_parse_uri_without_vault() {
        let uri = SecretUri::parse("hc://github/token").unwrap();
        assert_eq!(uri.vault, None);
        assert_eq!(uri.item, "github");
        assert_eq!(uri.field, "token");
    }

    #[test]
    fn test_parse_uri_with_nested_field() {
        let uri = SecretUri::parse("hc://aws/credentials/access_key").unwrap();
        assert_eq!(uri.vault, Some("aws".to_string()));
        assert_eq!(uri.item, "credentials");
        assert_eq!(uri.field, "access_key");
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
        assert!(SecretUri::is_uri("hc://vault/item/field"));
        assert!(SecretUri::is_uri("  hc://vault/item/field  "));
        assert!(!SecretUri::is_uri("http://example.com"));
        assert!(!SecretUri::is_uri("plain text"));
    }

    #[test]
    fn test_expand_env_vars_with_default() {
        std::env::remove_var("UNDEFINED_VAR");
        let result = SecretUri::expand_env_vars("hc://${UNDEFINED_VAR:-default}/item/field");
        assert_eq!(result, "hc://default/item/field");
    }

    #[test]
    fn test_expand_env_vars_with_existing() {
        std::env::set_var("TEST_VAULT", "production");
        let result = SecretUri::expand_env_vars("hc://${TEST_VAULT:-default}/item/field");
        assert_eq!(result, "hc://production/item/field");
        std::env::remove_var("TEST_VAULT");
    }

    #[test]
    fn test_expand_env_vars_no_default() {
        std::env::remove_var("MISSING");
        let result = SecretUri::expand_env_vars("hc://${MISSING}/item/field");
        assert_eq!(result, "hc://${MISSING}/item/field");
    }
}
