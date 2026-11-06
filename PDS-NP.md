# PDS-NP — Protokół komunikacji między instancjami PDS

**Wersja:** 1.0.0  
**Status:** Projekt  
**Zakres:** Komunikacja PDS ↔ PDS

## 1. Cel i zakres

PDS-NP (PDS Network Protocol) definiuje sposób komunikacji pomiędzy niezależnymi instancjami Personal Data Store (PDS).

Protokół określa:

- ustanawianie i zamykanie sesji;
- wzajemne uwierzytelnianie instancji PDS;
- negocjację wersji i możliwości protokołu;
- format i semantykę wiadomości;
- wymianę historii i danych synchronizacyjnych;
- transfer plików (blobów);
- potwierdzanie odbioru i obsługę błędów;
- wymagania dotyczące poufności, integralności i autentyczności komunikacji;
- ochronę przed powtórzeniem i modyfikacją wiadomości.

PDS-NP jest przeznaczony wyłącznie do komunikacji pomiędzy instancjami PDS.

Dostęp aplikacji do lokalnej instancji PDS jest definiowany przez odrębny interfejs **PDS-Client-API**. Komunikacja in-process i lokalne IPC nie należą do zakresu PDS-NP.

## 2. Zasady architektoniczne

1. Protokół jest niezależny od konkretnej technologii transportowej.
2. Transport odpowiada za dostarczanie danych, natomiast PDS-NP definiuje znaczenie i reguły wymiany wiadomości.
3. Obie strony muszą wzajemnie uwierzytelnić swoje tożsamości PDS.
4. Cała komunikacja musi być szyfrowana.
5. Każda wiadomość protokołu musi być podpisana przez nadawcę.
6. Uwierzytelnienie peera nie oznacza automatycznego przyznania mu uprawnień.
7. Synchronizacja nie może po cichu nadpisywać historii ani usuwać konfliktujących danych.
8. Protokół nie udostępnia niezabezpieczonego trybu komunikacji.
9. Błąd negocjacji lub uwierzytelniania musi zakończyć się przerwaniem połączenia, a nie przejściem na słabszy mechanizm bezpieczeństwa.
10. Tożsamość PDS jest niezależna od tożsamości transportowej.

## 3. Architektura

PDS-NP jest częścią warstwy komunikacyjnej PDS.

| Warstwa | Odpowiedzialność |
|---|---|
| PDS Core | Model danych, historia, uprawnienia i operacje na danych |
| Silnik synchronizacji | Ustalanie zakresu wymiany, wykrywanie zmian i obsługa konfliktów |
| PDS-NP | Handshake, wersjonowanie, format i semantyka wiadomości |
| Warstwa bezpieczeństwa | Uwierzytelnianie, podpisy, wiązanie sesji i kontrola dostępu |
| Adapter transportowy | Nawiązywanie połączenia i bezpieczne dostarczanie ramek |

### 3.1 Relacja z PDS-Client-API

PDS-NP i PDS-Client-API są odrębnymi interfejsami:

- **PDS-Client-API** – aplikacja kliencka ↔ lokalna instancja PDS.
- **PDS-NP** – lokalna instancja PDS ↔ inna instancja PDS.

Lokalny klient nie jest peerem PDS-NP. Nie przechodzi handshake PDS-NP i nie musi posiadać sieciowej tożsamości PDS.

Klient może zlecić lokalnej instancji rozpoczęcie synchronizacji. Właściwą komunikację zdalną realizuje jednak sama instancja PDS.

### 3.2 Niezależność od transportu

PDS-NP nie narzuca implementacji konkretnego transportu.

Domyślnym rozwiązaniem dla komunikacji peer-to-peer jest iroh. Inne transporty mogą zostać dodane, jeżeli spełniają wymagania bezpieczeństwa i dostarczania danych określone w tej specyfikacji.

Transport nie może omijać mechanizmów tożsamości, autoryzacji ani walidacji PDS.

## 4. Tożsamość instancji PDS

### 4.1 Tożsamość kryptograficzna

Każda instancja PDS posiada kryptograficzną tożsamość.

Tożsamość obejmuje:

- identyfikator instancji;
- klucz publiczny służący do weryfikacji podpisów;
- odpowiadający mu klucz prywatny służący do podpisywania.

