# Personal Data Store (PDS) - Plan Wdrożenia (Roadmap)

Ten dokument opisuje etapy realizacji architektury PDS zdefiniowanej w specyfikacjach:
- [`PDS.md`](PDS.md) — Ogólna specyfikacja architektury danych
- [`PDS-Client-API.md`](PDS-Client-API.md) — Interfejs dostępu aplikacji (In-process oraz Local IPC)
- [`PDS-NP.md`](PDS-NP.md) — Protokół komunikacji sieciowej PDS ↔ PDS

## Faza 1: Podstawowy model danych (Core Data Model) [Zakończona]


Cel: Zapewnienie struktur w pamięci i operacji na podstawowych elementach grafu danych.

1. **Identyfikatory i typy podstawowe** [Zrealizowano]:
   - Definicja uniwersalnych identyfikatorów (`EntityId`, `RelationId`, `BlobId`, `ChangeId` na bazie ULID).
   - Implementacja typów wartości właściwości (`PropertyValue`).
2. **Encje i właściwości** [Zrealizowano]:
   - Struktura encji z otwartym modelem właściwości (`Entity`).
   - Przestrzenie nazw dla właściwości i typów (`PropertyName`).
3. **Relacje** [Zrealizowano]:
   - Struktura relacji jako element pierwszej klasy (`Relation`).
   - Właściwości przypisane do relacji.
4. **Testy jednostkowe** [Zrealizowano]:
   - Weryfikacja tworzenia, serializacji i deserializacji struktur danych.


## Faza 2: Model historii i operacji (Append-Only Change Model) [Zakończona]

Cel: Rejestrowanie zmian w postaci niemutowalnego dziennika i rekonstrukcja stanu.

1. **Jednostki zmian (Change Units)** [Zrealizowano]:
   - Definicja atomowych operacji (`Operation`).
   - Struktura zatwierdzonej zmiany z metadanymi (`Change`, `ChangeMetadata`).
2. **Graf historii (DAG)** [Zrealizowano]:
   - Wiązanie zmian ze stanami poprzednimi (`HistoryDag`).
   - Wykrywanie rozgałęzień historii oraz sortowanie topologiczne.
3. **Rekonstrukcja stanu (Materialization)** [Zrealizowano]:
   - Aplikowanie sekwencji zmian w celu uzyskania bieżącego widoku (`DatasetState`).
   - Projekcja encji i relacji z grafu zmian DAG.


## Faza 3: Trwały magazyn danych i obiekty binarne (Storage & Blob Engine) [Zakończona]

Cel: Trwały zapis historii, indeksów oraz plików binarnych na dysku.

1. **Magazyn obiektów binarnych (Content-Addressable Storage)** [Zrealizowano]:
   - Zapis i odczyt plików na podstawie skrótów SHA-256 (`BlobStore`).
   - Weryfikacja integralności obiektów oraz automatyczna deduplikacja.
2. **Silnik zapisu historii** [Zrealizowano]:
   - Trwałe składowanie jednostek zmian w formacie JSON Lines (`DiskStorage`).
   - Zachowywanie porządku dopisywania (append-only) i odczyt przy uruchomieniu.
3. **Indeksy lokalne** [Zrealizowano]:
   - Szybki odczyt bieżących encji i relacji poprzez stan w pamięci (`DatasetState`).
   - Izolacja indeksów od trwałego źródła prawdy.


## Faza 4: Pochodzenie danych i schematy (Provenance & Schemas) [Zakończona]

Cel: Rejestrowanie kontekstu powstawania danych oraz opcjonalna walidacja.

1. **Model pochodzenia (Provenance / Activities)** [Zrealizowano]:
   - Rejestracja aktywności przetwarzania (`Activity`, `ActivityStatus`, `ProvenanceTarget`).
   - Łączenie wyników ze źródłami bez usuwania danych wejściowych.
2. **Obsługa schematów** [Zrealizowano]:
   - Wersjonowane definicje schematów (`EntitySchema`, `PropertyDefinition`, `PropertyType`).
   - Walidacja typów z zachowywaniem nieznanych właściwości bez utraty danych.


## Faza 5: Bezpieczeństwo i kontrola dostępu (Security & Access Control) [Zakończona]

Cel: Ochrona danych użytkownika i izolacja aplikacji.

1. **Tożsamość i zaufanie** [Zrealizowano]:
   - Zarządzanie zaufanymi urządzeniami i tożsamościami (`AccessController`).
   - Rejestracja, uwierzytelnianie i autoryzacja aplikacji klienckich (`AppRegistration`).
