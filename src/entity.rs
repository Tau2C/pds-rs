use crate::id::EntityId;
use crate::value::{PropertyName, PropertyValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: Option<String>,
    pub properties: BTreeMap<PropertyName, PropertyValue>,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            entity_type: None,
            properties: BTreeMap::new(),
        }
    }

    pub fn with_type(id: EntityId, entity_type: impl Into<String>) -> Self {
        Self {
            id,
            entity_type: Some(entity_type.into()),
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
    fn test_entity_creation_and_properties() {
        let mut entity = Entity::with_type(EntityId::new(), "contact");
        assert_eq!(entity.entity_type.as_deref(), Some("contact"));

        let prop_name: PropertyName = "name".into();
        entity.set_property(prop_name.clone(), "Jan Kowalski");

        assert_eq!(
            entity.get_property(&prop_name),
            Some(&PropertyValue::from("Jan Kowalski"))
        );

        let json = serde_json::to_string(&entity).unwrap();
        let deserialized: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, deserialized);
    }
}