Klucz prywatny pozostaje pod kontrolą właściciela instancji i nigdy nie jest przesyłany.

Identyfikator kryptograficzny instancji jest odciskiem klucza publicznego:

`identity_id = SHA256(public_key)`

Dzięki temu identyfikator jest deterministycznie powiązany z kluczem tożsamości.

### 4.2 Tożsamość transportowa

Tożsamość PDS jest niezależna od tożsamości transportowej.

W szczególności:

- identyfikator endpointu iroh nie zastępuje tożsamości PDS;
- adres IP nie jest tożsamością PDS;
- certyfikat TLS nie zastępuje klucza tożsamości PDS.

Adapter transportowy musi umożliwiać powiązanie połączenia z komunikującą się instancją PDS.

### 4.3 Zaufanie do tożsamości

Poprawny podpis nie oznacza automatycznie, że dana instancja jest zaufana.

Przed dopuszczeniem operacji wymagających zaufania odbiorca musi zweryfikować klucz peera względem lokalnego modelu zaufania.

Podstawą zaufania może być:

- wcześniejsze ręczne zatwierdzenie klucza;
- wcześniej zapisany odcisk tożsamości;
- jawnie zatwierdzona delegacja;
- inny mechanizm zaufania ustanowiony przez właściciela PDS.

Nieznana tożsamość nie może zostać automatycznie uznana za zaufaną wyłącznie na podstawie poprawnego podpisu.

### 4.4 Tożsamość urządzenia, konta i profilu

Tożsamość instancji PDS, urządzenia, konta i profilu stanowią odrębne pojęcia.

Ich wzajemne powiązania muszą być jawne i możliwe do zweryfikowania.

Podpis instancji potwierdza, która instancja wysłała wiadomość. Nie dowodzi sam w sobie, że instancja ma prawo działać w imieniu określonego konta lub profilu.

## 5. Bezpieczeństwo komunikacji

### 5.1 Wymagania obowiązkowe

Każda komunikacja PDS-NP musi spełniać następujące wymagania:

- poufność przesyłanych danych;
- integralność przesyłanych danych;
- uwierzytelnienie komunikujących się instancji;
- podpisywanie wiadomości;
- ochrona przed powtórzeniem wiadomości;
- powiązanie wiadomości z konkretną sesją;
- weryfikacja uprawnień do wykonywanych operacji.

Nie istnieje tryb komunikacji plaintext ani tryb anonimowy.

### 5.2 Szyfrowanie transportowe

Wszystkie wiadomości PDS-NP muszą być przesyłane przez uwierzytelniony i szyfrowany kanał.

Adapter transportowy odpowiada za poufność oraz integralność kanału.

Przykładowe rozwiązania:

| Transport | Mechanizm bezpieczeństwa |
|---|---|
| iroh | Szyfrowany transport QUIC |
| QUIC | TLS 1.3 |
| TCP | TLS 1.3 |

W przypadku TLS wymagane jest korzystanie z aktualnych, bezpiecznych parametrów TLS 1.3.

Nie wolno zaakceptować połączenia, którego zabezpieczenia są słabsze niż wymagane przez tę specyfikację.

Handshake PDS-NP jest wykonywany wewnątrz już ustanowionego szyfrowanego kanału transportowego.

### 5.3 Podpisy cyfrowe

Każda wiadomość protokołu musi posiadać podpis nadawcy.

Podpis potwierdza:

- pochodzenie wiadomości;
- integralność jej nagłówka;
- integralność jej payloadu;
- powiązanie wiadomości z nadawcą i odbiorcą.

Podpis nie gwarantuje prawdziwości treści ani uprawnienia nadawcy do wykonania operacji.

### 5.4 Algorytmy kryptograficzne

Profil bazowy PDS-NP definiuje:

| Zastosowanie | Algorytm |
|---|---|
| Podpisy tożsamości i wiadomości | Ed25519 |
| Funkcja skrótu | SHA-256 |
| Kodowanie danych | CBOR |
| Szyfrowanie komunikacji | Uwierzytelnione szyfrowanie zapewniane przez bezpieczny transport |

Implementacje nie mogą samodzielnie zastępować wymaganych algorytmów innymi algorytmami.

Przyszła zmiana algorytmów wymaga jawnej identyfikacji, negocjacji i zgodności z zasadami wersjonowania protokołu.

