use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub custom_fields: HashMap<String, String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entry {
    pub fn new(
        name: String,
        custom_fields: HashMap<String, String>,
        notes: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            name,
            custom_fields,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
