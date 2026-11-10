# Changelog

All notable changes to the Personal Data Store (PDS) project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-11-10

### Added
- **Backward Compatibility Infrastructure (Phase 12)**:
  - Client simulator modules for `v1.0.0` and `v1.1.0` in `tests/compat/`.
  - Comprehensive backward compatibility integration suite in `tests/test_backward_compat.rs`.
- **Dynamic API Version Negotiation (Phase 15)**:
  - Dynamic API version resolution in `ClientAuthenticator` supporting `DEFAULT_SUPPORTED_API_VERSIONS` (`1.0.0`, `1.0.1`, `1.1.0`, `1.2.0`).
  - `ClientHello` extended with `supported_versions` for multi-version client capability reporting.
  - Active IPC `Session` now stores the agreed `api_version`.
- **Continuous Integration Pipeline (Phase 13)**:
  - GitHub Actions workflow `.github/workflows/ci.yml` running `cargo check`, strict clippy (`-D warnings`), and backward compatibility regression tests.
- **Forward Compatibility & Schema Mutation Validation (Phase 14)**:
  - `Operation::Unknown { action, payload }` variant preserving foreign and future action types without data loss during DAG replication.
  - `DatasetState::apply_operation` gracefully skips unknown operations during state projection.
  - `EntitySchema::validate_operation` for early pre-flight validation of entity property mutations against declared schemas.

### Fixed
- Added `#[serde(default)]` to `ChangeMetadata.encryption_mode` ensuring transparent deserialization of legacy `v1.0.x` payloads as `Plaintext`.
- Resolved all clippy warnings across library targets and integration tests.

## [1.1.0] - 2026-11-06

### Added
- **PDS-Client-API Local IPC & Authentication (Phase 9)**:
  - Ed25519 challenge-response handshake (`ClientHello`, `AuthChallenge`, `ClientAuth`, `AuthSuccess`).
  - Session manager with expiration, permission scoping and process ID binding (`IpcSessionManager`, `IpcServerHandler`).
  - Local IPC request/response types (`IpcRequest`, `IpcResponse`) for entity, relation, and session lifecycle operations.
- **PDS-NP Network Protocol (Phase 10)**:
  - Canonical network framing (`NpFrame`, `NpFrameHeader`) with magic header `PDSN`, protocol version and Ed25519 signatures.
  - Mutual node authentication handshake (`NpClientHello`, `NpServerHello`, `NpClientAuth`, `NpServerFinished`).
  - Domain-separated transcript hash and deterministic session ID derivation (`PDS-NP-SESSION`).
  - Peer identity trust management (`NodeIdentityId`, `NpNode`).
- **End-to-End Encryption (E2EE)**:
  - `EncryptionMode` supporting `Plaintext`, `EncryptedPayload`, and `BlindChange` in `ChangeMetadata`.
  - ChaCha20-Poly1305 AEAD authenticated encryption engine and in-memory `KeyStore`.
  - `KeyId` ULID-based identifier for cryptographic keys.
  - `PropertyValue::Encrypted` variant for encrypted entity attributes with schema validation support.
  - Cryptographic access permissions: `Permission::ReadCryptoKeys` and `Permission::ManageCryptoKeys`.
  - Updated specifications: [PDS.md](file:///home/tau2c/Projects/PDS/PDS.md), [PDS-Client-API.md](file:///home/tau2c/Projects/PDS/PDS-Client-API.md), and [PDS-NP.md](file:///home/tau2c/Projects/PDS/PDS-NP.md).
- **Comprehensive Integration Test Suite**:
  - `tests/test_core_model.rs`: Entity, relation, and operation variants.
  - `tests/test_dag_and_state.rs`: DAG history branching, merging, and state projection.
  - `tests/test_blob_and_storage.rs`: CAS blob storage deduplication and disk storage reload.
  - `tests/test_security_and_provenance.rs`: Access control, revocation, and activity provenance.
  - `tests/test_sync_and_archive.rs`: Sync protocol exchange and CBOR binary archive codec.
  - `tests/test_ipc_and_auth.rs`: Full IPC challenge-response authentication and entity operations.
  - `tests/test_np_protocol.rs`: PDS-NP frame verification, tamper detection, and transcript handshake.
  - `tests/test_e2ee_encryption.rs`: Encrypted payload lifecycle and blind change DAG replication.

### Changed
- Standardized binary serialization on CBOR (`ciborium`) with SHA-256 integrity checksum (`PDS1`).
- Replaced `spec.md` with modular architecture specifications: `PDS.md`, `PDS-Client-API.md`, and `PDS-NP.md`.

## [1.0.1] - 2026-11-06

### Added
- Examples in `examples/`: `basic_usage.rs`, `blob_storage.rs`, `sync_nodes.rs`.
- Comprehensive project documentation in `README.md`.

### Fixed
- Fixed non-ASCII byte string literals in blob example.
- Fixed `ciborium` serialization of complex enums.

## [1.0.0] - 2026-11-05

### Added
- Core data model: ULID identifiers (`EntityId`, `RelationId`, `BlobId`, `ChangeId`), `PropertyName`, `PropertyValue`, `Entity`, `Relation`.
- Append-only change model: `Operation`, `Change`, `ChangeMetadata`, `HistoryDag` with topological sorting.
- Dataset state projection: `DatasetState` materialization from DAG.
- Storage engine: Content-Addressed Storage `BlobStore` with SHA-256 deduplication and append-only `DiskStorage`.
- Provenance and schema engine: `Activity`, `ProvenanceTarget`, `EntitySchema` validation preserving open/unknown fields.
- Access control engine: `AccessController`, `AppRegistration`, granular `Permission`.
- Synchronization engine: `SyncEngine`, `SyncMessage` multi-head resolution.
- Archive engine: `PdsArchive` portability export/import.
- Binary codec: `BinaryCodec` with magic header and SHA-256 integrity check.
