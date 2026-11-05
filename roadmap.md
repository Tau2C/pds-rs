# Personal Data Store (PDS) - Plan Wdrożenia (Roadmap)

Ten dokument opisuje etapy realizacji architektury PDS zdefiniowanej w specyfikacji ogólnej.

## Faza 1: Podstawowy model danych (Core Data Model)

Cel: Zapewnienie struktur w pamięci i operacji na podstawowych elementach grafu danych.

1. **Identyfikatory i typy podstawowe**:
   - Definicja uniwersalnych identyfikatorów (`EntityId`, `RelationId`, `BlobId`, `ChangeId`).
   - Implementacja typów wartości właściwości (`PropertyValue`).
2. **Encje i właściwości**:
   - Struktura encji z otwartym modelem właściwości (`Entity`).
   - Przestrzenie nazw dla właściwości i typów.
3. **Relacje**:
   - Struktura relacji jako element pierwszej klasy (`Relation`).
   - Właściwości przypisane do relacji.
4. **Testy jednostkowe**:
   - Weryfikacja tworzenia, serializacji i deserializacji struktur danych.

## Faza 2: Model historii i operacji (Append-Only Change Model)

Cel: Rejestrowanie zmian w postaci niemutowalnego dziennika i rekonstrukcja stanu.

1. **Jednostki zmian (Change Units)**:
   - Definicja atomowych operacji (tworzenie, aktualizacja właściwości, usunięcie logiczne).
   - Struktura zatwierdzonej zmiany z metadanymi (autor, urządzenie, znacznik czasu).
2. **Graf historii (DAG)**:
   - Wiązanie zmian ze stanami poprzednimi (rodzice w grafie DAG).
   - Wykrywanie rozgałęzień historii.
3. **Rekonstrukcja stanu (Materialization)**:
   - Aplikowanie sekwencji zmian w celu uzyskania bieżącego widoku encji.
   - Odtwarzanie stanu encji z określonego punktu w czasie.

## Faza 3: Trwały magazyn danych i obiekty binarne (Storage & Blob Engine)

Cel: Trwały zapis historii, indeksów oraz plików binarnych na dysku.

1. **Magazyn obiektów binarnych (Content-Addressable Storage)**:
   - Zapis i odczyt plików na podstawie skrótów kryptograficznych (deduplikacja).
   - Weryfikacja integralności obiektów.
2. **Silnik zapisu historii**:
   - Trwałe składowanie jednostek zmian (baza osadzona lub format plikowy).
   - Ochrona transakcyjna i integralność zapisu.
3. **Indeksy lokalne**:
   - Szybki odczyt bieżących encji i relacji.
   - Izolacja indeksów od trwałego źródła prawdy.

## Faza 4: Pochodzenie danych i schematy (Provenance & Schemas)

Cel: Rejestrowanie kontekstu powstawania danych oraz opcjonalna walidacja.

1. **Model pochodzenia (Provenance / Activities)**:
   - Rejestracja aktywności przetwarzania (wejścia, narzędzia, wersje, wyjścia).
   - Łączenie wyników ze źródłami bez usuwania danych wejściowych.
2. **Obsługa schematów**:
   - Wersjonowane definicje schematów (walidacja typów i pól).
   - Bezpieczne zachowywanie nieznanych pól i ewolucja schematów bez utraty danych.

## Faza 5: Bezpieczeństwo i kontrola dostępu (Security & Access Control)

Cel: Ochrona danych użytkownika i izolacja aplikacji.

1. **Tożsamość i zaufanie**:
   - Zarządzanie tożsamością właściciela i zaufanymi urządzeniami.
   - Rejestracja i autoryzacja aplikacji klienckich.
2. **Granice uprawnień**:
   - Jawne reguły dostępu (zakres encji, typów, historii i plików).
   - Egzekwowanie i cofanie uprawnień.
3. **Opcjonalne szyfrowanie**:
   - Szyfrowanie danych w spoczynku i zarządzanie kluczami.

## Faza 6: Synchronizacja rozproszona i praca offline (Sync Engine)

Cel: Wymiana danych między urządzeniami offline-first.

1. **Wymiana historii**:
   - Protokół wykrywania brakujących węzłów w grafie DAG.
   - Transfer jednostek zmian i obiektów binarnych.
2. **Obsługa konfliktów**:
   - Wykrywanie równoległych modyfikacji.
   - Strategie łączenia zmian i zachowywanie alternatyw.

## Faza 7: Eksport, kopie zapasowe i integracja (Backup & Client API)

Cel: Przenośność danych oraz publiczny interfejs dla aplikacji.

1. **Format eksportu i importu**:
   - Wersjonowany format archiwum przestrzeni danych.
   - Weryfikacja integralności i raportowanie brakujących zależności.
2. **Interfejs klienta**:
   - Publiczne API biblioteki dla zewnętrznych aplikacji.
   - Testy integracyjne z symulacją wielu niezależnych aplikacji.
