use crate::id::{ChangeId, KeyId};
use crate::operation::Operation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncryptionMode {
    #[default]
    Plaintext,
    EncryptedPayload {
        key_id: KeyId,
        nonce: [u8; 12],
    },
    BlindChange {
        key_id: KeyId,
        nonce: [u8; 12],
        auth_tag: [u8; 16],
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeMetadata {
    pub author_id: Option<String>,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
    #[serde(default)]
    pub encryption_mode: EncryptionMode,
}

impl ChangeMetadata {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            author_id: None,
            device_id: device_id.into(),
            timestamp: Utc::now(),
            message: None,
            encryption_mode: EncryptionMode::Plaintext,
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

    pub fn with_encryption_mode(mut self, mode: EncryptionMode) -> Self {
        self.encryption_mode = mode;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub id: ChangeId,
    pub parents: Vec<ChangeId>,
    pub metadata: ChangeMetadata,
    pub operations: Vec<Operation>,
}

impl Change {
    pub fn new(
        id: ChangeId,
        parents: Vec<ChangeId>,
        metadata: ChangeMetadata,
        operations: Vec<Operation>,
    ) -> Self {
        Self {
            id,
            parents,
            metadata,
            operations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::EntityId;

    #[test]
    fn test_change_serialization() {
        let meta = ChangeMetadata::new("device-laptop")
            .with_author("user-alice")
            .with_message("Add first entity");

        let entity_id = EntityId::new();
        let ops = vec![Operation::CreateEntity {
            entity_id,
            entity_type: Some("note".into()),
        }];

        let change = Change::new(ChangeId::new(), vec![], meta, ops);
        let json = serde_json::to_string(&change).unwrap();
        let deserialized: Change = serde_json::from_str(&json).unwrap();

        assert_eq!(change, deserialized);
    }
}
