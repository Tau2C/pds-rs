use crate::auth::{AuthChallenge, AuthSuccess, ClientAuth, ClientHello};
use crate::entity::Entity;
use crate::id::{ChangeId, EntityId, RelationId};
use crate::operation::Operation;
use crate::relation::Relation;
use crate::security::Permission;
use crate::storage::DiskStorage;
use crate::value::{PropertyName, PropertyValue};
use chrono::{DateTime, Utc};
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
    SessionClosed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub permissions: BTreeSet<Permission>,
    pub expires_at: DateTime<Utc>,
    pub peer_pid: Option<u32>,
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
            IpcRequest::CloseSession { session_id } => {
                self.sessions.remove_session(&session_id);
                IpcResponse::SessionClosed
            }
        }
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
        let hello = ClientHello {
            client_id,
            api_version: API_VERSION.to_string(),
            client_nonce: nonce,
            capabilities: vec!["ipc".into()],
        };

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
}
