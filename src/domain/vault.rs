use super::entry::Entry;
use super::error::VaultError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    entries: HashMap<String, Entry>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), VaultError> {
        if self.entries.contains_key(&entry.name) {
            return Err(VaultError::EntryAlreadyExists(entry.name.clone()));
        }
        self.entries.insert(entry.name.clone(), entry);
        Ok(())
    }

    pub fn get_entry(&self, name: &str) -> Result<&Entry, VaultError> {
        self.entries
            .get(name)
            .ok_or_else(|| VaultError::EntryNotFound(name.to_string()))
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Result<&mut Entry, VaultError> {
        self.entries
            .get_mut(name)
            .ok_or_else(|| VaultError::EntryNotFound(name.to_string()))
    }

    pub fn remove_entry(&mut self, name: &str) -> Result<Entry, VaultError> {
        self.entries
            .remove(name)
            .ok_or_else(|| VaultError::EntryNotFound(name.to_string()))
    }

    pub fn list_entries(&self) -> Vec<&Entry> {
        let mut entries: Vec<&Entry> = self.entries.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    #[allow(dead_code)]
    pub fn entry_exists(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}
