use crate::codec::BinaryCodec;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use ulid::Ulid;

pub const NP_MAGIC: &[u8; 4] = b"PDSN";
pub const NP_VERSION: &str = "1.0.0";
pub const NP_DOMAIN_SEPARATOR: &str = "PDS-NP-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeIdentityId(pub [u8; 32]);

impl NodeIdentityId {
    pub fn from_public_key(public_key: &[u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        Self(hasher.finalize().into())
    }
}

impl std::fmt::Display for NodeIdentityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(pub Ulid);

impl MessageId {
    pub fn new() -> Self {
        Self(Ulid::generate())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    HandshakeClientHello,
    HandshakeServerHello,
    HandshakeClientAuth,
    HandshakeServerFinished,
    SyncRequest,
    SyncResponse,
    BlobRequest,
    BlobResponse,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpFrameHeader {
    pub magic: [u8; 4],
    pub protocol_version: String,
    pub message_type: MessageType,
    pub session_id: [u8; 32],
    pub sequence: u64,
    pub message_id: MessageId,
    pub sender_id: NodeIdentityId,
    pub recipient_id: NodeIdentityId,
    pub payload_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpFrame {
    pub header: NpFrameHeader,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum NpError {
    #[error("Invalid magic: expected PDSN")]
    InvalidMagic,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Payload hash mismatch")]
    PayloadHashMismatch,
    #[error("Codec error: {0}")]
    Codec(String),
    #[error("Handshake error: {0}")]
    Handshake(String),
    #[error("Untrusted peer identity: {0}")]
    UntrustedPeer(NodeIdentityId),
}

impl NpFrame {
    pub fn create<T: Serialize>(
        signing_key: &SigningKey,
        sender_id: NodeIdentityId,
        recipient_id: NodeIdentityId,
        session_id: [u8; 32],
        sequence: u64,
        message_type: MessageType,
        payload_data: &T,
    ) -> Result<Self, NpError> {
        let serialized_payload =
            BinaryCodec::encode(payload_data).map_err(|e| NpError::Codec(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&serialized_payload);
        let payload_hash: [u8; 32] = hasher.finalize().into();

        let header = NpFrameHeader {
            magic: *NP_MAGIC,
            protocol_version: NP_VERSION.to_string(),
            message_type,
            session_id,
            sequence,
            message_id: MessageId::new(),
            sender_id,
            recipient_id,
            payload_hash,
        };

        let canonical_bytes = Self::canonical_header_bytes(&header);
        let signature = signing_key.sign(&canonical_bytes).to_bytes().to_vec();

        Ok(Self {
            header,
            payload: serialized_payload,
            signature,
        })
    }

    pub fn canonical_header_bytes(header: &NpFrameHeader) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(NP_DOMAIN_SEPARATOR.as_bytes());
        data.extend_from_slice(&header.magic);
        data.extend_from_slice(header.protocol_version.as_bytes());
        data.push(header.message_type as u8);
        data.extend_from_slice(&header.session_id);
        data.extend_from_slice(&header.sequence.to_be_bytes());
        data.extend_from_slice(header.message_id.0.to_bytes().as_ref());
        data.extend_from_slice(&header.sender_id.0);
        data.extend_from_slice(&header.recipient_id.0);
        data.extend_from_slice(&header.payload_hash);
        data
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<(), NpError> {
        if self.header.magic != *NP_MAGIC {
            return Err(NpError::InvalidMagic);
        }

        let mut hasher = Sha256::new();
        hasher.update(&self.payload);
        let computed_hash: [u8; 32] = hasher.finalize().into();
        if computed_hash != self.header.payload_hash {
            return Err(NpError::PayloadHashMismatch);
        }

        let canonical_bytes = Self::canonical_header_bytes(&self.header);
        let sig = Signature::from_slice(&self.signature).map_err(|_| NpError::InvalidSignature)?;

        verifying_key
            .verify(&canonical_bytes, &sig)
            .map_err(|_| NpError::InvalidSignature)?;

        Ok(())
    }

    pub fn decode_payload<T: DeserializeOwned>(&self) -> Result<T, NpError> {
        BinaryCodec::decode(&self.payload).map_err(|e| NpError::Codec(e.to_string()))
    }
}

// Handshake structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpClientHello {
    pub identity_id: NodeIdentityId,
    pub public_key: [u8; 32],
    pub nonce: [u8; 32],
    pub supported_versions: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpServerHello {
    pub identity_id: NodeIdentityId,
    pub public_key: [u8; 32],
    pub nonce: [u8; 32],
    pub client_hello_hash: [u8; 32],
    pub chosen_version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpClientAuth {
    pub identity_id: NodeIdentityId,
    pub transcript_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpServerFinished {
    pub identity_id: NodeIdentityId,
    pub transcript_hash: [u8; 32],
}

pub struct NpNode {
    pub signing_key: SigningKey,
    pub identity_id: NodeIdentityId,
    pub trusted_peers: BTreeSet<NodeIdentityId>,
}

impl NpNode {
    pub fn new() -> Self {
        let mut secret = [0u8; 32];
        rand::rng().fill(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key = signing_key.verifying_key().to_bytes();
        let identity_id = NodeIdentityId::from_public_key(&public_key);

        Self {
            signing_key,
            identity_id,
            trusted_peers: BTreeSet::new(),
        }
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    pub fn trust_peer(&mut self, peer_id: NodeIdentityId) {
        self.trusted_peers.insert(peer_id);
    }

    pub fn is_trusted(&self, peer_id: &NodeIdentityId) -> bool {
        self.trusted_peers.contains(peer_id)
    }

    pub fn compute_session_id(transcript_hash: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"PDS-NP-SESSION");
        hasher.update(transcript_hash);
        hasher.finalize().into()
    }
}

impl Default for NpNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_np_frame_signing_and_verification() {
        let node_a = NpNode::new();
        let node_b = NpNode::new();

        let payload_msg = vec!["sync".to_string(), "heads".to_string()];
        let session_id = [1u8; 32];

        let frame = NpFrame::create(
            &node_a.signing_key,
            node_a.identity_id,
            node_b.identity_id,
            session_id,
            1,
            MessageType::SyncRequest,
            &payload_msg,
        )
        .unwrap();

        assert!(frame.verify(&node_a.signing_key.verifying_key()).is_ok());

        let decoded: Vec<String> = frame.decode_payload().unwrap();
        assert_eq!(decoded, payload_msg);

        // Verification fails with wrong key
        assert!(frame.verify(&node_b.signing_key.verifying_key()).is_err());
    }

    #[test]
    fn test_np_handshake_session_derivation() {
        let node_a = NpNode::new();
        let mut node_b = NpNode::new();
        node_b.trust_peer(node_a.identity_id);

        let mut nonce_a = [0u8; 32];
        rand::rng().fill(&mut nonce_a);

        let client_hello = NpClientHello {
            identity_id: node_a.identity_id,
            public_key: node_a.public_key(),
            nonce: nonce_a,
            supported_versions: vec![NP_VERSION.to_string()],
            capabilities: vec!["sync".into(), "blobs".into()],
        };

        assert!(node_b.is_trusted(&client_hello.identity_id));

        let mut hasher = Sha256::new();
        hasher.update(&node_a.public_key());
        hasher.update(&nonce_a);
        let transcript: [u8; 32] = hasher.finalize().into();

        let session_id_a = NpNode::compute_session_id(&transcript);
        let session_id_b = NpNode::compute_session_id(&transcript);

        assert_eq!(session_id_a, session_id_b);
        assert_ne!(session_id_a, [0u8; 32]);
    }
}
