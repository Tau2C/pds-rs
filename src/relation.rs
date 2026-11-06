use crate::id::{EntityId, RelationId};
use crate::value::{PropertyName, PropertyValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    pub relation_type: String,
    pub source: EntityId,
    pub target: EntityId,
    pub properties: BTreeMap<PropertyName, PropertyValue>,
}

impl Relation {
    pub fn new(
        id: RelationId,
        relation_type: impl Into<String>,
        source: EntityId,
        target: EntityId,
    ) -> Self {
        Self {
            id,
            relation_type: relation_type.into(),
            source,
            target,
            properties: BTreeMap::new(),
        }
    }

    pub fn set_property(&mut self, name: impl Into<PropertyName>, value: impl Into<PropertyValue>) {
        self.properties.insert(name.into(), value.into());
    }

    pub fn get_property(&self, name: &PropertyName) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    pub fn remove_property(&mut self, name: &PropertyName) -> Option<PropertyValue> {
        self.properties.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_creation_and_properties() {
        let e1 = EntityId::new();
        let e2 = EntityId::new();
        let mut rel = Relation::new(RelationId::new(), "created_by", e1, e2);

        let role_prop: PropertyName = "role".into();
        rel.set_property(role_prop.clone(), "author");

        assert_eq!(rel.source, e1);
        assert_eq!(rel.target, e2);
        assert_eq!(
            rel.get_property(&role_prop),
            Some(&PropertyValue::from("author"))
        );

        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(rel, deserialized);
    }
}