## 6. Format wiadomości

### 6.1 Kodowanie payloadu

Payload wiadomości PDS-NP jest kodowany za pomocą istniejącego Binary Codec PDS.

Aktualny format Binary Codec:

```text
PDS1
SHA-256(payload)
CBOR(payload)
```

Binary Codec odpowiada za:

- serializację i deserializację struktur;
- kontrolę integralności zakodowanego payloadu;
- wykrywanie uszkodzeń danych.

Suma SHA-256 w Binary Codec nie jest podpisem cyfrowym i nie zapewnia autentyczności nadawcy.

Autentyczność zapewnia podpis wiadomości PDS-NP, a poufność zapewnia szyfrowany transport.

### 6.2 Ramka PDS-NP

Wiadomość przesyłana przez transport składa się z ramki PDS-NP.

Logiczna struktura ramki:

| Pole | Znaczenie |
|---|---|
| Length | Długość całej ramki |
| Magic | Identyfikator protokołu `PDSN` |
| Protocol Version | Wersja protokołu |
| Message Type | Typ wiadomości |
| Session ID | Identyfikator sesji |
| Sequence | Numer sekwencyjny |
| Message ID | Unikalny identyfikator wiadomości |
| Sender ID | Identyfikator nadawcy |
| Recipient ID | Identyfikator odbiorcy |
| Payload Length | Długość payloadu |
| Payload | Dane zakodowane przez Binary Codec |
| Signature | Podpis Ed25519 nadawcy |

Wartości identyfikatorów i pól liczbowych muszą mieć jednoznaczną reprezentację binarną.

Długość ramki i payloadu musi być sprawdzana przed alokacją pamięci.

Szczegółowa reprezentacja binarna musi być jednoznaczna dla każdej wersji protokołu.

### 6.3 Podpis wiadomości

Podpis obejmuje:

- identyfikator protokołu;
- wersję protokołu;
- typ wiadomości;
- identyfikator sesji;
- numer sekwencyjny;
- identyfikator wiadomości;
- tożsamość nadawcy;
- tożsamość odbiorcy;
- skrót dokładnych bajtów payloadu.

Schemat logiczny:

```text
payload_hash = SHA256(serialized_payload)

signature = Ed25519.Sign(
    sender_private_key,
    domain_separator || canonical_header || payload_hash
)
```

`domain_separator` musi być stałą przypisaną do PDS-NP, tak aby podpisów z tego protokołu nie można było pomylić z podpisami używanymi w innym kontekście.

Podpis musi zostać zweryfikowany przed wykonaniem operacji opisanej przez wiadomość.

### 6.4 Wiadomości handshake

Wiadomości handshake również muszą być podpisane.

W czasie handshake identyfikator sesji może być wartością zerową. Po zakończeniu handshake każda wiadomość musi zawierać właściwy identyfikator ustanowionej sesji.

Wiadomości handshake nie mogą być traktowane jako wiadomości aplikacyjne.

### 6.5 Limity rozmiaru

Implementacja musi definiować i egzekwować limity:

- maksymalnego rozmiaru ramki;
- maksymalnego rozmiaru payloadu;
- maksymalnego rozmiaru fragmentu bloba;
- liczby jednoczesnych żądań;
- czasu oczekiwania na komunikat;
- liczby niepotwierdzonych wiadomości.

Limity muszą chronić przed niekontrolowanym zużyciem pamięci, procesora i zasobów sieciowych.

## 7. Handshake

### 7.1 Cel

Handshake ustanawia uwierzytelnioną sesję pomiędzy dwiema instancjami PDS.

W jego trakcie ustalane są:

- tożsamości obu instancji;
- poprawność kluczy i podpisów;
- zaufanie do obu tożsamości;
- wspólna wersja protokołu;
- obsługiwane możliwości;
- identyfikator sesji.

Handshake musi odbywać się po ustanowieniu szyfrowanego kanału transportowego.

### 7.2 Przebieg handshake

Handshake składa się z czterech komunikatów:

1. `ClientHello`
2. `ServerHello`
3. `ClientAuth`
4. `ServerFinished`

Nazwy Client i Server określają role stron w pojedynczym handshake. Nie oznaczają trwałej hierarchii instancji.

### 7.3 ClientHello

