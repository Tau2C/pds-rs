use crate::id::{EntityId, RelationId};
use crate::value::{PropertyName, PropertyValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
enum RawOperation {
    CreateEntity {
        entity_id: EntityId,
        entity_type: Option<String>,
    },
    SetEntityProperty {
        entity_id: EntityId,
        name: PropertyName,
        value: PropertyValue,
    },
    RemoveEntityProperty {
        entity_id: EntityId,
        name: PropertyName,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        relation_id: RelationId,
        relation_type: String,
        source: EntityId,
        target: EntityId,
    },
    SetRelationProperty {
        relation_id: RelationId,
        name: PropertyName,
        value: PropertyValue,
    },
    RemoveRelationProperty {
        relation_id: RelationId,
        name: PropertyName,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum OperationWire {
    Known(RawOperation),
    Unknown {
        action: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    CreateEntity {
        entity_id: EntityId,
        entity_type: Option<String>,
    },
    SetEntityProperty {
        entity_id: EntityId,
        name: PropertyName,
        value: PropertyValue,
    },
    RemoveEntityProperty {
        entity_id: EntityId,
        name: PropertyName,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        relation_id: RelationId,
        relation_type: String,
        source: EntityId,
        target: EntityId,
    },
    SetRelationProperty {
        relation_id: RelationId,
        name: PropertyName,
        value: PropertyValue,
    },
    RemoveRelationProperty {
        relation_id: RelationId,
        name: PropertyName,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
    Unknown {
        action: String,
        payload: serde_json::Value,
    },
}

impl From<RawOperation> for Operation {
    fn from(raw: RawOperation) -> Self {
        match raw {
            RawOperation::CreateEntity {
                entity_id,
                entity_type,
            } => Operation::CreateEntity {
                entity_id,
                entity_type,
            },
            RawOperation::SetEntityProperty {
                entity_id,
                name,
                value,
            } => Operation::SetEntityProperty {
                entity_id,
                name,
                value,
            },
            RawOperation::RemoveEntityProperty { entity_id, name } => {
                Operation::RemoveEntityProperty { entity_id, name }
            }
            RawOperation::DeleteEntity { entity_id } => Operation::DeleteEntity { entity_id },
            RawOperation::CreateRelation {
                relation_id,
                relation_type,
                source,
                target,
            } => Operation::CreateRelation {
                relation_id,
                relation_type,
                source,
                target,
            },
            RawOperation::SetRelationProperty {
                relation_id,
                name,
                value,
            } => Operation::SetRelationProperty {
                relation_id,
                name,
                value,
            },
            RawOperation::RemoveRelationProperty { relation_id, name } => {
                Operation::RemoveRelationProperty { relation_id, name }
            }
            RawOperation::DeleteRelation { relation_id } => {
                Operation::DeleteRelation { relation_id }
            }
        }
    }
}

impl From<OperationWire> for Operation {
    fn from(wire: OperationWire) -> Self {
        match wire {
            OperationWire::Known(raw) => raw.into(),
            OperationWire::Unknown { action, payload } => Operation::Unknown { action, payload },
        }
    }
}

impl From<Operation> for OperationWire {
    fn from(op: Operation) -> Self {
        match op {
            Operation::CreateEntity {
                entity_id,
                entity_type,
            } => OperationWire::Known(RawOperation::CreateEntity {
                entity_id,
                entity_type,
            }),
            Operation::SetEntityProperty {
                entity_id,
                name,
                value,
            } => OperationWire::Known(RawOperation::SetEntityProperty {
                entity_id,
                name,
                value,
            }),
            Operation::RemoveEntityProperty { entity_id, name } => {
                OperationWire::Known(RawOperation::RemoveEntityProperty { entity_id, name })
            }
            Operation::DeleteEntity { entity_id } => {
                OperationWire::Known(RawOperation::DeleteEntity { entity_id })
            }
            Operation::CreateRelation {
                relation_id,
                relation_type,
                source,
                target,
            } => OperationWire::Known(RawOperation::CreateRelation {
                relation_id,
                relation_type,
                source,
                target,
            }),
            Operation::SetRelationProperty {
                relation_id,
                name,
                value,
            } => OperationWire::Known(RawOperation::SetRelationProperty {
                relation_id,
                name,
                value,
            }),
            Operation::RemoveRelationProperty { relation_id, name } => {
                OperationWire::Known(RawOperation::RemoveRelationProperty { relation_id, name })
            }
            Operation::DeleteRelation { relation_id } => {
                OperationWire::Known(RawOperation::DeleteRelation { relation_id })
            }
            Operation::Unknown { action, payload } => OperationWire::Unknown { action, payload },
        }
    }
}

impl Serialize for Operation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = OperationWire::from(self.clone());
        wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Operation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = OperationWire::deserialize(deserializer)?;
        Ok(wire.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_known_operation_roundtrip() {
        let op = Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("note".into()),
        };
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Operation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_unknown_operation_forward_compatibility() {
        // Simulates an operation from a future PDS version (e.g. BatchCreate or VectorEmbedding)
        let future_json = json!({
            "action": "VectorEmbeddingIndex",
            "payload": {
                "dimensions": 1536,
                "model": "text-embedding-3-small"
            }
        })
        .to_string();

        let op: Operation = serde_json::from_str(&future_json).unwrap();
        match &op {
            Operation::Unknown { action, payload } => {
                assert_eq!(action, "VectorEmbeddingIndex");
                assert_eq!(payload["dimensions"], 1536);
            }
            other => panic!("Expected Operation::Unknown, got {:?}", other),
        }

        // Must re-serialize with exact same action and payload for DAG replication
        let reserialized = serde_json::to_string(&op).unwrap();
        let value: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(value["action"], "VectorEmbeddingIndex");
        assert_eq!(value["payload"]["dimensions"], 1536);
    }
}
