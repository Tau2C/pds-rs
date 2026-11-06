use ed25519_dalek::SigningKey;
use pds::{
    create_client_auth_signature, BlobId, ClientAuth, ClientAuthenticator, ClientHello,
    DiskStorage, IpcRequest, IpcResponse, IpcServerHandler, IpcSessionManager, Permission,
    PropertyValue, API_VERSION,
};
use std::collections::BTreeSet;
use std::fs;

#[test]
fn test_ipc_authentication_and_unauthorized_rejections() {
    let temp_dir = std::env::temp_dir().join(format!("pds_test_ipc_{}", BlobId::new()));
    let mut storage = DiskStorage::open(&temp_dir).unwrap();
    let mut session_manager = IpcSessionManager::new();
    let mut authenticator = ClientAuthenticator::new("server-srv-1");

    // Register legitimate client
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let verifying_key = signing_key.verifying_key();

    let mut permissions = BTreeSet::new();
    permissions.insert(Permission::ReadEntities);
    permissions.insert(Permission::WriteEntities);

    let client_id = authenticator.register_client(
        "trusted_backup_client",
        verifying_key.to_bytes(),
    );
    authenticator.approve_client(&client_id, permissions);

    let mut handler =
        IpcServerHandler::new(&mut storage, &mut session_manager, &mut authenticator);

    // Step 1: Valid handshake flow
    let hello = ClientHello {
        client_id: client_id.clone(),
        api_version: API_VERSION.to_string(),
        client_nonce: [1u8; 32],
        capabilities: vec!["entities".to_string()],
    };
    let hello_resp = handler.handle_request(IpcRequest::Hello(hello), Some(9999));
    let challenge = match hello_resp {
        IpcResponse::Challenge(c) => c,
        other => panic!("Expected Challenge response, got {:?}", other),
    };

    let valid_sig = create_client_auth_signature(&signing_key, &challenge);
    let valid_auth = ClientAuth {
        client_id: client_id.clone(),
        signature: valid_sig,
    };
    let valid_resp = handler.handle_request(IpcRequest::Authenticate(valid_auth), Some(9999));
    let session_id = match valid_resp {
        IpcResponse::Authenticated(success) => success.session_id,
        other => panic!("Expected Authenticated, got {:?}", other),
    };

    // Step 2: Test rejection with forged/wrong key on a fresh challenge
    let hello_forged = ClientHello {
        client_id: client_id.clone(),
        api_version: API_VERSION.to_string(),
        client_nonce: [2u8; 32],
        capabilities: vec![],
    };
    let forged_challenge = match handler.handle_request(IpcRequest::Hello(hello_forged), Some(9999)) {
        IpcResponse::Challenge(c) => c,
        other => panic!("Expected Challenge response, got {:?}", other),
    };
    let wrong_key = SigningKey::from_bytes(&[8u8; 32]);
    let forged_sig = create_client_auth_signature(&wrong_key, &forged_challenge);
    let forged_auth = ClientAuth {
        client_id: client_id.clone(),
        signature: forged_sig,
    };
    let forged_resp = handler.handle_request(IpcRequest::Authenticate(forged_auth), Some(9999));
    assert!(matches!(forged_resp, IpcResponse::Error(_)));

    // Step 4: Perform operations with valid session
    let create_req = IpcRequest::CreateEntity {
        session_id: session_id.clone(),
        entity_type: Some("task".to_string()),
        properties: vec![("status".into(), PropertyValue::Text("pending".to_string()))],
    };
    let create_resp = handler.handle_request(create_req, Some(9999));
    let entity_id = match create_resp {
        IpcResponse::EntityCreated(id) => id,
        other => panic!("Expected EntityCreated, got {:?}", other),
    };

    let get_req = IpcRequest::GetEntity {
        session_id: session_id.clone(),
        entity_id,
    };
    let get_resp = handler.handle_request(get_req, Some(9999));
    match get_resp {
        IpcResponse::Entity(Some(entity)) => {
            assert_eq!(entity.entity_type.as_deref(), Some("task"));
        }
        other => panic!("Expected Entity, got {:?}", other),
    }

    // Step 5: Close session
    let close_resp = handler.handle_request(
        IpcRequest::CloseSession {
            session_id: session_id.clone(),
        },
        Some(9999),
    );
    assert_eq!(close_resp, IpcResponse::SessionClosed);

    // Step 6: Rejected after close
    let post_req = IpcRequest::GetEntity {
        session_id,
        entity_id,
    };
    let post_resp = handler.handle_request(post_req, Some(9999));
    assert!(matches!(post_resp, IpcResponse::Error(_)));

    let _ = fs::remove_dir_all(&temp_dir);
}
