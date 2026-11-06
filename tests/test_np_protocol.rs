use pds::{
    MessageType, NpClientAuth, NpClientHello, NpFrame, NpNode, NpServerFinished, NpServerHello,
    NP_MAGIC, NP_VERSION,
};
use sha2::{Digest, Sha256};

#[test]
fn test_np_frame_roundtrip_tamper_detection() {
    let sender = NpNode::new();
    let receiver = NpNode::new();

    let session_id = [42u8; 32];
    let payload = vec!["request_chunk_1".to_string(), "request_chunk_2".to_string()];

    let mut frame = NpFrame::create(
        &sender.signing_key,
        sender.identity_id,
        receiver.identity_id,
        session_id,
        1,
        MessageType::SyncRequest,
        &payload,
    )
    .expect("Failed to create frame");

    assert_eq!(&frame.header.magic, NP_MAGIC);
    assert_eq!(frame.header.protocol_version, NP_VERSION);
    assert_eq!(frame.header.sender_id, sender.identity_id);
    assert_eq!(frame.header.recipient_id, receiver.identity_id);
    assert_eq!(frame.header.sequence, 1);

    // Verify successfully with sender public key
    assert!(frame.verify(&sender.signing_key.verifying_key()).is_ok());

    // Payload decoding
    let decoded_payload: Vec<String> = frame.decode_payload().expect("Failed to decode payload");
    assert_eq!(decoded_payload, payload);

    // Tampering test: modify 1 byte of payload
    frame.payload[0] ^= 0xFF;
    assert!(frame.verify(&sender.signing_key.verifying_key()).is_err());
}

#[test]
fn test_np_handshake_transcript_and_mutual_trust() {
    let client = NpNode::new();
    let mut server = NpNode::new();

    // Verify peer untrusted by default
    assert!(!server.is_trusted(&client.identity_id));

    // Trust client node
    server.trust_peer(client.identity_id);
    assert!(server.is_trusted(&client.identity_id));

    let client_nonce = [11u8; 32];
    let server_nonce = [22u8; 32];

    let client_hello = NpClientHello {
        identity_id: client.identity_id,
        public_key: client.public_key(),
        nonce: client_nonce,
        supported_versions: vec![NP_VERSION.to_string()],
        capabilities: vec!["sync".into(), "cas".into()],
    };

    let mut client_hello_hasher = Sha256::new();
    client_hello_hasher.update(&client_hello.public_key);
    client_hello_hasher.update(&client_hello.nonce);
    let client_hello_hash: [u8; 32] = client_hello_hasher.finalize().into();

    let server_hello = NpServerHello {
        identity_id: server.identity_id,
        public_key: server.public_key(),
        nonce: server_nonce,
        client_hello_hash,
        chosen_version: NP_VERSION.to_string(),
        capabilities: vec!["sync".into(), "cas".into()],
    };

    let mut hasher = Sha256::new();
    hasher.update(&client_hello.public_key);
    hasher.update(&client_hello.nonce);
    hasher.update(&server_hello.nonce);
    let transcript: [u8; 32] = hasher.finalize().into();

    let client_session = NpNode::compute_session_id(&transcript);
    let server_session = NpNode::compute_session_id(&transcript);
    assert_eq!(client_session, server_session);

    let client_auth = NpClientAuth {
        identity_id: client.identity_id,
        transcript_hash: transcript,
    };

    let server_finished = NpServerFinished {
        identity_id: server.identity_id,
        transcript_hash: transcript,
    };

    assert_eq!(client_auth.identity_id, client.identity_id);
    assert_eq!(server_finished.identity_id, server.identity_id);
    assert_eq!(client_auth.transcript_hash, server_finished.transcript_hash);
}
