use crate::blob::BlobDescriptor;
use crate::change::Change;
use crate::schema::EntitySchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdsArchiveMetadata {
    pub format_version: u32,
    pub exported_at: DateTime<Utc>,
    pub pds_version: String,
    pub change_count: usize,
    pub blob_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdsArchive {
    pub metadata: PdsArchiveMetadata,
    pub changes: Vec<Change>,
    pub schemas: Vec<EntitySchema>,
    pub blob_descriptors: Vec<BlobDescriptor>,
}

impl PdsArchive {
    pub fn new(
        changes: Vec<Change>,
        schemas: Vec<EntitySchema>,
        blob_descriptors: Vec<BlobDescriptor>,
    ) -> Self {
        let metadata = PdsArchiveMetadata {
            format_version: 1,
            exported_at: Utc::now(),
            pds_version: env!("CARGO_PKG_VERSION").to_string(),
            change_count: changes.len(),
            blob_count: blob_descriptors.len(),
        };

        Self {
            metadata,
            changes,
            schemas,
            blob_descriptors,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_serde() {
        let archive = PdsArchive::new(vec![], vec![], vec![]);
        let json = archive.to_json().unwrap();
        let restored = PdsArchive::from_json(&json).unwrap();
        assert_eq!(
            archive.metadata.format_version,
            restored.metadata.format_version
        );
        assert_eq!(archive.changes.len(), restored.changes.len());
    }
}
