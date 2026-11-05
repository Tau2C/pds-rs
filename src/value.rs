use crate::id::{BlobId, EntityId};
use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyName {
    pub namespace: Option<String>,
    pub name: String,
}

impl PropertyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            namespace: None,
            name: name.into(),
        }
    }

    pub fn with_namespace(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            name: name.into(),
        }
    }
}

impl fmt::Display for PropertyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespace {
            Some(ns) => write!(f, "{}:{}", ns, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl FromStr for PropertyName {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((ns, name)) = s.split_once(':') {
            Self::with_namespace(ns, name)
        } else {
            Self::new(s)
        })
    }
}

impl From<&str> for PropertyName {
    fn from(s: &str) -> Self {
        s.parse().unwrap()
    }
}

impl Serialize for PropertyName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PropertyName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    DateTime(DateTime<Utc>),
    EntityRef(EntityId),
    BlobRef(BlobId),
    List(Vec<PropertyValue>),
    Map(BTreeMap<String, PropertyValue>),
}

impl From<bool> for PropertyValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i64> for PropertyValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<i32> for PropertyValue {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<f64> for PropertyValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<&str> for PropertyValue {
    fn from(v: &str) -> Self {
        Self::Text(v.to_string())
    }
}

impl From<String> for PropertyValue {
    fn from(v: String) -> Self {
        Self::Text(v)
    }
}

impl From<DateTime<Utc>> for PropertyValue {
    fn from(v: DateTime<Utc>) -> Self {
        Self::DateTime(v)
    }
}

impl From<EntityId> for PropertyValue {
    fn from(v: EntityId) -> Self {
        Self::EntityRef(v)
    }
}

impl From<BlobId> for PropertyValue {
    fn from(v: BlobId) -> Self {
        Self::BlobRef(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_name_parsing() {
        let p1: PropertyName = "title".into();
        assert_eq!(p1.namespace, None);
        assert_eq!(p1.name, "title");
        assert_eq!(p1.to_string(), "title");

        let p2: PropertyName = "osp:hub_id".into();
        assert_eq!(p2.namespace.as_deref(), Some("osp"));
        assert_eq!(p2.name, "hub_id");
        assert_eq!(p2.to_string(), "osp:hub_id");
    }

    #[test]
    fn test_property_value_serialization() {
        let val = PropertyValue::List(vec![
            PropertyValue::from(42),
            PropertyValue::from("dokument"),
            PropertyValue::from(true),
        ]);
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: PropertyValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, deserialized);
    }
}
