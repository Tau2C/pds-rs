//! v1.1.0 client simulator.
//!
//! This module produces JSON payloads that match the wire format used by
//! PDS v1.1.0 clients. The `ChangeMetadata` struct in that version includes
//! the `encryption_mode` field introduced with the E2EE feature.

use pds::{ChangeMetadata, EncryptionMode, KeyId};
use serde_json::Value;

/// A v1.1.0 client that sends plaintext changes.
pub fn plaintext_metadata(device_id: impl Into<String>) -> ChangeMetadata {
    ChangeMetadata::new(device_id)
}

/// A v1.1.0 client that sends changes with EncryptedPayload mode.
pub fn encrypted_payload_metadata(
    device_id: impl Into<String>,
    key_id: KeyId,
    nonce: [u8; 12],
) -> ChangeMetadata {
    ChangeMetadata::new(device_id)
        .with_encryption_mode(EncryptionMode::EncryptedPayload { key_id, nonce })
}

/// A v1.1.0 client that sends BlindChange changes.
pub fn blind_change_metadata(
    device_id: impl Into<String>,
    key_id: KeyId,
    nonce: [u8; 12],
    auth_tag: [u8; 16],
) -> ChangeMetadata {
    ChangeMetadata::new(device_id)
        .with_encryption_mode(EncryptionMode::BlindChange { key_id, nonce, auth_tag })
}

/// Returns `true` if the JSON payload contains the `encryption_mode` field.
pub fn has_encryption_mode_field(json: &str) -> bool {
    let v: Value = serde_json::from_str(json).expect("valid JSON");
    v.get("encryption_mode").is_some()
}
