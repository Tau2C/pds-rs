use crate::auth::{AuthChallenge, AuthSuccess, ClientAuth, ClientHello};
use crate::blob::BlobDescriptor;
use crate::change::Change;
use crate::entity::Entity;
use crate::id::{BlobId, ChangeId, EntityId, RelationId};
use crate::operation::Operation;
use crate::relation::Relation;
use crate::security::Permission;
use crate::storage::DiskStorage;
use crate::value::{PropertyName, PropertyValue};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum IpcRequest {
    Hello(ClientHello),
    Authenticate(ClientAuth),
    GetEntity {
        session_id: String,
        entity_id: EntityId,
    },
    CreateEntity {
        session_id: String,
        entity_type: Option<String>,
        properties: Vec<(PropertyName, PropertyValue)>,
    },
    GetRelation {
        session_id: String,
        relation_id: RelationId,
    },
    CreateRelation {
        session_id: String,
        relation_type: String,
        source: EntityId,
        target: EntityId,
    },
    GetBlob {
        session_id: String,
        blob_id: BlobId,
    },
    PutBlob {
        session_id: String,
        data: Vec<u8>,
        media_type: Option<String>,
    },
    PollChanges {
        session_id: String,
        since_change_id: Option<ChangeId>,
        limit: Option<usize>,
    },
    CloseSession {
        session_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload")]
pub enum IpcResponse {
    Challenge(AuthChallenge),
    Authenticated(AuthSuccess),
    Entity(Option<Entity>),
    EntityCreated(EntityId),
    Relation(Option<Relation>),
    RelationCreated(RelationId),
    Blob(Vec<u8>),
    BlobCreated(BlobDescriptor),
    Changes {
        changes: Vec<Change>,
        latest_change_id: Option<ChangeId>,
    },
    SessionClosed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub permissions: BTreeSet<Permission>,
    pub expires_at: DateTime<Utc>,
    pub peer_pid: Option<u32>,
    pub api_version: String,
}

#[derive(Default)]
pub struct IpcSessionManager {
    sessions: BTreeMap<String, Session>,
}

impl IpcSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
        }
    }

    pub fn insert_session(&mut self, session: Session) {
        self.sessions.insert(session.session_id.clone(), session);
    }

    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        let session = self.sessions.get(session_id)?;
        if Utc::now() > session.expires_at {
            return None;
        }
        Some(session)
    }

    pub fn remove_session(&mut self, session_id: &str) -> Option<Session> {
        self.sessions.remove(session_id)
    }

    pub fn check_permission(
        &self,
        session_id: &str,
        permission: &Permission,
    ) -> Result<(), &'static str> {
        let session = self.get_session(session_id).ok_or("Invalid or expired session")?;
        if !session.permissions.contains(permission) {
            return Err("Permission denied");
        }
        Ok(())
    }
}

pub struct IpcServerHandler<'a> {
    pub storage: &'a mut DiskStorage,
    pub sessions: &'a mut IpcSessionManager,
    pub authenticator: &'a mut crate::auth::ClientAuthenticator,
}

impl<'a> IpcServerHandler<'a> {
    pub fn new(
        storage: &'a mut DiskStorage,
        sessions: &'a mut IpcSessionManager,
        authenticator: &'a mut crate::auth::ClientAuthenticator,
    ) -> Self {
        Self {
            storage,
            sessions,
            authenticator,
        }
    }

