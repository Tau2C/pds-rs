use crate::change::Change;
use crate::dag::HistoryDag;
use crate::entity::Entity;
use crate::id::{EntityId, RelationId};
use crate::operation::Operation;
use crate::relation::Relation;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DatasetState {
    pub entities: BTreeMap<EntityId, Entity>,
    pub relations: BTreeMap<RelationId, Relation>,
}

impl DatasetState {
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        }
    }

    pub fn apply_operation(&mut self, op: &Operation) {
        match op {
            Operation::CreateEntity {
                entity_id,
                entity_type,
            } => {
                let entity = match entity_type {
                    Some(t) => Entity::with_type(*entity_id, t),
                    None => Entity::new(*entity_id),
                };
                self.entities.insert(*entity_id, entity);
            }
            Operation::SetEntityProperty {
                entity_id,
                name,
                value,
            } => {
                if let Some(entity) = self.entities.get_mut(entity_id) {
                    entity.set_property(name.clone(), value.clone());
                }
            }
            Operation::RemoveEntityProperty { entity_id, name } => {
                if let Some(entity) = self.entities.get_mut(entity_id) {
                    entity.remove_property(name);
                }
            }
            Operation::DeleteEntity { entity_id } => {
                self.entities.remove(entity_id);
                self.relations
                    .retain(|_, rel| rel.source != *entity_id && rel.target != *entity_id);
            }
            Operation::CreateRelation {
                relation_id,
                relation_type,
                source,
                target,
            } => {
                let relation = Relation::new(*relation_id, relation_type, *source, *target);
                self.relations.insert(*relation_id, relation);
            }
            Operation::SetRelationProperty {
                relation_id,
                name,
                value,
            } => {
                if let Some(relation) = self.relations.get_mut(relation_id) {
                    relation.set_property(name.clone(), value.clone());
                }
            }
            Operation::RemoveRelationProperty { relation_id, name } => {
                if let Some(relation) = self.relations.get_mut(relation_id) {
                    relation.remove_property(name);
                }
            }
            Operation::DeleteRelation { relation_id } => {
                self.relations.remove(relation_id);
            }
            Operation::Unknown { .. } => {
                // Unknown operations from newer or foreign versions are preserved in HistoryDag,
                // but safely ignored during local state projection.
            }
        }
    }

    pub fn apply_change(&mut self, change: &Change) {
        for op in &change.operations {
            self.apply_operation(op);
        }
    }

    pub fn from_dag(dag: &HistoryDag) -> Self {
        let mut state = Self::new();
        for change in dag.topological_order() {
            state.apply_change(change);
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeMetadata;
    use crate::id::ChangeId;
    use crate::value::PropertyValue;

    #[test]
    fn test_materialize_from_dag() {
        let mut dag = HistoryDag::new();

        let e_id = EntityId::new();
        let c1_id = ChangeId::new();
        let c1 = Change::new(
            c1_id,
            vec![],
            ChangeMetadata::new("device-1"),
            vec![
                Operation::CreateEntity {
                    entity_id: e_id,
                    entity_type: Some("task".into()),
                },
                Operation::SetEntityProperty {
                    entity_id: e_id,
                    name: "title".into(),
                    value: PropertyValue::from("Implement DAG"),
                },
            ],
        );
        dag.insert(c1).unwrap();

        let c2_id = ChangeId::new();
        let c2 = Change::new(
            c2_id,
            vec![c1_id],
            ChangeMetadata::new("device-1"),
            vec![Operation::SetEntityProperty {
                entity_id: e_id,
                name: "done".into(),
                value: PropertyValue::from(true),
            }],
        );
        dag.insert(c2).unwrap();

        let state = DatasetState::from_dag(&dag);
        let entity = state.entities.get(&e_id).expect("Entity should exist");
        assert_eq!(entity.entity_type.as_deref(), Some("task"));
        assert_eq!(
            entity.get_property(&"title".into()),
            Some(&PropertyValue::from("Implement DAG"))
        );
        assert_eq!(
            entity.get_property(&"done".into()),
            Some(&PropertyValue::from(true))
        );
    }
}
