use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    name: String,
    #[serde(alias = "custom_fields")]
    pub cards: HashMap<String, String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Hand {
    pub fn new(name: String, cards: HashMap<String, String>, notes: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            name,
            cards,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