    pub fn handle_request(&mut self, request: IpcRequest, peer_pid: Option<u32>) -> IpcResponse {
        match request {
            IpcRequest::Hello(hello) => match self.authenticator.handle_client_hello(&hello) {
                Ok(challenge) => IpcResponse::Challenge(challenge),
                Err(err) => IpcResponse::Error(err.to_string()),
            },
            IpcRequest::Authenticate(auth) => match self.authenticator.handle_client_auth(&auth) {
                Ok(success) => {
                    self.sessions.insert_session(Session {
                        session_id: success.session_id.clone(),
                        permissions: success.permissions.clone(),
                        expires_at: success.expires_at,
                        peer_pid,
                        api_version: success.api_version.clone(),
                    });
                    IpcResponse::Authenticated(success)
                }
                Err(err) => IpcResponse::Error(err.to_string()),
            },
            IpcRequest::GetEntity {
                session_id,
                entity_id,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::ReadEntities)
                {
                    return IpcResponse::Error(err.to_string());
                }
                let entity = self.storage.state().entities.get(&entity_id).cloned();
                IpcResponse::Entity(entity)
            }
            IpcRequest::CreateEntity {
                session_id,
                entity_type,
                properties,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::WriteEntities)
                {
                    return IpcResponse::Error(err.to_string());
                }

                let entity_id = EntityId::new();
                let mut ops = vec![Operation::CreateEntity {
                    entity_id,
                    entity_type,
                }];

                for (name, val) in properties {
                    ops.push(Operation::SetEntityProperty {
                        entity_id,
                        name,
                        value: val,
                    });
                }

                let parents: Vec<ChangeId> =
                    self.storage.dag().heads().iter().copied().collect();
                let meta = crate::change::ChangeMetadata::new("ipc-client");
                let change = crate::change::Change::new(ChangeId::new(), parents, meta, ops);

                if let Err(err) = self.storage.append_change(change) {
                    return IpcResponse::Error(err.to_string());
                }

                IpcResponse::EntityCreated(entity_id)
            }
            IpcRequest::GetRelation {
                session_id,
                relation_id,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::ReadEntities)
                {
                    return IpcResponse::Error(err.to_string());
                }
                let rel = self.storage.state().relations.get(&relation_id).cloned();
                IpcResponse::Relation(rel)
            }
            IpcRequest::CreateRelation {
                session_id,
                relation_type,
                source,
                target,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::WriteEntities)
                {
                    return IpcResponse::Error(err.to_string());
                }

                let relation_id = RelationId::new();
                let ops = vec![Operation::CreateRelation {
                    relation_id,
                    relation_type,
                    source,
                    target,
                }];

                let parents: Vec<ChangeId> =
                    self.storage.dag().heads().iter().copied().collect();
                let meta = crate::change::ChangeMetadata::new("ipc-client");
                let change = crate::change::Change::new(ChangeId::new(), parents, meta, ops);

                if let Err(err) = self.storage.append_change(change) {
                    return IpcResponse::Error(err.to_string());
                }

                IpcResponse::RelationCreated(relation_id)
            }
            IpcRequest::GetBlob {
                session_id,
                blob_id,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::ReadBlobs)
                {
                    return IpcResponse::Error(err.to_string());
                }
                match self.storage.blobs().get_data(&blob_id) {
                    Ok(data) => IpcResponse::Blob(data),
                    Err(err) => IpcResponse::Error(err.to_string()),
                }
            }
            IpcRequest::PutBlob {
                session_id,
                data,
                media_type,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::WriteBlobs)
                {
                    return IpcResponse::Error(err.to_string());
                }
                let blob_id = BlobId::new();
                match self.storage.blobs_mut().put(blob_id, &data, media_type) {
                    Ok(descriptor) => IpcResponse::BlobCreated(descriptor),
                    Err(err) => IpcResponse::Error(err.to_string()),
                }
            }
            IpcRequest::PollChanges {
                session_id,
                since_change_id,
                limit,
            } => {
                if let Err(err) =
                    self.sessions.check_permission(&session_id, &Permission::ReadHistory)
                {
                    return IpcResponse::Error(err.to_string());
                }
                let topological = self.storage.dag().topological_order();
                let mut start_collecting = since_change_id.is_none();
                let mut collected = Vec::new();
                let max_limit = limit.unwrap_or(100);

                for change in topological {
                    if start_collecting {
                        collected.push(change.clone());
                        if collected.len() >= max_limit {
                            break;
                        }
                    } else if since_change_id.as_ref() == Some(&change.id) {
                        start_collecting = true;
                    }
                }
                let latest_change_id = collected.last().map(|c| c.id);
                IpcResponse::Changes {
                    changes: collected,
                    latest_change_id,
                }
            }
            IpcRequest::CloseSession { session_id } => {
                self.sessions.remove_session(&session_id);
                IpcResponse::SessionClosed
            }
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IpcClientError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Session not established. Complete handshake first.")]
    NoActiveSession,
    #[error("Server returned error: {0}")]
    ServerError(String),
    #[error("Unexpected server response")]
    UnexpectedResponse,
}

pub struct PdsIpcClient {
    client_id: crate::auth::ClientId,
    signing_key: ed25519_dalek::SigningKey,
    session_id: Option<String>,
    api_version: String,
    negotiated_version: Option<String>,
}

