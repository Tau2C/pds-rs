use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ulid::Ulid;

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Ulid);

        impl $name {
            pub fn new() -> Self {
                Self(Ulid::generate())
            }

            pub fn from_ulid(ulid: Ulid) -> Self {
                Self(ulid)
            }

            pub fn as_ulid(&self) -> Ulid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = ulid::DecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ulid::from_str(s).map(Self)
            }
        }

        impl From<Ulid> for $name {
            fn from(ulid: Ulid) -> Self {
                Self(ulid)
            }
        }

        impl From<$name> for Ulid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(EntityId, "Unikalny identyfikator encji.");
define_id!(RelationId, "Unikalny identyfikator relacji.");
define_id!(BlobId, "Unikalny identyfikator zasobu binarnego.");
define_id!(ChangeId, "Unikalny identyfikator jednostki zmiany.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation_and_string_roundtrip() {
        let entity_id = EntityId::new();
        let s = entity_id.to_string();
        let parsed: EntityId = s.parse().unwrap();
        assert_eq!(entity_id, parsed);
    }

    #[test]
    fn test_id_serde_json() {
        let rel_id = RelationId::new();
        let json = serde_json::to_string(&rel_id).unwrap();
        let deserialized: RelationId = serde_json::from_str(&json).unwrap();
        assert_eq!(rel_id, deserialized);
    }
}
