use anyhow::Result;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use crate::domain::uri::SecretUri;
use crate::infrastructure::KeyringManager;
use crate::multi_vault_context::MultiVaultContext;

static TEMPLATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:hc|op)://(?:[^/]+/)?[^/\s]+/[^\s]+").expect("Failed to compile template regex")
});

pub struct SecretResolver;

impl SecretResolver {
    pub fn resolve(
        uri_str: &str,
        default_vault: Option<&str>,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<String> {
        let expanded = SecretUri::expand_env_vars(uri_str);
        let uri = SecretUri::parse(&expanded)?;

        let vault_name = uri.vault.as_deref().or(default_vault);
        let ctx = MultiVaultContext::load(vault_name, keyring, config_dir)?;

        let entry = ctx
            .inner
            .vault
            .get_entry(&uri.item)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        entry.custom_fields.get(&uri.field).cloned().ok_or_else(|| {
            anyhow::anyhow!("Field '{}' not found in entry '{}'", uri.field, uri.item)
        })
    }

    pub fn resolve_template(
        template: &str,
        default_vault: Option<&str>,
        keyring: &KeyringManager,
        config_dir: &Path,
    ) -> Result<String> {
        let mut replacements = Vec::new();
        let mut errors = Vec::new();

        for cap in TEMPLATE_REGEX.captures_iter(template) {
            let full_match = cap.get(0).unwrap();
            let uri_str = full_match.as_str().trim();

            match Self::resolve(uri_str, default_vault, keyring, config_dir) {
                Ok(value) => {
                    replacements.push((full_match.range(), value));
                }
                Err(e) => {
                    errors.push(format!("{}: {}", uri_str, e));
                }
            }
        }

        if !errors.is_empty() {
            anyhow::bail!("Failed to resolve secrets:\n  {}", errors.join("\n  "));
        }

        let mut result = template.to_string();
        for (range, value) in replacements.iter().rev() {
            result.replace_range(range.clone(), value);
        }

        Ok(result)
    }

    pub fn has_uri_references(text: &str) -> bool {
        text.contains("hc://") || text.contains("op://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_uri_references() {
        assert!(SecretResolver::has_uri_references("hc://vault/item/field"));
        assert!(SecretResolver::has_uri_references("op://vault/item/field"));
        assert!(SecretResolver::has_uri_references(
            "password: hc://prod/db/password"
        ));
        assert!(!SecretResolver::has_uri_references("plain text"));
        assert!(!SecretResolver::has_uri_references("http://example.com"));
    }
}
