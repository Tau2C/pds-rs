use pds::{
    BinaryCodec, BlobDescriptor, BlobId, Change, ChangeId, ChangeMetadata, EntityId,
    HistoryDag, Operation, PdsArchive, SyncEngine, SyncMessage,
};

#[test]
fn test_sync_between_two_disconnected_nodes() {
    let mut node_a_dag = HistoryDag::new();
    let mut node_b_dag = HistoryDag::new();

    // Node A creates common genesis
    let genesis_id = ChangeId::new();
    let genesis = Change::new(
        genesis_id,
        vec![],
        ChangeMetadata::new("device_a"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("folder".to_string()),
        }],
    );
    node_a_dag.insert(genesis.clone()).unwrap();
    node_b_dag.insert(genesis).unwrap();

    // Node A creates a change
    let a1_id = ChangeId::new();
    let a1 = Change::new(
        a1_id,
        vec![genesis_id],
        ChangeMetadata::new("device_a"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("file_a".to_string()),
        }],
    );
    node_a_dag.insert(a1).unwrap();

    // Node B creates an independent concurrent change
    let b1_id = ChangeId::new();
    let b1 = Change::new(
        b1_id,
        vec![genesis_id],
        ChangeMetadata::new("device_b"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("file_b".to_string()),
        }],
    );
    node_b_dag.insert(b1).unwrap();

    // Node A requests sync from Node B
    let req_from_a = SyncEngine::create_request(&node_a_dag);
    let known_by_a = match req_from_a {
        SyncMessage::SyncRequest { known_heads } => known_heads,
        _ => panic!("Expected SyncRequest"),
    };

    let resp_from_b = SyncEngine::handle_request(&node_b_dag, &known_by_a);
    let missing_for_a = match resp_from_b {
        SyncMessage::SyncResponse { missing_changes, .. } => missing_changes,
        _ => panic!("Expected SyncResponse"),
    };

    let applied_on_a = SyncEngine::apply_response(&mut node_a_dag, missing_for_a).unwrap();
    assert_eq!(applied_on_a, 1);
    assert_eq!(node_a_dag.len(), 3);

    // Node B requests sync from Node A
    let req_from_b = SyncEngine::create_request(&node_b_dag);
    let known_by_b = match req_from_b {
        SyncMessage::SyncRequest { known_heads } => known_heads,
        _ => panic!("Expected SyncRequest"),
    };

    let resp_from_a = SyncEngine::handle_request(&node_a_dag, &known_by_b);
    let missing_for_b = match resp_from_a {
        SyncMessage::SyncResponse { missing_changes, .. } => missing_changes,
        _ => panic!("Expected SyncResponse"),
    };

    let applied_on_b = SyncEngine::apply_response(&mut node_b_dag, missing_for_b).unwrap();
    assert_eq!(applied_on_b, 1);
    assert_eq!(node_b_dag.len(), 3);

    // Both DAGs now have identical heads
    assert_eq!(node_a_dag.heads(), node_b_dag.heads());
}

#[test]
fn test_archive_export_import_and_binary_codec() {
    let c_id = ChangeId::new();
    let change = Change::new(
        c_id,
        vec![],
        ChangeMetadata::new("exporter"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("note".to_string()),
        }],
    );

    let blob_desc = BlobDescriptor {
        id: BlobId::new(),
        sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        size_bytes: 0,
        mime_type: Some("application/octet-stream".to_string()),
    };

    let archive = PdsArchive::new(vec![change], vec![], vec![blob_desc]);
    let json_dump = archive.to_json().expect("Failed to serialize archive to JSON");
    let restored_from_json = PdsArchive::from_json(&json_dump).expect("Failed to restore from JSON");
    assert_eq!(restored_from_json.metadata.change_count, 1);
    assert_eq!(restored_from_json.metadata.blob_count, 1);

    // Binary codec encoding with SHA-256 checksum and magic header
    let encoded_bytes = BinaryCodec::encode(&archive).expect("CBOR encoding failed");
    assert_eq!(&encoded_bytes[..4], b"PDS1");

    let restored_from_binary: PdsArchive =
        BinaryCodec::decode(&encoded_bytes).expect("CBOR decoding failed");
    assert_eq!(restored_from_binary.metadata.change_count, 1);
    assert_eq!(restored_from_binary.changes[0].id, c_id);
}