Inicjująca instancja wysyła komunikat `ClientHello`.

Zawiera on:

| Pole | Znaczenie |
|---|---|
| Identity ID | Identyfikator instancji PDS |
| Public Key | Klucz publiczny tożsamości |
| Nonce | Losowa wartość 32-bajtowa |
| Versions | Obsługiwane wersje protokołu |
| Capabilities | Obsługiwane możliwości |
| Signature | Podpis zawartości komunikatu |

Nonce musi być generowany za pomocą kryptograficznie bezpiecznego generatora losowego.

Nonce nie może być ponownie użyty w innym handshake.

Klient podpisuje komunikat własnym kluczem prywatnym.

### 7.4 ServerHello

Odpowiadająca instancja wysyła `ServerHello`.

Zawiera on:

| Pole | Znaczenie |
|---|---|
| Identity ID | Identyfikator instancji PDS |
| Public Key | Klucz publiczny tożsamości |
| Nonce | Losowa wartość 32-bajtowa |
| ClientHello Hash | Skrót otrzymanego ClientHello |
| Version | Wybrana wersja protokołu |
| Capabilities | Uzgodnione możliwości |
| Signature | Podpis odpowiedzi i dotychczasowego transcriptu |

Podpis musi obejmować ClientHello oraz dane ServerHello, w tym obie tożsamości, oba nonce, wybraną wersję i uzgodnione możliwości.

Inicjująca instancja weryfikuje:

- poprawność podpisu;
- zgodność identyfikatora z kluczem publicznym;
- zaufanie do tożsamości;
- zgodność wybranej wersji z oferowanymi wersjami;
- poprawność negocjacji możliwości.

### 7.5 ClientAuth

Inicjująca instancja odpowiada komunikatem `ClientAuth`.

Zawiera on:

| Pole | Znaczenie |
|---|---|
| Identity ID | Identyfikator instancji |
| Transcript Hash | Skrót dotychczasowego transcriptu |
| Signature | Podpis transcriptu |

Podpis musi obejmować wszystkie wcześniejsze komunikaty handshake.

Odpowiadająca instancja weryfikuje podpis oraz tożsamość inicjującej instancji.

### 7.6 ServerFinished

Odpowiadająca instancja wysyła `ServerFinished`.

Zawiera on:

| Pole | Znaczenie |
|---|---|
| Identity ID | Identyfikator instancji |
| Transcript Hash | Skrót kompletnego transcriptu handshake |
| Signature | Podpis potwierdzający zakończenie handshake |

Po poprawnej weryfikacji `ServerFinished` inicjująca instancja uznaje handshake za zakończony.

Odpowiadająca instancja uznaje handshake za zakończony po poprawnej weryfikacji `ClientAuth` i wysłaniu `ServerFinished`.

### 7.7 Transcript handshake

Transcript obejmuje wszystkie komunikaty handshake w kolejności ich przesłania.

Każda instancja musi obliczyć identyczny transcript.

Transcript musi zawierać:

- obie tożsamości;
- oba klucze publiczne;
- oba nonce;
- oferowane wersje;
- wybraną wersję;
- oferowane i uzgodnione możliwości;
- wszystkie podpisane komunikaty handshake.

Identyfikator sesji jest wyprowadzany z transcriptu:

```text
session_id = SHA256(
    "PDS-NP-SESSION" || transcript_hash
)
```

Identyfikator sesji musi być związany z oboma peerami oraz wynegocjowanymi parametrami.

### 7.8 Niepowodzenie handshake

Handshake musi zostać przerwany, jeżeli:

- podpis jest nieprawidłowy;
- tożsamość jest nieznana lub niezaufana;
- klucz publiczny nie odpowiada deklarowanej tożsamości;
- wybrana wersja nie jest obsługiwana lub nie została zaoferowana;
- brakuje wymaganej możliwości;
- nonce lub transcript jest nieprawidłowy;
- nie można potwierdzić powiązania tożsamości PDS z połączeniem transportowym;
- transport nie zapewnia wymaganego poziomu bezpieczeństwa.

Nieudany handshake nie może utworzyć uwierzytelnionej sesji.

## 8. Sesja

### 8.1 Identyfikator sesji

Każda wiadomość po handshake musi zawierać identyfikator sesji.