2. **Granice uprawnień** [Zrealizowano]:
   - Jawne reguły dostępu (`Permission`: encje, pliki binarne, historia, uprawnienia).
   - Egzekwowanie i natychmiastowe unieważnianie uprawnień oraz aplikacji.
3. **Opcjonalne szyfrowanie**:
   - Przygotowanie fundamentów pod szyfrowanie danych w spoczynku.


## Faza 6: Synchronizacja rozproszona i praca offline (Sync Engine) [Zakończona]

Cel: Wymiana danych między urządzeniami offline-first.

1. **Wymiana historii** [Zrealizowano]:
   - Protokół wykrywania brakujących węzłów w grafie DAG (`SyncMessage`, `SyncEngine`).
   - Transfer jednostek zmian i synchronizacja bez konieczności centralnego serwera.
2. **Obsługa konfliktów** [Zrealizowano]:
   - Wykrywanie rozgałęzień (wielu `heads`) w grafie DAG.
   - Zachowywanie alternatyw i bezkonfliktowe dołączanie niezależnych zmian.


## Faza 7: Eksport, kopie zapasowe i integracja (Backup & Client API) [Zakończona]

Cel: Przenośność danych oraz publiczny interfejs dla aplikacji.

1. **Format eksportu i importu** [Zrealizowano]:
   - Wersjonowany format archiwum przestrzeni danych (`PdsArchive`, `PdsArchiveMetadata`).
   - Weryfikacja integralności i serializacja do formatu JSON.
2. **Interfejs klienta** [Zrealizowano]:
   - Publiczne API biblioteki dla zewnętrznych aplikacji (`PdsClient`).
   - Integracja z kontrolą uprawnień i trwałym magazynem danych.

## Faza 8: Format binarny (Binary Codec) [Zakończona]

Cel: Wydajne kodowanie i dekodowanie struktur danych z kontrolą integralności.

1. **Format ramki binarnej** [Zrealizowano]:
   - Nagłówek `PDS1` (4 bajty).
   - Suma kontrolna SHA-256 payloadu (32 bajty).
   - Bezpieczna serializacja payloadu w standardzie CBOR (`ciborium`).
2. **Koder i dekoder** [Zrealizowano]:
   - Uniwersalny koder `BinaryCodec::encode` oraz `BinaryCodec::decode`.
   - Wykrywanie modyfikacji danych i uszkodzeń sumy kontrolnej.

## Faza 9: Pełna implementacja PDS-Client-API (Local IPC & Protocol Handshake) [Zakończona]

Cel: Wdrożenie kompletnego interfejsu klienta zgodnego ze specyfikacją `PDS-Client-API.md`.

1. **Kryptograficzna rejestracja i tożsamość klienta** [Zrealizowano]:
   - Klucze Ed25519 dla aplikacji klienckich (`SigningKey`, `VerifyingKey`).
   - Rejestracja tożsamości `ClientId` (ULID) ze stanem (`Pending`, `Active`, `Suspended`, `Revoked`).
   - Zatwierdzanie rejestracji i nadawanie granularnych uprawnień.
2. **Protokół uwierzytelniania IPC (Challenge-Response)** [Zrealizowano]:
   - Sekwencja komunikatów `ClientHello` -> `AuthChallenge` -> `ClientAuth` -> `AuthSuccess`.
   - Podpis kryptograficzny `Ed25519.Sign` z unikalnymi nonces (ochrona przed powtórzeniem).
   - Obsługa kontekstu procesu (`peer_pid`).
3. **Zarządzanie sesjami i uprawnieniami** [Zrealizowano]:
   - Obsługa sesji `IpcSessionManager` z kryptograficznym `session_id` i czasem wygaśnięcia.
   - Serwer obsługi żądań `IpcServerHandler` (operacje na encjach, relacjach, sesjach).


## Faza 10: Protokół sieciowy PDS-NP (Network Protocol PDS ↔ PDS)

Cel: Wdrożenie protokołu komunikacji międzyinstancyjnej zgodnego ze specyfikacją `PDS-NP.md`.

1. **Tożsamość kryptograficzna węzłów**:
   - Para kluczy Ed25519 instancji i odcisk `identity_id = SHA256(public_key)`.
   - Model wzajemnego zaufania instancji PDS.
2. **Warstwa transportowa i szyfrowanie**:
   - Adapter transportowy (domyślnie `iroh` QUIC / WireGuard).
   - Obowiązkowe szyfrowanie i podpisywanie każdej ramki protokołu.
3. **Protokół synchronizacji PDS-NP**:
   - Negocjacja wersji, wymiana nagłówków DAG i transfer brakujących jednostek zmian.
   - Strumieniowy transfer i wznawianie pobierania dużych blobów.

