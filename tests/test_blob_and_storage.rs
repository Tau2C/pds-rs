use pds::{
    BlobId, BlobStore, Change, ChangeId, ChangeMetadata, DiskStorage, EntityId, Operation,
    PropertyValue,
};
use std::fs;

#[test]
fn test_blob_store_deduplication_and_retrieval() {
    let temp_path = std::env::temp_dir().join(format!("pds_test_blob_{}", BlobId::new()));
    let mut store = BlobStore::new(&temp_path).unwrap();

    let data = b"Hello, decentralized world of personal data!";
    let id1 = BlobId::new();
    let desc1 = store
        .put(id1, data, Some("text/plain".to_string()))
        .expect("Put blob 1 failed");

    assert_eq!(desc1.id, id1);
    assert_eq!(desc1.size_bytes, data.len() as u64);
    assert_eq!(desc1.mime_type.as_deref(), Some("text/plain"));

    // Put identical content under different BlobId -> CAS deduplication
    let id2 = BlobId::new();
    let desc2 = store
        .put(id2, data, Some("text/plain".to_string()))
        .expect("Put blob 2 failed");

    assert_eq!(desc2.sha256, desc1.sha256);
    assert_eq!(desc2.id, id2);

    let read_back1 = store.get_data(&id1).expect("Failed to get blob 1");
    let read_back2 = store.get_data(&id2).expect("Failed to get blob 2");
    assert_eq!(read_back1, data);
    assert_eq!(read_back2, data);

    let non_existent = BlobId::new();
    assert!(store.get_data(&non_existent).is_err());

    let _ = fs::remove_dir_all(&temp_path);
}

#[test]
fn test_disk_storage_reopen_and_persistence() {
    let root = std::env::temp_dir().join(format!("pds_test_storage_{}", BlobId::new()));
    let entity_id = EntityId::new();
    let c1_id = ChangeId::new();

    {
        let mut storage = DiskStorage::open(&root).expect("Failed to open storage");
        let change = Change::new(
            c1_id,
            vec![],
            ChangeMetadata::new("device-node-1").with_author("test_author"),
            vec![
                Operation::CreateEntity {
                    entity_id,
                    entity_type: Some("document".to_string()),
                },
                Operation::SetEntityProperty {
                    entity_id,
                    name: "title".into(),
                    value: PropertyValue::Text("My Notes".to_string()),
                },
            ],
        );

        storage.append_change(change).expect("Failed to append change");
        assert_eq!(storage.change_count(), 1);

        let blob_id = BlobId::new();
        storage
            .blobs_mut()
            .put(blob_id, b"binary payload", None)
            .expect("Failed to store blob");
    }

    // Reopen from disk
    {
        let storage = DiskStorage::open(&root).expect("Failed to reopen storage");
        assert_eq!(storage.change_count(), 1);
        let restored_change = storage.get_change(&c1_id).expect("Change not found");
        assert_eq!(restored_change.metadata.author_id.as_deref(), Some("test_author"));
        assert_eq!(restored_change.metadata.device_id, "device-node-1");

        let entity = storage
            .state()
            .entities
            .get(&entity_id)
            .expect("Entity not projected");
        assert_eq!(entity.entity_type.as_deref(), Some("document"));
        assert_eq!(
            entity.get_property(&"title".into()),
            Some(&PropertyValue::Text("My Notes".to_string()))
        );
    }

    let _ = fs::remove_dir_all(&root);
}