Wiadomość z innym identyfikatorem sesji musi zostać odrzucona.

Sesja jest powiązana z:

- tożsamością obu instancji;
- transcriptem handshake;
- wynegocjowaną wersją;
- uzgodnionymi możliwościami.

### 8.2 Numery sekwencyjne

Każda wiadomość aplikacyjna posiada numer sekwencyjny.

Numeracja jest niezależna w każdym kierunku komunikacji.

Numery muszą rosnąć monotonicznie w obrębie sesji.

Powtórzone numery sekwencyjne muszą być odrzucane.

Nowa sesja rozpoczyna nową przestrzeń numerów sekwencyjnych.

### 8.3 Identyfikator wiadomości

Każda wiadomość posiada unikalny identyfikator.

Identyfikator służy do:

- korelacji żądania z odpowiedzią;
- rozpoznawania powtórzeń;
- diagnostyki;
- obsługi ponowień w przypadku utraty połączenia.

Ponowienie operacji nie może powodować przypadkowego wielokrotnego wykonania nieidempotentnej operacji.

## 9. Typy wiadomości

### 9.1 Zarządzanie sesją

| Wiadomość | Znaczenie |
|---|---|
| `ClientHello` | Rozpoczęcie handshake |
| `ServerHello` | Odpowiedź na handshake |
| `ClientAuth` | Uwierzytelnienie inicjującej instancji |
| `ServerFinished` | Potwierdzenie zakończenia handshake |
| `Close` | Kontrolowane zamknięcie sesji |
| `ProtocolError` | Błąd protokołu |

### 9.2 Synchronizacja

| Wiadomość | Znaczenie |
|---|---|
| `SyncRequest` | Żądanie synchronizacji |
| `SyncResponse` | Informacje o synchronizacji |
| `SyncOffer` | Ogłoszenie dostępnych zmian |
| `SyncAccept` | Akceptacja zakresu synchronizacji |
| `SyncData` | Przekazanie zdarzeń lub zmian |
| `SyncAck` | Potwierdzenie odebranych danych |
| `SyncReject` | Odrzucenie synchronizacji |

Wiadomości synchronizacyjne muszą zachowywać oryginalną tożsamość, pochodzenie i powiązanie z historią przekazywanych danych.

Uwierzytelnienie nadawcy nie oznacza automatycznego zaufania do wszystkich otrzymanych danych.

Odbiorca musi zweryfikować dane zgodnie z modelem historii, integralności i zaufania PDS.

Rozwiązywanie konfliktów należy do modelu synchronizacji PDS, a nie do transportu.

W przypadku jednostek zmian w trybie `EncryptedPayload` lub `BlindChange`:
- Węzły sieciowe przesyłają zaszyfrowany ładunek bez jego modyfikacji.
- Węzły pośredniczące (relay) mogą uczestniczyć w replikacji i przechowywaniu zmian wyłącznie na podstawie jawnej struktury DAG bez znajomości kluczy deszyfrujących.

### 9.3 Transfer blobów

| Wiadomość | Znaczenie |
|---|---|
| `BlobRequest` | Żądanie bloba |
| `BlobOffer` | Ogłoszenie metadanych bloba |
| `BlobAccept` | Akceptacja transferu |
| `BlobChunk` | Fragment bloba |
| `BlobAck` | Potwierdzenie fragmentu |
| `BlobComplete` | Zakończenie transferu |
| `BlobReject` | Odrzucenie transferu |

Transfer musi obsługiwać:

- fragmentację;
- weryfikację fragmentów;
- potwierdzanie odbioru;
- wznowienie przerwanego transferu;
- weryfikację kompletności całego bloba.

Każdy komunikat `BlobChunk` musi być podpisany tak samo jak pozostałe wiadomości.

Manifest transferu powinien zawierać oczekiwany rozmiar i skrót zawartości.

Odbiorca nie może oznaczyć bloba jako kompletnego przed zweryfikowaniem całej zawartości względem oczekiwanego skrótu.

### 9.4 Pozostałe operacje

| Wiadomość | Znaczenie |
|---|---|
| `ObjectRequest` | Żądanie obiektu |
| `ObjectResponse` | Odpowiedź z obiektem |
| `QueryRequest` | Żądanie zapytania |
| `QueryResponse` | Wynik zapytania |
| `PermissionRequest` | Żądanie uprawnień |
| `PermissionResponse` | Decyzja dotycząca uprawnień |

