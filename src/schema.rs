use crate::entity::Entity;
use crate::value::{PropertyName, PropertyValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyType {
    Bool,
    Int,
    Float,
    Text,
    DateTime,
    EntityRef,
    BlobRef,
    List,
    Map,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub name: PropertyName,
    pub expected_type: PropertyType,
    pub required: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SchemaValidationError {
    #[error("Missing required property: {0}")]
    MissingRequired(PropertyName),
    #[error("Type mismatch for property {name}: expected {expected:?}, found {actual}")]
    TypeMismatch {
        name: PropertyName,
        expected: PropertyType,
        actual: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySchema {
    pub entity_type: String,
    pub version: u32,
    pub properties: BTreeMap<PropertyName, PropertyDefinition>,
}

impl EntitySchema {
    pub fn new(entity_type: impl Into<String>, version: u32) -> Self {
        Self {
            entity_type: entity_type.into(),
            version,
            properties: BTreeMap::new(),
        }
    }

    pub fn add_property(
        &mut self,
        name: impl Into<PropertyName>,
        expected_type: PropertyType,
        required: bool,
    ) {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            PropertyDefinition {
                name,
                expected_type,
                required,
            },
        );
    }

    pub fn validate(&self, entity: &Entity) -> Result<(), SchemaValidationError> {
        for (prop_name, def) in &self.properties {
            match entity.get_property(prop_name) {
                Some(val) => {
                    if !self.matches_type(val, &def.expected_type) {
                        return Err(SchemaValidationError::TypeMismatch {
                            name: prop_name.clone(),
                            expected: def.expected_type.clone(),
                            actual: self.val_type_name(val).into(),
                        });
                    }
                }
                None => {
                    if def.required {
                        return Err(SchemaValidationError::MissingRequired(prop_name.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    fn matches_type(&self, val: &PropertyValue, expected: &PropertyType) -> bool {
        match (val, expected) {
            (_, PropertyType::Any) => true,
            (PropertyValue::Encrypted { .. }, _) => true,
            (PropertyValue::Bool(_), PropertyType::Bool) => true,
            (PropertyValue::Int(_), PropertyType::Int) => true,
            (PropertyValue::Float(_), PropertyType::Float) => true,
            (PropertyValue::Text(_), PropertyType::Text) => true,
            (PropertyValue::DateTime(_), PropertyType::DateTime) => true,
            (PropertyValue::EntityRef(_), PropertyType::EntityRef) => true,
            (PropertyValue::BlobRef(_), PropertyType::BlobRef) => true,
            (PropertyValue::List(_), PropertyType::List) => true,
            (PropertyValue::Map(_), PropertyType::Map) => true,
            _ => false,
        }
    }

    fn val_type_name(&self, val: &PropertyValue) -> &'static str {
        match val {
            PropertyValue::Null => "Null",
            PropertyValue::Bool(_) => "Bool",
            PropertyValue::Int(_) => "Int",
            PropertyValue::Float(_) => "Float",
            PropertyValue::Text(_) => "Text",
            PropertyValue::DateTime(_) => "DateTime",
            PropertyValue::EntityRef(_) => "EntityRef",
            PropertyValue::BlobRef(_) => "BlobRef",
            PropertyValue::Encrypted { .. } => "Encrypted",
            PropertyValue::List(_) => "List",
            PropertyValue::Map(_) => "Map",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::EntityId;

    #[test]
    fn test_schema_validation() {
        let mut schema = EntitySchema::new("document", 1);
        schema.add_property("title", PropertyType::Text, true);
        schema.add_property("pages", PropertyType::Int, false);

        let mut entity = Entity::with_type(EntityId::new(), "document");
        assert!(matches!(
            schema.validate(&entity),
            Err(SchemaValidationError::MissingRequired(_))
        ));

        entity.set_property("title", "Dokument testowy");
        assert!(schema.validate(&entity).is_ok());

        entity.set_property("pages", "not-a-number");
        assert!(matches!(
            schema.validate(&entity),
            Err(SchemaValidationError::TypeMismatch { .. })
        ));

        entity.set_property("pages", 12);
        entity.set_property("custom_unknown_field", "dane z innej aplikacji");
        assert!(schema.validate(&entity).is_ok());
    }
}
