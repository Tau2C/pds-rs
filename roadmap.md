# Personal Data Store (PDS) - Plan Wdrożenia (Roadmap)

Ten dokument opisuje etapy realizacji architektury PDS zdefiniowanej w specyfikacji ogólnej.

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