Wszystkie operacje podlegają autoryzacji przez odbiorczą instancję PDS.

## 10. Przetwarzanie wiadomości

Odbiorca przetwarza wiadomości w następującej kolejności:

1. Odbiera kompletną ramkę z szyfrowanego transportu.
2. Weryfikuje framing, identyfikator protokołu i limity rozmiaru.
3. Weryfikuje wersję i strukturę ramki.
4. Weryfikuje identyfikator sesji.
5. Sprawdza numer sekwencyjny i ochronę przed powtórzeniem.
6. Weryfikuje tożsamość nadawcy i podpis.
7. Sprawdza uprawnienia do żądanej operacji.
8. Dekoduje i waliduje payload.
9. Wykonuje operację.
10. Zwraca podpisaną odpowiedź lub błąd.

Żadna operacja na danych nie może zostać wykonana przed zakończeniem wymaganych kontroli.

Błąd weryfikacji podpisu musi skutkować odrzuceniem wiadomości.

## 11. Wersjonowanie i negocjacja możliwości

PDS-NP jest wersjonowany niezależnie od Binary Codec i PDS-Client-API.

Wersja protokołu składa się z numeru głównego i pobocznego.

- Zmiana główna oznacza niekompatybilne zmiany protokołu.
- Zmiana poboczna oznacza rozszerzenia zachowujące zgodność.
- Poprawki niezmieniające zachowania protokołu nie wymagają zmiany formatu wiadomości.

Podczas handshake instancje negocjują najwyższą wspólną obsługiwaną wersję.

Wersja i możliwości muszą być uwzględnione w podpisanym transcriptcie handshake.

Nieznane możliwości opcjonalne mogą być ignorowane.

Nieznane możliwości wymagane muszą skutkować przerwaniem handshake albo odrzuceniem operacji wymagającej takiej możliwości.

Implementacja nie może automatycznie obniżać poziomu zabezpieczeń ani przechodzić na niezabezpieczoną wersję.

## 12. Autoryzacja

Uwierzytelnienie określa, z którą instancją PDS nawiązano połączenie.

Autoryzacja określa, jakie operacje dana instancja może wykonać.

Uprawnienia mogą rozróżniać co najmniej:

- odczyt obiektów;
- zapis obiektów;
- odczyt historii;
- dopisywanie zdarzeń do historii;
- odczyt blobów;
- zapis blobów;
- synchronizację wybranego zakresu danych;
- zarządzanie dostępem peerów.

Samo ustanowienie sesji nie przyznaje żadnych dodatkowych uprawnień.

Decyzje autoryzacyjne podejmuje instancja odbierająca żądanie.

Uprawnienia muszą być egzekwowane również dla danych otrzymywanych w ramach synchronizacji.

## 13. Zarządzanie kluczami

Implementacja powinna obsługiwać:

- rotację kluczy;
- unieważnianie tożsamości;
- usuwanie urządzeń;
- delegowanie zaufania;
- odzyskiwanie tożsamości.

Nowy klucz nie dziedziczy automatycznie zaufania do starego klucza.

Rotacja musi być uwierzytelniona przez dotychczasową tożsamość albo jawnie zatwierdzony mechanizm odzyskiwania.

Unieważniona tożsamość nie może ustanawiać nowych sesji.

Sesje należące do unieważnionej tożsamości powinny zostać zamknięte po otrzymaniu informacji o unieważnieniu.

## 14. Adaptery transportowe

Adapter transportowy musi zapewniać:

- dostarczanie danych lub jawne zgłoszenie błędu;
- poufność i integralność komunikacji;
- uwierzytelnienie transportowe, jeżeli oferuje własną tożsamość;
- powiązanie połączenia z tożsamością PDS;
- obsługę cyklu życia połączenia;
- limity zasobów;
- propagację błędów.

### 14.1 iroh

iroh jest preferowanym adapterem komunikacji P2P.

Odpowiada za:

- odnajdywanie peerów i ustanawianie połączeń;
- szyfrowaną komunikację QUIC;
- przechodzenie przez ograniczenia sieciowe;
- cykl życia połączenia;
- dostarczanie ramek PDS-NP.

