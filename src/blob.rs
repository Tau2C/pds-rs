use crate::id::BlobId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Integrity check failed. Expected hash: {expected}, got: {actual}")]
    IntegrityMismatch { expected: String, actual: String },
    #[error("Blob not found: {0}")]
    NotFound(BlobId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobDescriptor {
    pub id: BlobId,
    pub sha256: String,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BlobStore {
    root_dir: PathBuf,
    descriptors: BTreeMap<BlobId, BlobDescriptor>,
}

impl BlobStore {
    pub fn new(root_dir: impl AsRef<Path>) -> io::Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)?;
        Ok(Self {
            root_dir,
            descriptors: BTreeMap::new(),
        })
    }

    pub fn compute_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn blob_path(&self, sha256: &str) -> PathBuf {
        let prefix = &sha256[..2];
        let rest = &sha256[2..];
        self.root_dir.join(prefix).join(rest)
    }

    pub fn put(
        &mut self,
        id: BlobId,
        data: &[u8],
        mime_type: Option<String>,
    ) -> Result<BlobDescriptor, BlobError> {
        let sha256 = Self::compute_sha256(data);
        let path = self.blob_path(&sha256);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !path.exists() {
            fs::write(&path, data)?;
        }

        let descriptor = BlobDescriptor {
            id,
            sha256,
            size_bytes: data.len() as u64,
            mime_type,
        };

        self.descriptors.insert(id, descriptor.clone());
        Ok(descriptor)
    }

    pub fn get_descriptor(&self, id: &BlobId) -> Option<&BlobDescriptor> {
        self.descriptors.get(id)
    }

    pub fn get_data(&self, id: &BlobId) -> Result<Vec<u8>, BlobError> {
        let descriptor = self.descriptors.get(id).ok_or(BlobError::NotFound(*id))?;
        let path = self.blob_path(&descriptor.sha256);
        let data = fs::read(&path)?;

        let actual_hash = Self::compute_sha256(&data);
        if actual_hash != descriptor.sha256 {
            return Err(BlobError::IntegrityMismatch {
                expected: descriptor.sha256.clone(),
                actual: actual_hash,
            });
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_store_cas_and_deduplication() {
        let temp_dir = std::env::temp_dir().join(format!("pds_blob_test_{}", BlobId::new()));
        let mut store = BlobStore::new(&temp_dir).unwrap();

        let data = b"Testowy plik binarny zawartosci PDS";
        let b1 = BlobId::new();
        let b2 = BlobId::new();

        let desc1 = store.put(b1, data, Some("text/plain".into())).unwrap();
        let desc2 = store.put(b2, data, Some("text/plain".into())).unwrap();

        assert_eq!(desc1.sha256, desc2.sha256);
        assert_eq!(desc1.size_bytes, data.len() as u64);

        let retrieved = store.get_data(&b1).unwrap();
        assert_eq!(retrieved, data);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