impl PdsIpcClient {
    pub fn new(client_id: crate::auth::ClientId, signing_key: ed25519_dalek::SigningKey) -> Self {
        Self {
            client_id,
            signing_key,
            session_id: None,
            api_version: crate::auth::API_VERSION.to_string(),
            negotiated_version: None,
        }
    }

    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    pub fn client_id(&self) -> crate::auth::ClientId {
        self.client_id
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn negotiated_version(&self) -> Option<&str> {
        self.negotiated_version.as_deref()
    }

    pub fn start_handshake(&self, capabilities: Vec<String>) -> IpcRequest {
        let mut nonce = [0u8; 32];
        rand::rng().fill_bytes(&mut nonce);
        let hello = ClientHello::new(self.client_id, &self.api_version, nonce, capabilities);
        IpcRequest::Hello(hello)
    }

    pub fn handle_challenge(&mut self, challenge: &AuthChallenge) -> IpcRequest {
        let sig = crate::auth::create_client_auth_signature(&self.signing_key, challenge);
        IpcRequest::Authenticate(ClientAuth {
            client_id: self.client_id,
            signature: sig,
        })
    }

    pub fn handle_auth_success(&mut self, success: &AuthSuccess) {
        self.session_id = Some(success.session_id.clone());
        self.negotiated_version = Some(success.api_version.clone());
    }

    pub fn require_session(&self) -> Result<&str, IpcClientError> {
        self.session_id.as_deref().ok_or(IpcClientError::NoActiveSession)
    }

    pub fn get_entity_request(&self, entity_id: EntityId) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::GetEntity { session_id, entity_id })
    }

    pub fn create_entity_request(
        &self,
        entity_type: Option<String>,
        properties: Vec<(PropertyName, PropertyValue)>,
    ) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::CreateEntity {
            session_id,
            entity_type,
            properties,
        })
    }

    pub fn get_relation_request(&self, relation_id: RelationId) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::GetRelation { session_id, relation_id })
    }

    pub fn create_relation_request(
        &self,
        relation_type: String,
        source: EntityId,
        target: EntityId,
    ) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::CreateRelation {
            session_id,
            relation_type,
            source,
            target,
        })
    }

    pub fn get_blob_request(&self, blob_id: BlobId) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::GetBlob { session_id, blob_id })
    }

    pub fn put_blob_request(
        &self,
        data: Vec<u8>,
        media_type: Option<String>,
    ) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::PutBlob {
            session_id,
            data,
            media_type,
        })
    }

    pub fn poll_changes_request(
        &self,
        since_change_id: Option<ChangeId>,
        limit: Option<usize>,
    ) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        Ok(IpcRequest::PollChanges {
            session_id,
            since_change_id,
            limit,
        })
    }

    pub fn close_session_request(&mut self) -> Result<IpcRequest, IpcClientError> {
        let session_id = self.require_session()?.to_string();
        self.session_id = None;
        Ok(IpcRequest::CloseSession { session_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{create_client_auth_signature, ClientAuthenticator, API_VERSION};
    use ed25519_dalek::SigningKey;
    use rand::Rng;
    use std::fs;


    #[test]
    fn test_ipc_handler_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("pds_ipc_test_{}", EntityId::new()));
        let mut storage = DiskStorage::open(&temp_dir).unwrap();
        let mut sessions = IpcSessionManager::new();
        let mut authenticator = ClientAuthenticator::new("pds-local-test");

        let mut csprng = rand::rng();
        let mut secret = [0u8; 32];
        csprng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let client_id = authenticator.register_client("ShopLedger", signing_key.verifying_key().to_bytes());

        let mut perms = BTreeSet::new();
        perms.insert(Permission::ReadEntities);
        perms.insert(Permission::WriteEntities);
        authenticator.approve_client(&client_id, perms);

        let mut handler = IpcServerHandler::new(&mut storage, &mut sessions, &mut authenticator);

        let mut nonce = [0u8; 32];
        csprng.fill_bytes(&mut nonce);
        let hello = ClientHello::new(
            client_id,
            API_VERSION,
            nonce,
            vec!["ipc".into()],
        );

        let resp = handler.handle_request(IpcRequest::Hello(hello), Some(1234));
        let challenge = match resp {
            IpcResponse::Challenge(c) => c,
            other => panic!("Expected challenge, got {:?}", other),
        };

        let sig = create_client_auth_signature(&signing_key, &challenge);
        let auth = ClientAuth {
            client_id,
            signature: sig,
        };

        let auth_resp = handler.handle_request(IpcRequest::Authenticate(auth), Some(1234));
        let session_id = match auth_resp {
            IpcResponse::Authenticated(success) => success.session_id,
            other => panic!("Expected Authenticated, got {:?}", other),
        };

        // Create entity over IPC session
        let create_req = IpcRequest::CreateEntity {
            session_id: session_id.clone(),
            entity_type: Some("invoice".into()),
            properties: vec![("total".into(), PropertyValue::from(1500))],
        };
        let create_resp = handler.handle_request(create_req, Some(1234));
        let created_id = match create_resp {
            IpcResponse::EntityCreated(id) => id,
            other => panic!("Expected EntityCreated, got {:?}", other),
        };

        // Get entity over IPC session
        let get_req = IpcRequest::GetEntity {
            session_id: session_id.clone(),
            entity_id: created_id,
        };
        let get_resp = handler.handle_request(get_req, Some(1234));
        match get_resp {
            IpcResponse::Entity(Some(ent)) => {
                assert_eq!(ent.entity_type.as_deref(), Some("invoice"));
                assert_eq!(
                    ent.get_property(&"total".into()),
                    Some(&PropertyValue::from(1500))
                );
            }
            other => panic!("Expected Entity, got {:?}", other),
        }

        // Close session
        let close_resp = handler.handle_request(IpcRequest::CloseSession { session_id: session_id.clone() }, Some(1234));
        assert_eq!(close_resp, IpcResponse::SessionClosed);

        // Next request fails
        let post_close = handler.handle_request(
            IpcRequest::GetEntity {
                session_id,
                entity_id: created_id,
            },
            Some(1234),
        );
        assert!(matches!(post_close, IpcResponse::Error(_)));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_pds_ipc_client_full_flow() {
        let temp_dir = std::env::temp_dir().join(format!("pds_ipc_client_test_{}", EntityId::new()));
        let mut storage = DiskStorage::open(&temp_dir).unwrap();
        let mut sessions = IpcSessionManager::new();
        let mut authenticator = ClientAuthenticator::new("pds-local-node");

        let signing_key = SigningKey::from_bytes(&[42u8; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let client_id = authenticator.register_client("DesktopApp", public_key);

        let mut perms = BTreeSet::new();
        perms.insert(Permission::ReadEntities);
        perms.insert(Permission::WriteEntities);
        perms.insert(Permission::ReadBlobs);
        perms.insert(Permission::WriteBlobs);
        perms.insert(Permission::ReadHistory);
        authenticator.approve_client(&client_id, perms);

        let mut handler = IpcServerHandler::new(&mut storage, &mut sessions, &mut authenticator);
        let mut client = PdsIpcClient::new(client_id, signing_key);

        // 1. Handshake
        let hello_req = client.start_handshake(vec!["entities".into(), "blobs".into()]);
        let challenge = match handler.handle_request(hello_req, Some(5678)) {
            IpcResponse::Challenge(c) => c,
            other => panic!("Expected challenge, got {:?}", other),
        };

        let auth_req = client.handle_challenge(&challenge);
        let success = match handler.handle_request(auth_req, Some(5678)) {
            IpcResponse::Authenticated(s) => s,
            other => panic!("Expected authenticated, got {:?}", other),
        };
        client.handle_auth_success(&success);
        assert!(client.session_id().is_some());

        // 2. Put and Get Blob over IPC
        let blob_data = b"Hello from encrypted personal cloud blob!".to_vec();
        let put_blob_req = client
            .put_blob_request(blob_data.clone(), Some("text/plain".into()))
            .unwrap();
        let desc = match handler.handle_request(put_blob_req, Some(5678)) {
            IpcResponse::BlobCreated(d) => d,
            other => panic!("Expected BlobCreated, got {:?}", other),
        };

        let get_blob_req = client.get_blob_request(desc.id).unwrap();
        match handler.handle_request(get_blob_req, Some(5678)) {
            IpcResponse::Blob(retrieved) => assert_eq!(retrieved, blob_data),
            other => panic!("Expected Blob, got {:?}", other),
        }

        // 3. Create Entity and Poll Changes
        let create_req = client
            .create_entity_request(
                Some("note".into()),
                vec![("title".into(), PropertyValue::Text("My Note".into()))],
            )
            .unwrap();
        let entity_id = match handler.handle_request(create_req, Some(5678)) {
            IpcResponse::EntityCreated(id) => id,
            other => panic!("Expected EntityCreated, got {:?}", other),
        };

        let get_ent_req = client.get_entity_request(entity_id).unwrap();
        match handler.handle_request(get_ent_req, Some(5678)) {
            IpcResponse::Entity(Some(ent)) => {
                assert_eq!(ent.entity_type.as_deref(), Some("note"));
            }
            other => panic!("Expected Entity, got {:?}", other),
        }

        let poll_req = client.poll_changes_request(None, Some(10)).unwrap();
        match handler.handle_request(poll_req, Some(5678)) {
            IpcResponse::Changes { changes, latest_change_id } => {
                assert!(!changes.is_empty());
                assert!(latest_change_id.is_some());
            }
            other => panic!("Expected Changes, got {:?}", other),
        }

        // 4. Close Session
        let close_req = client.close_session_request().unwrap();
        assert_eq!(
            handler.handle_request(close_req, Some(5678)),
            IpcResponse::SessionClosed
        );
        assert!(client.session_id().is_none());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
