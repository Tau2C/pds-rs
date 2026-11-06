//! v1.0.0 client simulator.
//!
//! This module produces JSON payloads that match the wire format used by
//! PDS v1.0.0 clients. The `ChangeMetadata` struct in that version had no
//! `encryption_mode` field.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Wire format of `ChangeMetadata` as used by a v1.0.0 client.
/// The field `encryption_mode` did not exist in v1.0.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyChangeMetadataV100 {
    pub author_id: Option<String>,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

impl LegacyChangeMetadataV100 {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            author_id: None,
            device_id: device_id.into(),
            timestamp: Utc::now(),
            message: None,
        }
    }

    pub fn with_author(mut self, author_id: impl Into<String>) -> Self {
        self.author_id = Some(author_id.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Serializes this struct to JSON.
    /// The result intentionally omits `encryption_mode`.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialization is infallible")
    }
}
