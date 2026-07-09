use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRecord {
    pub target: String,
    pub source: String,
    pub fields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl DebugRecord {
    pub fn new(target: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            source: source.into(),
            fields: BTreeMap::new(),
            note: None,
        }
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}
