use crate::blob::{BlobError, BlobStore};
use crate::change::Change;
use crate::dag::{DagError, HistoryDag};
use crate::id::ChangeId;
use crate::state::DatasetState;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("DAG error: {0}")]
    Dag(#[from] DagError),
    #[error("Blob error: {0}")]
    Blob(#[from] BlobError),
}

pub struct DiskStorage {
    root_dir: PathBuf,
    dag: HistoryDag,
    state: DatasetState,
    blobs: BlobStore,
}

impl DiskStorage {
    pub fn open(root_dir: impl AsRef<Path>) -> Result<Self, StorageError> {
        let root_dir = root_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)?;

        let blob_dir = root_dir.join("blobs");
        let blobs = BlobStore::new(blob_dir)?;

        let mut storage = Self {
            root_dir,
            dag: HistoryDag::new(),
            state: DatasetState::new(),
            blobs,
        };

        storage.load_history()?;
        Ok(storage)
    }

    fn history_file_path(&self) -> PathBuf {
        self.root_dir.join("history.jsonl")
    }

    fn load_history(&mut self) -> Result<(), StorageError> {
        let path = self.history_file_path();
        if !path.exists() {
            return Ok(());
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let change: Change = serde_json::from_str(&line)?;
            self.dag.insert(change)?;
        }

        self.state = DatasetState::from_dag(&self.dag);
        Ok(())
    }

    pub fn append_change(&mut self, change: Change) -> Result<(), StorageError> {
        self.dag.insert(change.clone())?;
        self.state.apply_change(&change);

        let path = self.history_file_path();
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        let serialized = serde_json::to_string(&change)?;
        writeln!(file, "{}", serialized)?;
        file.flush()?;

        Ok(())
    }

    pub fn dag(&self) -> &HistoryDag {
        &self.dag
    }

    pub fn state(&self) -> &DatasetState {
        &self.state
    }

    pub fn blobs(&self) -> &BlobStore {
        &self.blobs
    }

    pub fn blobs_mut(&mut self) -> &mut BlobStore {
        &mut self.blobs
    }

    pub fn change_count(&self) -> usize {
        self.dag.len()
    }

    pub fn get_change(&self, id: &ChangeId) -> Option<&Change> {
        self.dag.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeMetadata;
    use crate::id::EntityId;
    use crate::operation::Operation;
    use crate::value::PropertyValue;

    #[test]
    fn test_disk_storage_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("pds_disk_test_{}", EntityId::new()));

        let entity_id = EntityId::new();
        let c1_id = ChangeId::new();

        {
            let mut storage = DiskStorage::open(&temp_dir).unwrap();
            let c1 = Change::new(
                c1_id,
                vec![],
                ChangeMetadata::new("laptop-1"),
                vec![
                    Operation::CreateEntity {
                        entity_id,
                        entity_type: Some("doc".into()),
                    },
                    Operation::SetEntityProperty {
                        entity_id,
                        name: "name".into(),
                        value: PropertyValue::from("Specyfikacja"),
                    },
                ],
            );
            storage.append_change(c1).unwrap();
            assert_eq!(storage.change_count(), 1);
        }

        {
            let storage = DiskStorage::open(&temp_dir).unwrap();
            assert_eq!(storage.change_count(), 1);
            let entity = storage.state().entities.get(&entity_id).unwrap();
            assert_eq!(entity.entity_type.as_deref(), Some("doc"));
            assert_eq!(
                entity.get_property(&"name".into()),
                Some(&PropertyValue::from("Specyfikacja"))
            );
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
