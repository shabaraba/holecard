use crate::domain::{Entry, Vault};
use anyhow::{Context, Result};
use regex::Regex;

pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a template string with entry data
    /// Supports:
    /// - {{entry.field}} - specific field from entry
    /// - {{entry}} - all fields as KEY=value format
    pub fn render(template: &str, entry: &Entry) -> Result<String> {
        let re = Regex::new(r"\{\{([^}]+)\}\}").context("Failed to compile regex")?;

        let mut result = template.to_string();
        let mut missing_fields = Vec::new();

        for cap in re.captures_iter(template) {
            let full_match = &cap[0];
            let var_name = cap[1].trim();

            let replacement = if var_name == "entry" {
                // {{entry}} -> expand all fields
                Self::expand_entry(entry)
            } else if let Some((entry_part, field_part)) = var_name.split_once('.') {
                // {{entry.field}} -> lookup specific field
                if entry_part == "entry" {
                    if let Some(value) = entry.custom_fields.get(field_part) {
                        value.clone()
                    } else {
                        missing_fields.push(field_part.to_string());
                        continue;
                    }
                } else {
                    anyhow::bail!(
                        "Invalid template variable: {}. Only 'entry' is supported",
                        entry_part
                    );
                }
            } else {
                anyhow::bail!(
                    "Invalid template syntax: {}. Use {{{{entry.field}}}} or {{{{entry}}}}",
                    var_name
                );
            };

            result = result.replace(full_match, &replacement);
        }

        if !missing_fields.is_empty() {
            anyhow::bail!(
                "Missing fields in entry '{}': {}",
                entry.name,
                missing_fields.join(", ")
            );
        }

        Ok(result)
    }

    fn expand_entry(entry: &Entry) -> String {
        entry
            .custom_fields
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_uppercase(), v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Resolve a template string that may contain {{entry_name.field}} references
    /// Returns the resolved value or the original string if not a template
    pub fn resolve_value(value: &str, vault: &Vault) -> Result<String> {
        let re = Regex::new(r"^\{\{([^.]+)\.([^}]+)\}\}$").context("Failed to compile regex")?;

        if let Some(cap) = re.captures(value) {
            let entry_name = cap[1].trim();
            let field_name = cap[2].trim();

            let entry = vault
                .get_entry(entry_name)
                .with_context(|| format!("Entry '{}' not found in vault", entry_name))?;

            entry.custom_fields.get(field_name).cloned().ok_or_else(|| {
                anyhow::anyhow!("Field '{}' not found in entry '{}'", field_name, entry_name)
            })
        } else {
            Ok(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_entry() -> Entry {
        let mut fields = HashMap::new();
        fields.insert("username".to_string(), "john".to_string());
        fields.insert("password".to_string(), "secret123".to_string());
        fields.insert("host".to_string(), "db.example.com".to_string());

        Entry::new("testentry".to_string(), fields, None)
    }

    #[test]
    fn test_render_single_field() {
        let entry = create_test_entry();
        let result = TemplateEngine::render("User: {{entry.username}}", &entry).unwrap();
        assert_eq!(result, "User: john");
    }

    #[test]
    fn test_render_multiple_fields() {
        let entry = create_test_entry();
        let result =
            TemplateEngine::render("Connect to {{entry.host}} as {{entry.username}}", &entry)
                .unwrap();
        assert_eq!(result, "Connect to db.example.com as john");
    }

    #[test]
    fn test_render_entire_entry() {
        let entry = create_test_entry();
        let result = TemplateEngine::render("{{entry}}", &entry).unwrap();

        // Order might vary, check all fields are present
        assert!(result.contains("USERNAME=john"));
        assert!(result.contains("PASSWORD=secret123"));
        assert!(result.contains("HOST=db.example.com"));
    }

    #[test]
    fn test_missing_field_error() {
        let entry = create_test_entry();
        let result = TemplateEngine::render("{{entry.nonexistent}}", &entry);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing fields"));
    }

    #[test]
    fn test_invalid_syntax() {
        let entry = create_test_entry();
        let result = TemplateEngine::render("{{invalid}}", &entry);
        assert!(result.is_err());
    }
}
