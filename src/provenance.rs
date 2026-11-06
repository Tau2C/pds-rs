use crate::id::{BlobId, ChangeId, EntityId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivityId(pub Ulid);

impl ActivityId {
    pub fn new() -> Self {
        Self(Ulid::generate())
    }
}

impl Default for ActivityId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityStatus {
    Started,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProvenanceTarget {
    Entity(EntityId),
    Blob(BlobId),
    Change(ChangeId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub id: ActivityId,
    pub process_name: String,
    pub version: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub inputs: Vec<ProvenanceTarget>,
    pub outputs: Vec<ProvenanceTarget>,
    pub configuration: BTreeMap<String, String>,
    pub status: ActivityStatus,
}

impl Activity {
    pub fn new(process_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: ActivityId::new(),
            process_name: process_name.into(),
            version: version.into(),
            started_at: Utc::now(),
            completed_at: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            configuration: BTreeMap::new(),
            status: ActivityStatus::Started,
        }
    }

    pub fn add_input(&mut self, input: ProvenanceTarget) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: ProvenanceTarget) {
        self.outputs.push(output);
    }

    pub fn set_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.configuration.insert(key.into(), value.into());
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = ActivityStatus::Completed;
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.completed_at = Some(Utc::now());
        self.status = ActivityStatus::Failed(error.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_lifecycle_and_provenance() {
        let mut act = Activity::new("ocr_pipeline", "1.2.0");
        let input_blob = BlobId::new();
        let output_entity = EntityId::new();

        act.add_input(ProvenanceTarget::Blob(input_blob));
        act.add_output(ProvenanceTarget::Entity(output_entity));
        act.set_config("language", "pl");
        act.complete();

        assert_eq!(act.status, ActivityStatus::Completed);
        assert_eq!(act.inputs.len(), 1);
        assert_eq!(act.outputs.len(), 1);

        let json = serde_json::to_string(&act).unwrap();
        let deserialized: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(act, deserialized);
    }
}