Tożsamość endpointu iroh nie jest automatycznie tożsamością PDS.

Powiązanie tożsamości transportowej z tożsamością PDS musi zostać jawnie zweryfikowane.

Handshake PDS-NP pozostaje wymagany również wtedy, gdy iroh zapewnia szyfrowanie i uwierzytelnienie transportowe.

### 14.2 Inne transporty

Możliwe jest dodawanie innych transportów sieciowych, jeżeli spełniają wymagania bezpieczeństwa tej specyfikacji.

Przykłady:

- QUIC;
- TLS 1.3 nad TCP.

In-process i lokalne IPC nie są adapterami PDS-NP. Należą do PDS-Client-API.

## 15. Obsługa błędów

Protokół definiuje co najmniej następujące kategorie błędów:

| Kod | Znaczenie |
|---|---|
| `InvalidFrame` | Nieprawidłowa ramka |
| `UnsupportedVersion` | Nieobsługiwana wersja |
| `InvalidSignature` | Nieprawidłowy podpis |
| `UntrustedIdentity` | Niezaufana tożsamość |
| `AuthenticationFailed` | Nieudane uwierzytelnienie |
| `AuthorizationFailed` | Brak uprawnień |
| `InvalidSession` | Nieprawidłowa sesja |
| `ReplayDetected` | Wykryto powtórzoną wiadomość |
| `InvalidPayload` | Nieprawidłowy payload |
| `ResourceLimitExceeded` | Przekroczony limit zasobów |
| `Conflict` | Konflikt danych lub stanu |
| `OperationRejected` | Odrzucenie operacji |
| `InternalError` | Nieoczekiwany błąd |

Błędy protokołu przekazywane drugiej stronie muszą być podpisane.

Komunikaty błędów nie mogą ujawniać kluczy prywatnych, sekretów ani danych niedostępnych dla odbiorcy.

## 16. Bezpieczeństwo i zagrożenia

Implementacja musi uwzględniać co najmniej:

- ataki powtórzeniowe;
- ataki man-in-the-middle;
- ataki downgrade;
- nieautoryzowany dostęp peerów;
- modyfikację wiadomości;
- podszywanie się pod tożsamość;
- odmowę usługi;
- złośliwe payloady CBOR;
- nadmierne rozmiary blobów;
- wyczerpanie zasobów;
- przejęte lub złośliwe instancje PDS.

Wszystkie dane przychodzące są niezaufane, nawet jeśli pochodzą od uwierzytelnionego peera.

Podpis potwierdza pochodzenie wiadomości, ale nie gwarantuje bezpieczeństwa jej treści.

Dane muszą być walidowane zgodnie z modelem PDS.

Logi nie mogą ujawniać kluczy prywatnych, sekretów ani niezaszyfrowanych payloadów.

## 17. Wymagania zgodności

Implementacja zgodna z PDS-NP musi:

- realizować wymagany handshake;
- wzajemnie uwierzytelniać instancje PDS;
- weryfikować podpisy wiadomości;
- szyfrować całą komunikację;
- odrzucać wiadomości nieprawidłowe, powtórzone i nieautoryzowane;
- obsługiwać negocjację wersji i możliwości;
- stosować wymagany format ramek i kodowanie payloadu;
- zachowywać historię i pochodzenie danych synchronizacyjnych;
- weryfikować integralność blobów;
- nie udostępniać niezabezpieczonego trybu awaryjnego;
- nie traktować lokalnego klienta jako peera PDS-NP.

## 18. Granica odpowiedzialności

PDS-NP definiuje sposób komunikacji między niezależnymi instancjami PDS.

Nie definiuje:

- lokalnego dostępu aplikacji do PDS;
- interfejsu in-process;
- lokalnego IPC;
- wewnętrznego modelu przechowywania danych;
- szczegółów implementacji bazy danych;
- interfejsu użytkownika.

Transport odpowiada za bezpieczne dostarczanie danych.

PDS-NP definiuje znaczenie i reguły wiadomości.

Warstwa bezpieczeństwa weryfikuje tożsamość, podpisy i uprawnienia.

PDS Core wykonuje zaakceptowane operacje na danych.

Żaden adapter transportowy nie może omijać reguł tożsamości, bezpieczeństwa ani autoryzacji PDS.
