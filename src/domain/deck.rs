use super::error::DeckError;
use super::hand::Hand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    #[serde(alias = "entries")]
    hands: HashMap<String, Hand>,
}

impl Deck {
    pub fn new() -> Self {
        Self {
            hands: HashMap::new(),
        }
    }

    pub fn add_hand(&mut self, hand: Hand) -> Result<(), DeckError> {
        if self.hands.contains_key(hand.name()) {
            return Err(DeckError::HandAlreadyExists(hand.name().to_string()));
        }
        self.hands.insert(hand.name().to_string(), hand);
        Ok(())
    }

    pub fn get_hand(&self, name: &str) -> Result<&Hand, DeckError> {
        self.hands
            .get(name)
            .ok_or_else(|| DeckError::HandNotFound(name.to_string()))
    }

    pub fn get_hand_mut(&mut self, name: &str) -> Result<&mut Hand, DeckError> {
        self.hands
            .get_mut(name)
            .ok_or_else(|| DeckError::HandNotFound(name.to_string()))
    }

    pub fn remove_hand(&mut self, name: &str) -> Result<Hand, DeckError> {
        self.hands
            .remove(name)
            .ok_or_else(|| DeckError::HandNotFound(name.to_string()))
    }

    pub fn list_hands(&self) -> Vec<&Hand> {
        let mut hands: Vec<&Hand> = self.hands.values().collect();
        hands.sort_by(|a, b| a.name().cmp(b.name()));
        hands
    }

    #[allow(dead_code)]
    pub fn rename_hand(&mut self, old_name: &str, new_name: String) -> Result<(), DeckError> {
        if self.hands.contains_key(&new_name) {
            return Err(DeckError::HandAlreadyExists(new_name));
        }
        let mut hand = self.remove_hand(old_name)?;
        hand.set_name(new_name.clone());
        self.hands.insert(new_name, hand);
        Ok(())
    }

    pub fn import_hand(&mut self, hand: Hand, overwrite: bool) -> Result<bool, DeckError> {
        if self.hands.contains_key(hand.name()) {
            if overwrite {
                self.hands.insert(hand.name().to_string(), hand);
                Ok(true)
            } else {
                Err(DeckError::HandAlreadyExists(hand.name().to_string()))
            }
        } else {
            self.hands.insert(hand.name().to_string(), hand);
            Ok(false)
        }
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}
