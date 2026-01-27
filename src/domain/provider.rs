use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider configuration stored in encrypted storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub provider_id: String,
    pub credentials: HashMap<String, String>,
}

/// Provider trait for secret management services
pub trait Provider: Send + Sync {
    /// Push a single secret to the provider
    fn push_secret(&self, key: &str, value: &str) -> Result<()>;

    /// List all secrets in the provider
    fn list_secrets(&self) -> Result<Vec<String>>;

    /// Delete a secret from the provider
    fn delete_secret(&self, key: &str) -> Result<()>;

    /// Get provider name (e.g., "github", "cloudflare")
    fn provider_type(&self) -> &str;

    /// Get provider ID (e.g., "my-repo", "my-worker")
    fn provider_id(&self) -> &str;
}

/// Convert field name to secret name (snake_case/camelCase -> UPPER_SNAKE_CASE)
pub fn field_to_secret_name(field_name: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for (i, ch) in field_name.chars().enumerate() {
        if ch == '_' {
            result.push('_');
            prev_lower = false;
        } else if ch.is_uppercase() {
            if i > 0 && prev_lower {
                result.push('_');
            }
            result.push(ch);
            prev_lower = false;
        } else {
            result.push(ch.to_ascii_uppercase());
            prev_lower = true;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_to_secret_name() {
        assert_eq!(field_to_secret_name("db_url"), "DB_URL");
        assert_eq!(field_to_secret_name("apiKey"), "API_KEY");
        assert_eq!(field_to_secret_name("DATABASE_URL"), "DATABASE_URL");
        assert_eq!(field_to_secret_name("mySecretValue"), "MY_SECRET_VALUE");
    }
}
