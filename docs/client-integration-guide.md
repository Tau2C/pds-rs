# PDS Client Integration Guide

Przewodnik integracji aplikacji zewnętrznych z lokalnym Personal Data Store (PDS).

---

## 1. Architektura komunikacji

PDS umożliwia dwustronną komunikację z aplikacjami na dwa sposoby:
1. **In-Process API (`PdsClient`)**: Dla aplikacji osadzonych w tym samym procesie Rust.
2. **Local IPC (`PdsIpcClient` / Unix Domain Socket)**: Dla niezależnych procesów systemowych (aplikacje desktopowe, mobilne, demony pomocnicze, wtyczki w Pythonie, TypeScript, Go, C# itp.).

Komunikacja IPC bazuje na formacie JSON (lub binarnym CBOR) przesyłanym w ramkach przez deskryptor gniazda lokalnego:
`/run/user/<UID>/pds.sock`

---

## 2. Tożsamość i rejestracja klienta

Każdy klient identyfikuje się unikalną parą kluczy Ed25519 oraz identyfikatorem `ClientId` (ULID).

1. Klient generuje klucze:
   - Klucz prywatny: `32 bajty` (przechowywany w bezpiecznym magazynie aplikacji).
   - Klucz publiczny: `32 bajty`.
2. Administrator rejestruje klienta w PDS:
   - Zgłoszenie: `register_client("NazwaAplikacji", public_key) -> ClientId`.
   - Nadanie uprawnień: `approve_client(&client_id, permissions)`.

### Zakresy uprawnień (`Permission`)
- `entities.read` (`Permission::ReadEntities`): Odczyt encji i relacji.
- `entities.write` (`Permission::WriteEntities`): Tworzenie i modyfikacja encji i relacji.
- `blobs.read` (`Permission::ReadBlobs`): Odczyt plików z CAS (Content-Addressed Storage).
- `blobs.write` (`Permission::WriteBlobs`): Zapis plików binarnych do CAS.
- `history.read` (`Permission::ReadHistory`): Przeglądanie historii zmian i polling DAG.
- `crypto.read` (`Permission::ReadCryptoKeys`): Odczyt kluczy E2EE.
- `crypto.manage` (`Permission::ManageCryptoKeys`): Generowanie i rotacja kluczy E2EE.

---

## 3. Protokół Handshake i uwierzytelnianie

Handshake składa się z 4 kroków zapobiegających atakom powtórzeniowym (replay attacks):

```text
Klient                                      Serwer PDS
  |                                              |
  |--- 1. ClientHello -------------------------->|
  |<-- 2. AuthChallenge (z server_nonce) --------|
  |                                              |
  | [Podpis Ed25519 payloadu wyzwania]           |
  |--- 3. ClientAuth (signature) --------------->|
  |<-- 4. AuthSuccess (session_id) --------------|
```

### Krok 1: `ClientHello`
```json
{
  "action": "Hello",
  "payload": {
    "client_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "api_version": "1.2.0",
    "client_nonce": [1, 2, 3, ...],
    "capabilities": ["entities", "blobs", "history"],
    "supported_versions": ["1.0.0", "1.1.0", "1.2.0"]
  }
}
```

### Krok 2: `AuthChallenge`
Serwer negocjuje najwyższą wspólną wersję API i odsyła wyzwanie:
```json
{
  "status": "Challenge",
  "payload": {
    "server_instance_id": "pds-node-alpha",
    "client_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "client_nonce": [1, 2, 3, ...],
    "server_nonce": [45, 67, ...],
    "api_version": "1.2.0",
    "challenge_expires_at": "2026-11-10T14:45:00Z"
  }
}
```

### Krok 3: Wyliczenie podpisu i `ClientAuth`
Podpis generowany jest dla połączonych bajtów:
`"PDS-CLIENT-AUTH-v1" || server_instance_id || client_id || client_nonce || server_nonce || api_version`

```json
{
  "action": "Authenticate",
  "payload": {
    "client_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "signature": [ ... 64 bajty podpisu Ed25519 ... ]
  }
}
```

### Krok 4: `AuthSuccess`
```json
{
  "status": "Authenticated",
  "payload": {
    "session_id": "a9f3b2c1...",
    "client_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "permissions": ["ReadEntities", "WriteEntities", "ReadBlobs", "WriteBlobs"],
    "expires_at": "2026-11-11T14:45:00Z",
    "api_version": "1.2.0"
  }
}
```

---

## 4. Przykłady operacji IPC

Każde kolejne żądanie przesyła otrzymany `session_id`.

### Tworzenie encji (`CreateEntity`)
```json
{
  "action": "CreateEntity",
  "payload": {
    "session_id": "a9f3b2c1...",
    "entity_type": "note",
    "properties": [
      ["title", { "Text": "Notatka ze spotkania" }],
      ["priority", { "Int": 1 }]
    ]
  }
}
```
**Odpowiedź:**
```json
{
  "status": "EntityCreated",
  "payload": "01ARZ3NDEKTSV4RRFFQ69G5XYZ"
}
```

### Zapis pliku binarnego (`PutBlob`)
```json
{
  "action": "PutBlob",
  "payload": {
    "session_id": "a9f3b2c1...",
    "data": [72, 101, 108, 108, 111],
    "media_type": "text/plain"
  }
}
```
**Odpowiedź:**
```json
{
  "status": "BlobCreated",
  "payload": {
    "id": "01ARZ3NDEKTSV4RRFFQ69GB10B",
    "size_bytes": 5,
    "sha256": "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969",
    "mime_type": "text/plain"
  }
}
```

### Śledzenie zmian (`PollChanges`)
Reaktywne odpytywanie o nowe zmiany w grafie DAG od zadanego identyfikatora:
```json
{
  "action": "PollChanges",
  "payload": {
    "session_id": "a9f3b2c1...",
    "since_change_id": "01ARZ3NDEKTSV4RRFFQ69G5PREV",
    "limit": 50
  }
}
```
**Odpowiedź:**
```json
{
  "status": "Changes",
  "payload": {
    "changes": [ ... ],
    "latest_change_id": "01ARZ3NDEKTSV4RRFFQ69G5LATEST"
  }
}
```

---

## 5. Implementacja w Rust (`PdsIpcClient`)

W aplikacjach Rust wystarczy skorzystać ze struktury `PdsIpcClient` dostarczanej przez bibliotekę:

```rust
use pds::{PdsIpcClient, IpcRequest, IpcResponse, PropertyValue};
use ed25519_dalek::SigningKey;

// 1. Inicjalizacja klienta z tożsamością
let mut client = PdsIpcClient::new(client_id, signing_key);

// 2. Rozpoczęcie handshake
let hello_msg = client.start_handshake(vec!["entities".into(), "blobs".into()]);

// 3. Po otrzymaniu AuthChallenge z serwera:
let auth_msg = client.handle_challenge(&challenge);

// 4. Po otrzymaniu AuthSuccess z serwera:
client.handle_auth_success(&auth_success);

// 5. Wykonywanie operacji biznesowych
let req = client.create_entity_request(
    Some("task".into()),
    vec![("summary".into(), PropertyValue::Text("Kupić mleko".into()))],
)?;
```
