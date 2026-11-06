use crate::security::Permission;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use ulid::Ulid;

pub const API_VERSION: &str = "1.1.0";
pub const AUTH_CONTEXT_TAG: &str = "PDS-CLIENT-AUTH-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub Ulid);

impl ClientId {
    pub fn new() -> Self {
        Self(Ulid::generate())
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientRecord {
    pub client_id: ClientId,
    pub name: String,
    pub public_key: [u8; 32],
    pub status: ClientStatus,
    pub permissions: BTreeSet<Permission>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientHello {
    pub client_id: ClientId,
    pub api_version: String,
    pub client_nonce: [u8; 32],
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub server_instance_id: String,
    pub client_id: ClientId,
    pub client_nonce: [u8; 32],
    pub server_nonce: [u8; 32],
    pub api_version: String,
    pub challenge_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientAuth {
    pub client_id: ClientId,
    pub signature: Vec<u8>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSuccess {
    pub session_id: String,
    pub client_id: ClientId,
    pub permissions: BTreeSet<Permission>,
    pub expires_at: DateTime<Utc>,
    pub api_version: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("Client not found: {0}")]
    ClientNotFound(ClientId),
    #[error("Client is not active (status: {0:?})")]
    ClientNotActive(ClientStatus),
    #[error("Challenge expired")]
    ChallengeExpired,
    #[error("Invalid challenge nonce or parameters")]
    InvalidChallenge,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unsupported API version: expected {expected}, got {actual}")]
    UnsupportedApiVersion { expected: String, actual: String },
}

pub struct ClientAuthenticator {
    server_instance_id: String,
    clients: BTreeMap<ClientId, ClientRecord>,
    active_challenges: BTreeMap<ClientId, AuthChallenge>,
}

impl ClientAuthenticator {
    pub fn new(server_instance_id: impl Into<String>) -> Self {
        Self {
            server_instance_id: server_instance_id.into(),
            clients: BTreeMap::new(),
            active_challenges: BTreeMap::new(),
        }
    }

    pub fn register_client(
        &mut self,
        name: impl Into<String>,
        public_key: [u8; 32],
    ) -> ClientId {
        let client_id = ClientId::new();
        let record = ClientRecord {
            client_id,
            name: name.into(),
            public_key,
            status: ClientStatus::Pending,
            permissions: BTreeSet::new(),
            created_at: Utc::now(),
        };
        self.clients.insert(client_id, record);
        client_id
    }

    pub fn approve_client(&mut self, client_id: &ClientId, permissions: BTreeSet<Permission>) {
        if let Some(client) = self.clients.get_mut(client_id) {
            client.status = ClientStatus::Active;
            client.permissions = permissions;
        }
    }

    pub fn get_client(&self, client_id: &ClientId) -> Option<&ClientRecord> {
        self.clients.get(client_id)
    }

    pub fn handle_client_hello(&mut self, hello: &ClientHello) -> Result<AuthChallenge, AuthError> {
        if hello.api_version != API_VERSION {
            return Err(AuthError::UnsupportedApiVersion {
                expected: API_VERSION.to_string(),
                actual: hello.api_version.clone(),
            });
        }

        let client = self
            .clients
            .get(&hello.client_id)
            .ok_or(AuthError::ClientNotFound(hello.client_id))?;

        if client.status != ClientStatus::Active {
            return Err(AuthError::ClientNotActive(client.status.clone()));
        }

        let mut server_nonce = [0u8; 32];
        rand::rng().fill_bytes(&mut server_nonce);

        let challenge = AuthChallenge {
            server_instance_id: self.server_instance_id.clone(),
            client_id: hello.client_id,
            client_nonce: hello.client_nonce,
            server_nonce,
            api_version: API_VERSION.to_string(),
            challenge_expires_at: Utc::now() + Duration::seconds(60),
        };

        self.active_challenges.insert(hello.client_id, challenge.clone());
        Ok(challenge)
    }

    pub fn compute_signing_payload(
        server_instance_id: &str,
        client_id: &ClientId,
        client_nonce: &[u8; 32],
        server_nonce: &[u8; 32],
        api_version: &str,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(AUTH_CONTEXT_TAG.as_bytes());
        data.extend_from_slice(server_instance_id.as_bytes());
        data.extend_from_slice(client_id.0.to_bytes().as_ref());
        data.extend_from_slice(client_nonce);
        data.extend_from_slice(server_nonce);
        data.extend_from_slice(api_version.as_bytes());
        data
    }

    pub fn handle_client_auth(&mut self, auth: &ClientAuth) -> Result<AuthSuccess, AuthError> {
        let challenge = self
            .active_challenges
            .remove(&auth.client_id)
            .ok_or(AuthError::InvalidChallenge)?;

        if Utc::now() > challenge.challenge_expires_at {
            return Err(AuthError::ChallengeExpired);
        }

        let client = self
            .clients
            .get(&auth.client_id)
            .ok_or(AuthError::ClientNotFound(auth.client_id))?;

        if client.status != ClientStatus::Active {
            return Err(AuthError::ClientNotActive(client.status.clone()));
        }

        let payload = Self::compute_signing_payload(
            &challenge.server_instance_id,
            &challenge.client_id,
            &challenge.client_nonce,
            &challenge.server_nonce,
            &challenge.api_version,
        );

        let verifying_key = VerifyingKey::from_bytes(&client.public_key)
            .map_err(|_| AuthError::InvalidSignature)?;
        let signature = Signature::from_slice(&auth.signature)
            .map_err(|_| AuthError::InvalidSignature)?;

        verifying_key
            .verify(&payload, &signature)
            .map_err(|_| AuthError::InvalidSignature)?;

        let mut session_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut session_bytes);
        let session_id = hex::encode(session_bytes);

        Ok(AuthSuccess {
            session_id,
            client_id: auth.client_id,
            permissions: client.permissions.clone(),
            expires_at: Utc::now() + Duration::hours(24),
            api_version: challenge.api_version,
        })
    }
}

pub fn create_client_auth_signature(
    signing_key: &SigningKey,
    challenge: &AuthChallenge,
) -> Vec<u8> {
    let payload = ClientAuthenticator::compute_signing_payload(
        &challenge.server_instance_id,
        &challenge.client_id,
        &challenge.client_nonce,
        &challenge.server_nonce,
        &challenge.api_version,
    );
    let signature = signing_key.sign(&payload);
    signature.to_bytes().to_vec()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_challenge_response_handshake() {
        let mut server = ClientAuthenticator::new("pds-local-node-1");

        let mut csprng = rand::rng();
        let mut secret = [0u8; 32];
        csprng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key = signing_key.verifying_key().to_bytes();

        let client_id = server.register_client("ShopApp", public_key);

        let mut perms = BTreeSet::new();
        perms.insert(Permission::ReadEntities);
        perms.insert(Permission::WriteEntities);
        server.approve_client(&client_id, perms.clone());

        let mut client_nonce = [0u8; 32];
        csprng.fill_bytes(&mut client_nonce);
        let hello = ClientHello {
            client_id,
            api_version: API_VERSION.to_string(),
            client_nonce,
            capabilities: vec!["ipc".into()],
        };

        let challenge = server.handle_client_hello(&hello).unwrap();
        assert_eq!(challenge.client_id, client_id);

        let sig_bytes = create_client_auth_signature(&signing_key, &challenge);
        let auth = ClientAuth {
            client_id,
            signature: sig_bytes,
        };

        let success = server.handle_client_auth(&auth).unwrap();
        assert_eq!(success.client_id, client_id);
        assert_eq!(success.permissions, perms);
        assert_eq!(success.session_id.len(), 64);
    }
}
