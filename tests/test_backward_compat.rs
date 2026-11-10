//! Backward compatibility tests (Phase 12).
//!
//! These tests verify that the current PDS server correctly handles
//! data produced by older client versions, and that newer clients
//! can still read data written by older clients.

mod compat;

use pds::{Change, ChangeId, ChangeMetadata, EncryptionMode, KeyId};

// ── helpers ─────────────────────────────────────────────────────────────────

/// Builds a full `Change` JSON string using the v1.0.0 wire format
/// (no `encryption_mode` field).
fn v100_change_json(change_id: ChangeId, device_id: &str) -> String {
    let legacy_meta = compat::client_v1_0_0::LegacyChangeMetadataV100::new(device_id)
        .with_author("alice")
        .with_message("legacy change");

    // Build the full Change JSON manually so the metadata uses legacy format.
    serde_json::json!({
        "id": change_id,
        "parents": [],
        "metadata": serde_json::to_value(&legacy_meta).unwrap(),
        "operations": []
    })
    .to_string()
}

// ── tests ────────────────────────────────────────────────────────────────────

/// A v1.0.0 JSON payload (no `encryption_mode`) must deserialize into a
/// current `Change` with `EncryptionMode::Plaintext`.
#[test]
fn v100_change_deserializes_as_plaintext() {
    let id = ChangeId::new();
    let json = v100_change_json(id, "device-old");

    let change: Change = serde_json::from_str(&json)
        .expect("v1.0.0 Change JSON must deserialize without error");

    assert_eq!(change.id, id);
    assert_eq!(
        change.metadata.encryption_mode,
        EncryptionMode::Plaintext,
        "missing encryption_mode field must default to Plaintext"
    );
}

/// A v1.0.0 JSON payload must NOT contain the `encryption_mode` key.
/// This verifies the simulator itself is faithful to the old wire format.
#[test]
fn v100_metadata_has_no_encryption_mode_field() {
    let meta = compat::client_v1_0_0::LegacyChangeMetadataV100::new("device-old");
    let json = meta.to_json();

    assert!(
        !json.contains("encryption_mode"),
        "v1.0.0 wire format must not contain encryption_mode; got: {json}"
    );
}

/// A v1.1.0 JSON payload (with `encryption_mode: Plaintext`) must
/// deserialize correctly.
#[test]
fn v110_plaintext_change_round_trips() {
    let meta = compat::client_v1_1_0::plaintext_metadata("device-new");
    let json = serde_json::to_string(&meta).unwrap();

    let decoded: ChangeMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.encryption_mode, EncryptionMode::Plaintext);
}

/// A v1.1.0 `EncryptedPayload` change round-trips through JSON without data loss.
#[test]
fn v110_encrypted_payload_round_trips() {
    let key_id = KeyId::new();
    let nonce = [1u8; 12];
    let meta =
        compat::client_v1_1_0::encrypted_payload_metadata("device-new", key_id, nonce);

    let json = serde_json::to_string(&meta).unwrap();
    let decoded: ChangeMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decoded.encryption_mode,
        EncryptionMode::EncryptedPayload {
            key_id,
            nonce,
        }
    );
}

/// A v1.1.0 `BlindChange` change round-trips through JSON without data loss.
#[test]
fn v110_blind_change_round_trips() {
    let key_id = KeyId::new();
    let nonce = [2u8; 12];
    let auth_tag = [3u8; 16];
    let meta = compat::client_v1_1_0::blind_change_metadata(
        "device-new",
        key_id,
        nonce,
        auth_tag,
    );

    let json = serde_json::to_string(&meta).unwrap();
    let decoded: ChangeMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decoded.encryption_mode,
        EncryptionMode::BlindChange {
            key_id,
            nonce,
            auth_tag,
        }
    );
}

/// v1.1.0 wire format must include the `encryption_mode` field.
#[test]
fn v110_metadata_has_encryption_mode_field() {
    let meta = compat::client_v1_1_0::plaintext_metadata("device-new");
    let json = serde_json::to_string(&meta).unwrap();

    assert!(
        compat::client_v1_1_0::has_encryption_mode_field(&json),
        "v1.1.0 wire format must contain encryption_mode; got: {json}"
    );
}

/// A v1.0.0 JSON payload rehydrated by the current server must be
/// re-serialized as a v1.1.0 payload (with `encryption_mode` present).
#[test]
fn v100_change_reserializes_with_encryption_mode() {
    let id = ChangeId::new();
    let json = v100_change_json(id, "device-old");

    let change: Change = serde_json::from_str(&json).unwrap();
    let reserialied = serde_json::to_string(&change.metadata).unwrap();

    assert!(
        compat::client_v1_1_0::has_encryption_mode_field(&reserialied),
        "rehydrated v1.0.0 change must serialize with encryption_mode; got: {reserialied}"
    );
}

/// A Change containing future/unknown operations must deserialize and preserve
/// those operations for DAG sync while allowing local DatasetState projection.
#[test]
fn test_change_with_unknown_operation_projection_and_reserialization() {
    let raw_json = serde_json::json!({
        "id": ChangeId::new(),
        "parents": [],
        "metadata": {
            "author_id": "future-user",
            "device_id": "future-device",
            "timestamp": "2026-11-10T12:00:00Z",
            "message": "future operations test",
            "encryption_mode": "Plaintext"
        },
        "operations": [
            {
                "action": "QuantumEncryptionKeyRotate",
                "payload": {
                    "key_algorithm": "Kyber1024",
                    "slot": 1
                }
            }
        ]
    })
    .to_string();

    let change: Change = serde_json::from_str(&raw_json)
        .expect("Change with unknown operation must deserialize safely");

    assert_eq!(change.operations.len(), 1);
    match &change.operations[0] {
        pds::Operation::Unknown { action, payload } => {
            assert_eq!(action, "QuantumEncryptionKeyRotate");
            assert_eq!(payload["key_algorithm"], "Kyber1024");
        }
        other => panic!("Expected Operation::Unknown, got {:?}", other),
    }

    // Materialize state: must not fail or panic
    let mut state = pds::DatasetState::new();
    state.apply_change(&change);

    // Re-serialize: must keep the exact same unknown operation payload
    let reserialized = serde_json::to_string(&change).unwrap();
    let re_decoded: Change = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(change, re_decoded);
}
