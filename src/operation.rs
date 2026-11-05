use crate::id::{EntityId, RelationId};
use crate::value::{PropertyName, PropertyValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
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
}
