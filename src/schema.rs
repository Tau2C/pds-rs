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

    pub fn validate_operation(&self, op: &crate::operation::Operation) -> Result<(), SchemaValidationError> {
        match op {
            crate::operation::Operation::SetEntityProperty { name, value, .. } => {
                if let Some(def) = self.properties.get(name).filter(|d| !self.matches_type(value, &d.expected_type)) {
                    Err(SchemaValidationError::TypeMismatch {
                        name: name.clone(),
                        expected: def.expected_type.clone(),
                        actual: self.val_type_name(value).into(),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn matches_type(&self, val: &PropertyValue, expected: &PropertyType) -> bool {
        matches!(
            (val, expected),
            (_, PropertyType::Any)
                | (PropertyValue::Encrypted { .. }, _)
                | (PropertyValue::Bool(_), PropertyType::Bool)
                | (PropertyValue::Int(_), PropertyType::Int)
                | (PropertyValue::Float(_), PropertyType::Float)
                | (PropertyValue::Text(_), PropertyType::Text)
                | (PropertyValue::DateTime(_), PropertyType::DateTime)
                | (PropertyValue::EntityRef(_), PropertyType::EntityRef)
                | (PropertyValue::BlobRef(_), PropertyType::BlobRef)
                | (PropertyValue::List(_), PropertyType::List)
                | (PropertyValue::Map(_), PropertyType::Map)
        )
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

    #[test]
    fn test_schema_validate_operation() {
        let mut schema = EntitySchema::new("document", 1);
        schema.add_property("title", PropertyType::Text, true);
        schema.add_property("pages", PropertyType::Int, false);

        let entity_id = EntityId::new();

        let valid_op = crate::operation::Operation::SetEntityProperty {
            entity_id,
            name: "title".into(),
            value: PropertyValue::Text("Valid Title".into()),
        };
        assert!(schema.validate_operation(&valid_op).is_ok());

        let invalid_op = crate::operation::Operation::SetEntityProperty {
            entity_id,
            name: "pages".into(),
            value: PropertyValue::Text("nan".into()),
        };
        assert!(matches!(
            schema.validate_operation(&invalid_op),
            Err(SchemaValidationError::TypeMismatch { .. })
        ));

        // Unknown property on schema is allowed (open model)
        let custom_op = crate::operation::Operation::SetEntityProperty {
            entity_id,
            name: "tags".into(),
            value: PropertyValue::List(vec![]),
        };
        assert!(schema.validate_operation(&custom_op).is_ok());
    }
}
