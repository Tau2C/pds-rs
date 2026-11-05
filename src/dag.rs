use crate::change::Change;
use crate::id::ChangeId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("Missing parent change: {0}")]
    MissingParent(ChangeId),
    #[error("Change already exists: {0}")]
    AlreadyExists(ChangeId),
}

#[derive(Debug, Clone, Default)]
pub struct HistoryDag {
    changes: BTreeMap<ChangeId, Change>,
    heads: BTreeSet<ChangeId>,
}

impl HistoryDag {
    pub fn new() -> Self {
        Self {
            changes: BTreeMap::new(),
            heads: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, change: Change) -> Result<(), DagError> {
        let change_id = change.id;
        if self.changes.contains_key(&change_id) {
            return Err(DagError::AlreadyExists(change_id));
        }

        for parent in &change.parents {
            if !self.changes.contains_key(parent) {
                return Err(DagError::MissingParent(*parent));
            }
        }

        for parent in &change.parents {
            self.heads.remove(parent);
        }

        self.heads.insert(change_id);
        self.changes.insert(change_id, change);
        Ok(())
    }

    pub fn get(&self, id: &ChangeId) -> Option<&Change> {
        self.changes.get(id)
    }

    pub fn heads(&self) -> &BTreeSet<ChangeId> {
        &self.heads
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn topological_order(&self) -> Vec<&Change> {
        let mut in_degree: BTreeMap<ChangeId, usize> = BTreeMap::new();
        let mut dependents: BTreeMap<ChangeId, Vec<ChangeId>> = BTreeMap::new();

        for (id, change) in &self.changes {
            in_degree.insert(*id, change.parents.len());
            for parent in &change.parents {
                dependents.entry(*parent).or_default().push(*id);
            }
        }

        let mut queue: VecDeque<ChangeId> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut ordered = Vec::with_capacity(self.changes.len());

        while let Some(current) = queue.pop_front() {
            if let Some(change) = self.changes.get(&current) {
                ordered.push(change);
            }
            if let Some(deps) = dependents.get(&current) {
                for dep in deps {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(*dep);
                    }
                }
            }
        }

        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeMetadata;

    #[test]
    fn test_dag_insertion_and_heads() {
        let mut dag = HistoryDag::new();

        let root_id = ChangeId::new();
        let root_change = Change::new(
            root_id,
            vec![],
            ChangeMetadata::new("device-1"),
            vec![],
        );
        dag.insert(root_change).unwrap();
        assert_eq!(dag.heads().len(), 1);
        assert!(dag.heads().contains(&root_id));

        let c1_id = ChangeId::new();
        let c1 = Change::new(
            c1_id,
            vec![root_id],
            ChangeMetadata::new("device-1"),
            vec![],
        );
        dag.insert(c1).unwrap();
        assert_eq!(dag.heads().len(), 1);
        assert!(dag.heads().contains(&c1_id));

        let c2_id = ChangeId::new();
        let c2 = Change::new(
            c2_id,
            vec![root_id],
            ChangeMetadata::new("device-2"),
            vec![],
        );
        dag.insert(c2).unwrap();
        assert_eq!(dag.heads().len(), 2);
        assert!(dag.heads().contains(&c1_id));
        assert!(dag.heads().contains(&c2_id));

        let order = dag.topological_order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0].id, root_id);
    }
}
