use pds::{
    Change, ChangeId, ChangeMetadata, DatasetState, EncryptionMode, EntityId, HistoryDag,
    KeyStore, Operation, PropertyName, PropertyValue,
};

#[test]
fn test_e2ee_encrypted_payload_property_lifecycle() {
    let mut keystore = KeyStore::new();
    let (key_id, _) = keystore.generate_key();

    let mut dag = HistoryDag::new();
    let entity_id = EntityId::new();
    let c1_id = ChangeId::new();

    // Sensitive data to encrypt
    let sensitive_health_data = b"Patient: Jan Kowalski, Diagnosis: Healthy, BP: 120/80";
    let encrypted_data = keystore.encrypt(&key_id, sensitive_health_data).unwrap();

    let change = Change::new(
        c1_id,
        vec![],
        ChangeMetadata::new("medical_device_1")
            .with_encryption_mode(EncryptionMode::EncryptedPayload {
                key_id,
                nonce: encrypted_data.nonce,
            }),
        vec![
            Operation::CreateEntity {
                entity_id,
                entity_type: Some("health_record".to_string()),
            },
            Operation::SetEntityProperty {
                entity_id,
                name: "medical.record".into(),
                value: PropertyValue::Encrypted {
                    key_id,
                    ciphertext: encrypted_data.ciphertext.clone(),
                },
            },
            Operation::SetEntityProperty {
                entity_id,
                name: "record.category".into(),
                value: PropertyValue::Text("cardiology".to_string()),
            },
        ],
    );

    dag.insert(change).expect("Failed to insert encrypted change into DAG");

    // Project state
    let state = DatasetState::from_dag(&dag);
    let entity = state.entities.get(&entity_id).expect("Entity not found in state");

    // Plaintext property visible
    assert_eq!(
        entity.get_property(&PropertyName::from("record.category")),
        Some(&PropertyValue::Text("cardiology".to_string()))
    );

    // Encrypted property present but opaque
    let enc_prop = entity.get_property(&PropertyName::from("medical.record")).unwrap();
    match enc_prop {
        PropertyValue::Encrypted { key_id: prop_key_id, ciphertext } => {
            assert_eq!(*prop_key_id, key_id);
            assert_eq!(ciphertext, &encrypted_data.ciphertext);

            // Authorized client decrypts payload using KeyStore
            let decrypted = keystore.decrypt(&encrypted_data).unwrap();
            assert_eq!(decrypted, sensitive_health_data);
        }
        other => panic!("Expected PropertyValue::Encrypted, got {:?}", other),
    }
}

#[test]
fn test_e2ee_blind_change_dag_replication() {
    let mut keystore = KeyStore::new();
    let (key_id, _) = keystore.generate_key();

    let mut dag = HistoryDag::new();
    let c1_id = ChangeId::new();

    // Prepare blind change: operations encrypted into opaque ciphertext block
    let raw_operations = b"Blind change operations binary representation";
    let encrypted_block = keystore.encrypt(&key_id, raw_operations).unwrap();

    let blind_change = Change::new(
        c1_id,
        vec![],
        ChangeMetadata::new("anonymized_node").with_encryption_mode(EncryptionMode::BlindChange {
            key_id,
            nonce: encrypted_block.nonce,
            auth_tag: [0u8; 16],
        }),
        vec![], // Untrusted nodes see empty operations vector in the graph
    );

    dag.insert(blind_change).expect("Relay failed to insert blind change");
    assert_eq!(dag.heads().iter().copied().collect::<Vec<_>>(), vec![c1_id]);

    // Untrusted node replicates DAG structure and child links without seeing payload
    let c2_id = ChangeId::new();
    let child_change = Change::new(
        c2_id,
        vec![c1_id],
        ChangeMetadata::new("child_node"),
        vec![],
    );
    dag.insert(child_change).expect("Relay failed to connect child change");
    assert_eq!(dag.len(), 2);
}
