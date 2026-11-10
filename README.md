# Personal Data Store (PDS)

PDS to uniwersalny, należący do użytkownika magazyn danych w architekturze *local-first* i *offline-first*.
Umożliwia niezależnym aplikacjom współdzielenie spójnego grafu encji, relacji, historii zmian i plików binarnych.

## Główne założenia

1. **Własność użytkownika**: Dane nie są powiązane z cyklem życia pojedynczej aplikacji.
2. **Historia typu Append-Only**: Wszystkie modyfikacje tworzą skierowany graf acykliczny (DAG).
3. **Pochodzenie danych (Provenance)**: Rejestracja procesów wytwarzających dane i ich powiązań.
4. **Magazyn binarny CAS**: Obiekty deduplikowane na podstawie skrótów SHA-256.
5. **Jawne uprawnienia**: Granularna kontrola dostępu dla aplikacji klienckich.
6. **Synchronizacja P2P**: Wymiana brakujących węzłów DAG bez konieczności centralnego serwera.

## Architektura modułów

- [`id`](src/id.rs): Identyfikatory oparte na standardzie ULID (`EntityId`, `RelationId`, `BlobId`, `ChangeId`).
- [`value`](src/value.rs): Typy właściwości (`PropertyValue`) oraz nazwy z obsługą przestrzeni nazw (`PropertyName`).
- [`entity`](src/entity.rs): Struktura encji z dynamicznymi właściwościami.
- [`relation`](src/relation.rs): Relacje jako elementy pierwszej klasy łączące encje.
- [`operation`](src/operation.rs): Atomowe operacje modyfikujące graf danych.
- [`change`](src/change.rs): Niemutowalne jednostki zmian z metadanymi i rodzicami w grafie DAG.
- [`dag`](src/dag.rs): Zarządzanie skierowanym grafem historii zmian i sortowanie topologiczne.
- [`state`](src/state.rs): Projekcja bieżącego stanu (`DatasetState`) odtwarzana z historii.
- [`blob`](src/blob.rs): Magazyn obiektów binarnych Content-Addressable Storage (CAS).
- [`storage`](src/storage.rs): Trwały magazyn plikowy (`DiskStorage`) w formacie append-only.
- [`schema`](src/schema.rs): Wersjonowane schematy walidujące dane bez utraty nieznanych pól.
- [`provenance`](src/provenance.rs): Śledzenie aktywności i pochodzenia danych (`Activity`).
- [`security`](src/security.rs): Kontroler uprawnień i rejestracja aplikacji (`AccessController`).
- [`sync`](src/sync.rs): Protokół wymiany historii i synchronizacji węzłów (`SyncEngine`).
- [`archive`](src/archive.rs): Eksport i odtwarzanie przestrzeni danych (`PdsArchive`).
- [`client`](src/client.rs): Bezpieczny punkt wejścia dla aplikacji klienckich (`PdsClient`).
- [`codec`](src/codec.rs): Binarny koder ramek CBOR z 4-bajtowym nagłówkiem magii i sumą kontrolną SHA-256.

## Przykłady uruchomieniowe

W katalogu `examples/` znajdują się przykłady użycia:

1. **Podstawowe operacje na encjach i relacjach**:
   ```bash
   cargo run --example basic_usage
   ```

2. **Zapis i deduplikacja obiektów binarnych**:
   ```bash
   cargo run --example blob_storage
   ```

3. **Synchronizacja historii DAG między niezależnymi węzłami**:
   ```bash
   cargo run --example sync_nodes
   ```

## Dokumentacja i integracja

- [Przewodnik integracji klienta (`docs/client-integration-guide.md`)](docs/client-integration-guide.md) — Kompletny opis protokołu handshake IPC, operacji na encjach, blobów i subskrypcji zdarzeń.
- [Architektura danych PDS (`PDS.md`)](PDS.md) — Główna specyfikacja modelu danych i DAG.
- [Interfejs klienta (`PDS-Client-API.md`)](PDS-Client-API.md) — Specyfikacja komunikacji in-process oraz IPC.
- [Protokół sieciowy (`PDS-NP.md`)](PDS-NP.md) — Specyfikacja komunikacji między węzłami PDS ↔ PDS.

## Uruchamianie testów

```bash
cargo test
```
