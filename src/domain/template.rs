use crate::domain::{Deck, Hand};
use anyhow::{Context, Result};
use regex::Regex;

pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a template string with hand data
    /// Supports:
    /// - {{card.key}} - specific card value from hand
    /// - {{card}} - all cards as KEY=value format
    #[allow(dead_code)]
    pub fn render(template: &str, hand: &Hand) -> Result<String> {
        let re = Regex::new(r"\{\{([^}]+)\}\}").context("Failed to compile regex")?;

        let mut result = template.to_string();
        let mut missing_fields = Vec::new();

        for cap in re.captures_iter(template) {
            let full_match = &cap[0];
            let var_name = cap[1].trim();

            let replacement = if var_name == "card" {
                // {{hand}} -> expand all fields
                Self::expand_hand(hand)
            } else if let Some((card_part, field_part)) = var_name.split_once('.') {
                // {{hand.field}} -> lookup specific field
                if card_part == "card" {
                    if let Some(value) = hand.cards.get(field_part) {
                        value.clone()
                    } else {
                        missing_fields.push(field_part.to_string());
                        continue;
                    }
                } else {
                    anyhow::bail!(
                        "Invalid template variable: {}. Only 'card' is supported",
                        card_part
                    );
                }
            } else {
                anyhow::bail!(
                    "Invalid template syntax: {}. Use {{{{card.field}}}} or {{{{card}}}}",
                    var_name
                );
            };

            result = result.replace(full_match, &replacement);
        }

        if !missing_fields.is_empty() {
            anyhow::bail!(
                "Missing cards in hand '{}': {}",
                hand.name(),
                missing_fields.join(", ")
            );
        }

        Ok(result)
    }

    #[allow(dead_code)]
    fn expand_hand(hand: &Hand) -> String {
        hand.cards
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_uppercase(), v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Resolve a template string that may contain {{hand_name.card}} references
    /// Returns the resolved value or the original string if not a template
    pub fn resolve_value(value: &str, deck: &Deck) -> Result<String> {
        let re = Regex::new(r"^\{\{([^.]+)\.([^}]+)\}\}$").context("Failed to compile regex")?;

        if let Some(cap) = re.captures(value) {
            let hand_name = cap[1].trim();
            let card_name = cap[2].trim();

            let hand = deck
                .get_hand(hand_name)
                .with_context(|| format!("Hand '{}' not found in deck", hand_name))?;

            hand.cards.get(card_name).cloned().ok_or_else(|| {
                anyhow::anyhow!("Card '{}' not found in hand '{}'", card_name, hand_name)
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

    fn create_test_hand() -> Hand {
        let mut cards = HashMap::new();
        cards.insert("username".to_string(), "john".to_string());
        cards.insert("password".to_string(), "secret123".to_string());
        cards.insert("host".to_string(), "db.example.com".to_string());

        Hand::new("testhand".to_string(), cards, None)
    }

    #[test]
    fn test_render_single_field() {
        let hand = create_test_hand();
        let result = TemplateEngine::render("User: {{card.username}}", &hand).unwrap();
        assert_eq!(result, "User: john");
    }

    #[test]
    fn test_render_multiple_fields() {
        let hand = create_test_hand();
        let result =
            TemplateEngine::render("Connect to {{card.host}} as {{card.username}}", &hand).unwrap();
        assert_eq!(result, "Connect to db.example.com as john");
    }

    #[test]
    fn test_render_entire_hand() {
        let hand = create_test_hand();
        let result = TemplateEngine::render("{{card}}", &hand).unwrap();

        // Order might vary, check all fields are present
        assert!(result.contains("USERNAME=john"));
        assert!(result.contains("PASSWORD=secret123"));
        assert!(result.contains("HOST=db.example.com"));
    }

    #[test]
    fn test_missing_field_error() {
        let hand = create_test_hand();
        let result = TemplateEngine::render("{{card.nonexistent}}", &hand);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing cards"));
    }

    #[test]
    fn test_invalid_syntax() {
        let hand = create_test_hand();
        let result = TemplateEngine::render("{{invalid}}", &hand);
        assert!(result.is_err());
    }
}
