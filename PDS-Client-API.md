# PDS-Client-API — Interfejs dostępu do lokalnej instancji PDS

**Wersja:** 1.1.0  
**Status:** Projekt  
**Zakres:** Aplikacja kliencka ↔ lokalna instancja PDS

## 1. Cel i zakres

PDS-Client-API definiuje sposób, w jaki aplikacje i inne lokalne komponenty korzystają z instancji Personal Data Store (PDS).

Interfejs określa wspólny kontrakt dostępu do danych PDS, niezależnie od sposobu komunikacji pomiędzy klientem a lokalną instancją.

Zakres interfejsu obejmuje:

- dostęp do obiektów, atrybutów i relacji;
- dostęp do plików i blobów;
- odczyt historii oraz pochodzenia danych;
- wykonywanie zapytań i wyszukiwanie;
- korzystanie ze schematów i metadanych;
- import i eksport danych;
- obsługę zadań i operacji długotrwałych;
- obserwowanie zmian;
- zarządzanie synchronizacją z innymi instancjami PDS;
- uwierzytelnianie i autoryzację klientów;
- zarządzanie rejestracją klientów;
- odczyt stanu i możliwości lokalnej instancji.

PDS-Client-API dotyczy wyłącznie dostępu do lokalnej instancji PDS.

Komunikację pomiędzy niezależnymi instancjami PDS definiuje osobny dokument **PDS-NP**.

## 2. Zasady architektoniczne

1. PDS jest właścicielem i strażnikiem przechowywanych danych.
2. Klient korzysta z publicznego interfejsu, a nie z wewnętrznego mechanizmu przechowywania danych.
3. Klient nie może omijać walidacji, historii, uprawnień ani reguł integralności PDS.
4. In-process i Local IPC są sposobami dostępu do tej samej lokalnej instancji PDS.
5. Wszystkie sposoby dostępu muszą zachowywać tę samą semantykę operacji.
6. Uwierzytelnienie klienta i autoryzacja operacji są rozdzielnymi mechanizmami.
7. Lokalny dostęp nie oznacza automatycznego przyznania pełnych uprawnień.
8. Klient nie musi znać szczegółów synchronizacji ani komunikacji sieciowej PDS.
9. Interfejs nie narzuca konkretnej technologii implementacyjnej ani sposobu przechowywania danych.
10. Tożsamość klienta jest niezależna od tożsamości instancji PDS, urządzenia oraz konta.
11. Każdy klient podlegający kontroli dostępu musi posiadać jawnie zarejestrowaną tożsamość.
12. Dostęp klienta może zostać unieważniony bez usuwania danych należących do PDS.

## 3. Architektura

PDS-Client-API stanowi granicę pomiędzy aplikacją a lokalną instancją PDS.

```text
+----------------------------------+
|           Applications           |
|                                  |
|  UI / Application Logic          |
+----------------+-----------------+
                 |
                 | PDS-Client-API
                 |
          +------+------+
          |             |
     In-process    Local IPC
          |             |
          +------+------+
                 |
+----------------v-----------------+
|       Local PDS Instance         |
|                                  |
|  Client Registration             |
|  Client Authentication           |
|  Authorization                   |
|  Session Management              |
|  Validation                      |
|                                  |
|  PDS Core                        |
|  Data Model                      |
|  History & Provenance            |
|  Storage                         |
|  Synchronization Engine           |
+----------------+-----------------+
                 |
                 | PDS-NP
                 |
         Other PDS Instances
```

Lokalna instancja PDS odpowiada za wszystkie operacje na danych.

Jeżeli klient zleca synchronizację z inną instancją, lokalny PDS realizuje ją we własnym zakresie za pomocą PDS-NP.

Klient nie uczestniczy bezpośrednio w komunikacji PDS-NP.

## 4. Model dostępu

PDS-Client-API definiuje jeden logiczny kontrakt, który może być udostępniany za pomocą różnych mechanizmów komunikacji.

### 4.1 In-process

In-process umożliwia aplikacji korzystanie z PDS w obrębie tego samego procesu.

Charakterystyka:

- bezpośrednia komunikacja z lokalną instancją;
- brak konieczności przesyłania danych przez IPC;
- brak wymogu serializacji komunikatów transportowych;
- wspólny model operacji, wyników i błędów;
- zachowanie mechanizmów autoryzacji i historii operacji.

In-process nie jest osobnym protokołem komunikacyjnym ani mechanizmem synchronizacji.

#### 4.1.1 Granica bezpieczeństwa

Kod działający w tym samym procesie co PDS należy do tego samego podstawowego obszaru zaufania.

PDS może przypisywać operacje do określonych kontekstów klientów, jednak takie przypisanie nie zapewnia izolacji przed złośliwym kodem działającym w tym samym procesie.

W szczególności:

- kod działający w tym samym procesie może mieć dostęp do pamięci procesu;
- nie można traktować identyfikatora modułu jako mechanizmu izolacji bezpieczeństwa;
- klucz prywatny klienta nie może być uznany za chroniony przed innym kodem z tego samego obszaru zaufania;
- mechanizmy uprawnień In-process służą przede wszystkim kontroli dostępu i rozliczalności operacji.

Jeżeli aplikacja korzysta z niezaufanych pluginów lub modułów, powinny one działać w osobnych procesach i komunikować się z PDS przez Local IPC.

### 4.2 Local IPC

Local IPC umożliwia komunikację pomiędzy aplikacją a lokalną instancją PDS działającą w osobnym procesie.

Charakterystyka:

- komunikacja ograniczona do lokalnego środowiska;
- możliwość współdzielenia jednej instancji PDS przez wiele aplikacji;
- rozdzielenie cyklu życia aplikacji i PDS;
- komunikacja za pomocą jawnego kontraktu żądań i odpowiedzi;
- obsługa rozłączeń i ponownego nawiązywania połączenia;
- możliwość niezależnego uwierzytelniania i autoryzacji klientów.

Mechanizm IPC musi umożliwiać ograniczenie dostępu do uprawnionych klientów.

Implementacja może korzystać z mechanizmów kontroli dostępu zapewnianych przez system operacyjny.

Przykładowe mechanizmy:

- Unix Domain Socket wraz z kontrolą uprawnień i identyfikacji procesu;
- Named Pipes wraz z ACL w systemie Windows;
- inne lokalne mechanizmy IPC oferujące porównywalne możliwości.

### 4.3 Niezależność od sposobu dostępu

In-process i Local IPC muszą udostępniać te same operacje.

Różnice implementacyjne nie mogą wpływać na:

- semantykę operacji;
- model danych;
- walidację;
- uprawnienia;
- historię;
- obsługę błędów;
- zasady spójności.

Klient powinien móc korzystać z tych samych abstrakcji niezależnie od wybranego sposobu dostępu.

## 5. Rejestracja klientów

### 5.1 Tożsamość klienta

Każda aplikacja korzystająca z PDS i podlegająca niezależnej kontroli dostępu posiada własną tożsamość klienta.

Tożsamość klienta identyfikuje konkretną rejestrację aplikacji w lokalnej instancji PDS.

Nie jest ona tożsamością:

- instancji PDS;
- urządzenia uczestniczącego w synchronizacji;
- użytkownika systemowego;
- konta lub profilu przechowywanego w PDS.

Jedna aplikacja może posiadać różne tożsamości klientów w różnych instancjach PDS.

### 5.2 Klucze klienta

Klient posiada parę kluczy kryptograficznych:

- klucz publiczny, rejestrowany w PDS;
- klucz prywatny, przechowywany przez aplikację.

Profil bazowy wykorzystuje Ed25519.

Klucz prywatny:

- nie może być przekazywany do PDS;
- nie może być zapisywany w logach;
- powinien być przechowywany w bezpiecznym magazynie kluczy dostępnym dla aplikacji;
- nie powinien być współdzielony pomiędzy niezależnymi instalacjami aplikacji.

Klucz publiczny służy do weryfikacji podpisów i identyfikacji klienta.

### 5.3 Identyfikator klienta

PDS nadaje każdej rejestracji unikalny identyfikator klienta.

Identyfikator nie jest bezpośrednim zamiennikiem klucza publicznego i nie służy samodzielnie do uwierzytelnienia.

Przykładowy rekord:

```text
Client:
    client_id: <ULID>
    name: ShopLedger
    public_key: <Ed25519 public key>
    status: Pending
    permissions: []
    created_at: <timestamp>
```

### 5.4 Proces rejestracji

Rejestracja nowego klienta przebiega następująco:

1. Aplikacja generuje lokalną parę kluczy Ed25519.
2. Aplikacja nawiązuje połączenie z lokalną instancją PDS.
3. Aplikacja wysyła żądanie rejestracji zawierające nazwę klienta i klucz publiczny.
4. PDS weryfikuje poprawność żądania.
5. PDS tworzy rekord klienta o stanie `Pending`.
6. Właściciel PDS zatwierdza lub odrzuca rejestrację.
7. Po zatwierdzeniu klient otrzymuje stan `Active` oraz przyznane uprawnienia.

Samo przesłanie klucza publicznego nie oznacza przyznania dostępu do danych.

### 5.5 Zatwierdzanie rejestracji

Nowy klient nie może samodzielnie przyznać sobie uprawnień.

Zatwierdzenie musi pochodzić z zaufanego kontekstu administracyjnego PDS.

Dopuszczalne mechanizmy zatwierdzania:

- zatwierdzenie przez interfejs administracyjny lokalnego PDS;
- ręczna akceptacja odcisku klucza;
- jednorazowy kod rejestracyjny wygenerowany przez właściciela;
- inny jawnie zatwierdzony mechanizm parowania.

Jeżeli używany jest kod rejestracyjny, musi on być:

- wygenerowany kryptograficznie;
- ograniczony czasowo;
- jednorazowy;
- związany z konkretną instancją PDS;
- unieważniany po wykorzystaniu lub wygaśnięciu.

Kod nie może być traktowany jako stały sekret klienta.

### 5.6 Stan klienta

Klient może znajdować się w jednym z następujących stanów:

| Stan | Znaczenie |
|---|---|
| `Pending` | Oczekuje na zatwierdzenie |
| `Active` | Może się uwierzytelniać i wykonywać dozwolone operacje |
| `Suspended` | Tymczasowo zablokowany |
| `Revoked` | Dostęp został trwale unieważniony |

Klient w stanie innym niż `Active` nie może uzyskać nowej aktywnej sesji.

### 5.7 Zmiana klucza

Zmiana klucza publicznego wymaga jawnej operacji rotacji lub ponownej rejestracji.

Nowy klucz nie dziedziczy automatycznie zaufania do poprzedniego klucza.

Rotacja powinna być potwierdzona przez:

- dotychczasowy klucz prywatny;
- albo zaufany mechanizm administracyjny PDS.

W przypadku utraty klucza prywatnego właściciel PDS musi mieć możliwość unieważnienia starej rejestracji i zatwierdzenia nowej.

## 6. Uwierzytelnianie klientów

### 6.1 Rozdzielenie uwierzytelniania i autoryzacji

Uwierzytelnianie ustala tożsamość klienta.

Autoryzacja określa, jakie operacje uwierzytelniony klient może wykonywać.

Poprawne uwierzytelnienie nie oznacza automatycznego przyznania dostępu do wszystkich danych.

### 6.2 Uwierzytelnianie Local IPC

Przy każdym nowym połączeniu IPC klient musi udowodnić, że posiada klucz prywatny przypisany do aktywnej rejestracji.

Uwierzytelnianie wykorzystuje mechanizm challenge-response.

### 6.3 Przebieg uwierzytelniania

Wymiana składa się z następujących komunikatów:

1. `ClientHello`
2. `AuthChallenge`
3. `ClientAuth`
4. `AuthSuccess` lub `AuthFailure`

#### 6.3.1 ClientHello

Klient przesyła:

| Pole | Znaczenie |
|---|---|
| `client_id` | Identyfikator rejestracji klienta |
| `api_version` | Obsługiwana wersja PDS-Client-API |
| `client_nonce` | Losowa wartość 32-bajtowa |
| `capabilities` | Obsługiwane możliwości klienta |

Klient nie jest jeszcze uwierzytelniony.

#### 6.3.2 AuthChallenge

PDS weryfikuje, czy klient istnieje i czy jego status umożliwia rozpoczęcie uwierzytelniania.

Następnie generuje kryptograficznie losową wartość i wysyła:

| Pole | Znaczenie |
|---|---|
| `server_instance_id` | Identyfikator lokalnej instancji PDS |
| `client_id` | Identyfikator klienta |
| `client_nonce` | Nonce otrzymany od klienta |
| `server_nonce` | Nowy losowy nonce PDS |
| `api_version` | Uzgodniona wersja API |
| `challenge_expires_at` | Termin ważności wyzwania |

Nonce serwera musi mieć co najmniej 32 bajty.

Wyzwanie jest jednorazowe i ograniczone czasowo.

#### 6.3.3 ClientAuth

Klient podpisuje dane wyzwania swoim kluczem prywatnym.

Podpis obejmuje:

- stały identyfikator kontekstu uwierzytelniania;
- identyfikator instancji PDS;
- identyfikator klienta;
- nonce klienta;
- nonce serwera;
- wersję API.

Schemat logiczny:

```text
signature = Ed25519.Sign(
    client_private_key,
    "PDS-CLIENT-AUTH-v1" ||
    server_instance_id ||
    client_id ||
    client_nonce ||
    server_nonce ||
    api_version
)
```

Klient przesyła:

| Pole | Znaczenie |
|---|---|
| `client_id` | Identyfikator klienta |
| `signature` | Podpis danych wyzwania |

#### 6.3.4 Weryfikacja przez PDS

PDS:

1. Pobiera zarejestrowany klucz publiczny klienta.
2. Sprawdza, czy klient nadal ma stan `Active`.
3. Sprawdza ważność i jednorazowość wyzwania.
4. Sprawdza powiązanie wyzwania z aktualnym połączeniem.
5. Weryfikuje podpis Ed25519.
6. Odczytuje uprawnienia klienta.
7. Tworzy sesję klienta albo odrzuca uwierzytelnienie.

Weryfikacja podpisu musi odbywać się przed przyznaniem sesji.

#### 6.3.5 AuthSuccess

Po pomyślnym uwierzytelnieniu PDS zwraca:

| Pole | Znaczenie |
|---|---|
| `session_id` | Identyfikator sesji |
| `client_id` | Identyfikator klienta |
| `permissions` | Przyznane uprawnienia |
| `expires_at` | Termin wygaśnięcia sesji |
| `api_version` | Wersja API używana w sesji |

W przypadku błędu PDS zwraca `AuthFailure` z odpowiednim kodem błędu.

### 6.4 Ochrona przed powtórzeniem

Każde wyzwanie musi być:

- kryptograficznie losowe;
- jednorazowe;
- związane z konkretnym klientem;
- związane z konkretną instancją PDS;
- ograniczone czasowo;
- związane z konkretnym połączeniem.

Po użyciu wyzwanie musi zostać unieważnione.

Wyzwanie nie może być ponownie wykorzystane nawet wtedy, gdy uwierzytelnianie zakończyło się błędem.

### 6.5 Weryfikacja procesu przez system operacyjny

W przypadku Local IPC PDS powinien dodatkowo sprawdzić tożsamość procesu łączącego się z lokalną instancją.

Przykładowe mechanizmy:

- `SO_PEERCRED` w systemach Linux;
- `getpeereid` w systemach obsługujących tę funkcję;
- ACL i informacje o kliencie potoku nazwanego w systemie Windows.

Wynik kontroli systemowej powinien zostać powiązany z kontekstem uwierzytelniania klienta.

Kontrola systemowa i podpis Ed25519 pełnią różne funkcje:

- kontrola systemowa potwierdza, jaki lokalny użytkownik lub proces otworzył połączenie;
- podpis Ed25519 potwierdza posiadanie klucza prywatnego przypisanego do rejestracji klienta.

Oba mechanizmy powinny być stosowane razem, gdy system operacyjny udostępnia odpowiednie możliwości.

### 6.6 Uwierzytelnianie In-process

W przypadku In-process klient korzysta z kontekstu dostępu przekazanego podczas inicjalizacji interfejsu.

Kontekst może określać:

- identyfikator klienta;
- przypisane uprawnienia;
- identyfikator komponentu;
- sposób wykonywania operacji.

Uwierzytelnianie In-process nie stanowi izolacji bezpieczeństwa pomiędzy modułami tego samego procesu.

PDS może wykorzystywać kontekst klienta do autoryzacji i rejestrowania operacji, ale nie może zakładać, że inny kod w tym samym procesie nie jest w stanie go podrobić.

Niezaufany kod wymagający rzeczywistej granicy bezpieczeństwa musi korzystać z osobnego procesu i Local IPC.

## 7. Sesje klientów

### 7.1 Utworzenie sesji

Po poprawnym uwierzytelnieniu PDS tworzy sesję klienta.

Sesja jest przypisana do:

- konkretnej rejestracji klienta;
- konkretnego uwierzytelnienia;
- konkretnego połączenia IPC;
- konkretnego zestawu uprawnień;
- lokalnej instancji PDS.

Przykładowy model:

```text
ClientSession:
    session_id: <random identifier>
    client_id: <ULID>
    permissions: [...]
    created_at: <timestamp>
    expires_at: <timestamp>
    status: Active
```

Identyfikator sesji powinien być generowany kryptograficznie i mieć co najmniej 256 bitów entropii.

### 7.2 Ważność sesji

Sesja posiada ograniczony czas ważności.

PDS powinien umożliwiać:

- ustawienie maksymalnego czasu życia sesji;
- automatyczne wygaszenie sesji;
- unieważnienie sesji;
- zakończenie sesji po rozłączeniu IPC;
- unieważnienie wszystkich sesji określonego klienta.

Rozłączenie połączenia IPC powinno zakończyć przypisaną do niego sesję.

Ponowne połączenie wymaga ponownego uwierzytelnienia.

### 7.3 Wykonywanie operacji

Przy każdej operacji PDS:

1. Rozpoznaje kontekst sesji.
2. Sprawdza, czy sesja jest aktywna.
3. Sprawdza, czy rejestracja klienta nadal jest aktywna.
4. Sprawdza wymagane uprawnienie.
5. Waliduje parametry operacji.
6. Wykonuje operację albo zwraca błąd.

Klient nie może samodzielnie rozszerzać uprawnień przypisanych do sesji.

### 7.4 Zmiany uprawnień

Zmiana uprawnień klienta powinna wpływać na jego aktywne sesje.

PDS powinien unieważnić sesje klienta lub odświeżyć ich zakres uprawnień.

Klient nie może zachować wcześniej przyznanych uprawnień po ich odebraniu przez właściciela PDS.

## 8. Zarządzanie klientami

Lokalna instancja PDS powinna udostępniać właścicielowi operacje zarządzania rejestracjami klientów.

### 8.1 Dostępne operacje

- wyświetlanie zarejestrowanych klientów;
- wyświetlanie oczekujących rejestracji;
- zatwierdzanie nowych klientów;
- odrzucanie rejestracji;
- zmiana nazwy lub metadanych klienta;
- zmiana uprawnień;
- tymczasowe blokowanie klienta;
- odblokowywanie klienta;
- rotacja klucza;
- unieważnianie klucza;
- usuwanie rejestracji;
- wyświetlanie aktywnych sesji;
- unieważnianie sesji;
- przeglądanie historii operacji wykonywanych przez klientów.

Operacje administracyjne wymagają osobnych uprawnień.

### 8.2 Unieważnienie klienta

Unieważnienie klienta musi:

- zmienić jego status na `Revoked`;
- uniemożliwić tworzenie nowych sesji;
- unieważnić wszystkie aktywne sesje;
- odrzucać kolejne operacje tego klienta;
- zachować historyczne informacje o jego wcześniejszej aktywności.

Unieważnienie klienta nie może automatycznie usuwać danych utworzonych przez tego klienta.

### 8.3 Zawieszenie klienta

Zawieszenie klienta jest odwracalne.

Klient w stanie `Suspended` nie może uzyskiwać nowych sesji ani wykonywać kolejnych operacji.

Po przywróceniu statusu `Active` klient może ponownie się uwierzytelniać, jeżeli jego klucz nadal jest ważny.

## 9. Autoryzacja

Każda operacja musi zostać sprawdzona względem uprawnień klienta oraz polityki lokalnej instancji PDS.

Autoryzacja jest niezależna od sposobu komunikacji.

### 9.1 Zakres uprawnień

Przykładowe uprawnienia:

| Uprawnienie | Znaczenie |
|---|---|
| `object.read` | Odczyt obiektów |
| `object.create` | Tworzenie obiektów |
| `object.update` | Aktualizacja obiektów |
| `object.delete` | Usuwanie lub oznaczanie obiektów jako usunięte |
| `attribute.read` | Odczyt atrybutów |
| `attribute.write` | Zmiana atrybutów |
| `relation.read` | Odczyt relacji |
| `relation.write` | Tworzenie i zmiana relacji |
| `history.read` | Odczyt historii |
| `history.append` | Dopisywanie zdarzeń |
| `blob.read` | Odczyt blobów |
| `blob.write` | Dodawanie blobów |
| `query.execute` | Wykonywanie zapytań |
| `sync.read` | Odczyt stanu synchronizacji |
| `sync.manage` | Zarządzanie synchronizacją |
| `schema.read` | Odczyt schematów |
| `schema.manage` | Zarządzanie schematami |
| `client.read` | Odczyt rejestracji klientów |
| `client.manage` | Zarządzanie rejestracją klientów |
| `crypto.read` | Odczyt kluczy szyfrujących E2EE |
| `crypto.manage` | Zarządzanie kluczami i rotacja E2EE |
| `admin` | Operacje administracyjne |

Lista ma charakter logiczny i może być rozszerzana.

Uprawnienia mogą być ograniczane do określonych zakresów danych, kolekcji, obiektów lub operacji.

### 9.2 Odmowa dostępu

Brak uprawnienia musi skutkować jawnym błędem autoryzacji.

Odmowa dostępu nie może być przedstawiana jako pusty wynik zapytania, jeżeli powodowałoby to niejednoznaczność.

### 9.3 Zasada minimalnych uprawnień

Nowy klient powinien otrzymywać minimalny zestaw uprawnień potrzebny do działania.

Klient nie może otrzymywać uprawnień administracyjnych domyślnie.

Przyznane uprawnienia powinny być możliwe do indywidualnej zmiany i odebrania.

## 10. Model danych API

PDS-Client-API udostępnia pojęcia domenowe PDS, a nie wewnętrzne struktury konkretnej bazy danych.

### 10.1 Obiekt

Obiekt jest podstawową jednostką danych PDS.

API umożliwia identyfikowanie, odczytywanie i modyfikowanie obiektów zgodnie z modelem danych PDS.

### 10.2 Atrybut

Atrybut opisuje właściwość obiektu.

API nie narzuca konkretnego zestawu atrybutów domenowych.

### 10.3 Relacja

Relacja reprezentuje jawne powiązanie pomiędzy obiektami.

Relacja może posiadać własne atrybuty, pochodzenie i historię.

### 10.4 Blob

Blob reprezentuje zawartość binarną przechowywaną lub udostępnianą przez PDS.

API umożliwia dostęp do zawartości niezależnie od sposobu jej fizycznego przechowywania.

### 10.5 Rewizja i zdarzenie

Rewizja reprezentuje określony stan danych.

Zdarzenie reprezentuje zmianę lub operację zarejestrowaną w historii PDS.

### 10.6 Pochodzenie

Pochodzenie opisuje źródło danych i sposób ich powstania.

API umożliwia odczyt informacji o pochodzeniu zgodnie z uprawnieniami klienta.

## 11. Operacje na obiektach

API powinno udostępniać następujące operacje:

- utworzenie obiektu;
- pobranie obiektu;
- pobranie wielu obiektów;
- aktualizacja obiektu;
- usunięcie lub oznaczenie obiektu jako usuniętego;
- pobranie określonej rewizji;
- sprawdzenie istnienia obiektu.

Zmiana danych musi być zgodna z modelem historii PDS.

Jeżeli PDS stosuje historię append-only, aktualizacja jest nowym zdarzeniem lub rewizją, a nie nieodwracalnym nadpisaniem poprzedniego stanu.

Usunięcie obiektu musi zachowywać zasady historii, integralności i powiązań określone przez PDS.

### 11.1 Idempotencja

Operacje, które mogą być ponawiane po utracie połączenia, powinny umożliwiać przekazanie identyfikatora operacji lub klucza idempotencji.

PDS musi umożliwiać rozpoznanie ponowionego żądania, aby uniknąć przypadkowego wielokrotnego wykonania tej samej operacji.

## 12. Atrybuty i relacje

### 12.1 Atrybuty

API umożliwia:

- odczyt atrybutów obiektu;
- dodawanie atrybutów;
- zmianę wartości atrybutów;
- usuwanie atrybutów;
- walidację względem schematu.

Walidacja może być wykonywana przez PDS.

Błąd walidacji powinien wskazywać atrybut lub regułę, której dane nie spełniają.

API nie powinno wymuszać na kliencie znajomości wewnętrznej reprezentacji atrybutów.

### 12.2 Relacje

API umożliwia:

- tworzenie relacji;
- pobieranie relacji;
- wyszukiwanie relacji według obiektu;
- filtrowanie po typie relacji;
- filtrowanie po atrybutach relacji;
- zmianę dozwolonych właściwości relacji;
- zakończenie lub usunięcie relacji zgodnie z modelem historii PDS.

Relacja może posiadać własną historię i pochodzenie.

## 13. Pliki i bloby

API zapewnia dostęp do zawartości binarnej zarządzanej przez PDS.

Powinno udostępniać następujące operacje:

- rozpoczęcie dodawania bloba;
- przekazywanie zawartości;
- wznowienie przerwanego transferu;
- zakończenie dodawania;
- pobranie bloba;
- pobranie fragmentu bloba;
- pobranie metadanych;
- pobranie skrótu zawartości;
- sprawdzenie dostępności bloba.

### 13.1 Transfer

Duże pliki powinny być przesyłane strumieniowo lub w porcjach.

API nie powinno wymagać wczytania całego pliku do pamięci klienta.

PDS odpowiada za:

- kontrolę integralności;
- weryfikację kompletności;
- rejestrację zawartości;
- zarządzanie metadanymi;
- zapewnienie zgodności z modelem przechowywania.

Klient nie może uznać operacji za zakończoną przed otrzymaniem potwierdzenia PDS.

### 13.2 Uprawnienia

Odczyt i zapis blobów podlegają osobnym uprawnieniom.

Dostęp do obiektu nie oznacza automatycznie dostępu do wszystkich powiązanych blobów.

## 14. Historia i pochodzenie

API powinno udostępniać:

- historię obiektu;
- historię relacji;
- informacje o podmiocie wykonującym operację;
- czas wykonania operacji;
- źródło danych;
- powiązanie z rewizją lub zdarzeniem;
- informacje o pochodzeniu danych pochodnych.

Operacje wykonywane przez klienta muszą być przypisywane do uwierzytelnionego kontekstu klienta.

Klient może odczytywać historię zgodnie z uprawnieniami.

Nie może dowolnie modyfikować ani usuwać wcześniejszych wpisów historii.

Operacje administracyjne dotyczące historii, jej retencji lub odzyskiwania podlegają odrębnym uprawnieniom i regułom PDS.

## 15. Zapytania i wyszukiwanie

API udostępnia zapytania niezależne od konkretnego silnika przechowywania.

Obsługiwane operacje mogą obejmować:

- wyszukiwanie według identyfikatora;
- filtrowanie według atrybutów;
- filtrowanie według relacji;
- zapytania zakresowe;
- sortowanie;
- stronicowanie;
- wyszukiwanie tekstowe;
- odczyt historii;
- wyszukiwanie według pochodzenia.

Interfejs nie powinien wymagać od klienta znajomości SQL ani wewnętrznej struktury bazy danych.

PDS odpowiada za:

- walidację zapytań;
- egzekwowanie uprawnień;
- ograniczanie kosztu zapytań;
- kontrolę dostępu do zwracanych danych.

Wyniki powinny zawierać informacje o stronicowaniu i, jeżeli jest to możliwe, rewizję lub punkt historii, względem którego wykonano odczyt.

## 16. Zdarzenia i obserwowanie zmian

API może udostępniać mechanizm obserwowania zmian w lokalnej instancji PDS.

Zdarzenia mogą informować o:

- utworzeniu obiektu;
- zmianie obiektu;
- zmianie relacji;
- dodaniu bloba;
- zmianie stanu zadania;
- zmianie stanu synchronizacji;
- zmianie uprawnień;
- zmianie konfiguracji.

### 16.1 Trwałość zdarzeń

API musi rozróżniać:

- trwałe zdarzenia zapisane w historii PDS;
- tymczasowe powiadomienia informujące o zmianie stanu.

Powiadomienie nie jest samo w sobie źródłem prawdy.

Klient musi mieć możliwość ponownego odczytania aktualnego stanu po utracie połączenia lub pominięciu powiadomień.

### 16.2 Wznawianie obserwacji

Mechanizm obserwowania zmian powinien umożliwiać wznowienie od określonego kursora lub punktu historii, jeżeli PDS gwarantuje dostępność odpowiednich zdarzeń.

Jeżeli wznowienie nie jest możliwe, PDS musi poinformować klienta o konieczności ponownego pobrania stanu.

## 17. Zadania i operacje długotrwałe

Operacje, których wykonanie może trwać dłużej, powinny być reprezentowane jako zadania.

Przykładowe operacje:

- import większego zbioru danych;
- eksport danych;
- przetwarzanie plików;
- operacje porządkowania;
- synchronizacja z inną instancją PDS;
- operacje utrzymaniowe.

API powinno umożliwiać:

- utworzenie zadania;
- pobranie jego stanu;
- obserwowanie postępu;
- anulowanie, jeżeli jest to możliwe;
- pobranie wyniku;
- pobranie informacji o błędzie.

Rozłączenie klienta nie oznacza automatycznego anulowania zadania.

Politykę kontynuowania zadań określa lokalna instancja PDS.

## 18. Zarządzanie kluczami E2EE

PDS-Client-API udostępnia operacje zarządzania kluczami szyfrującymi dla aplikacji korzystających z szyfrowania end-to-end:

- `crypto.key.get` — pobranie klucza publicznego lub owiniętego klucza symetrycznego dla danego `KeyId`;
- `crypto.key.register` — zarejestrowanie nowego klucza domeny lub przestrzeni nazw;
- `crypto.key.wrap` — dodanie wersji klucza owiniętej kluczem publicznym innego zaufanego urządzenia;
- `crypto.key.rotate` — rotacja klucza symetrycznego dla nowych zmian.

Aplikacja kliencka posiadająca uprawnienie `crypto.manage` może konfigurować politykę szyfrowania dla własnej przestrzeni nazw (np. wymuszenie trybu `EncryptedPayload` lub `BlindChange`).


## 19. Synchronizacja z innymi instancjami

PDS-Client-API może udostępniać operacje zarządzania synchronizacją.

Nie udostępnia jednak samego protokołu PDS-NP.

Przykładowe operacje:

- wyświetlenie znanych peerów;
- dodanie lub zatwierdzenie peera;
- usunięcie zaufania do peera;
- pobranie stanu synchronizacji;
- rozpoczęcie synchronizacji;
- wstrzymanie synchronizacji;
- wznowienie synchronizacji;
- pobranie informacji o konfliktach;
- pobranie błędów synchronizacji.

Właściwa komunikacja z peerem jest realizowana przez lokalną instancję PDS za pomocą PDS-NP.

Klient nie może bezpośrednio modyfikować stanu sesji sieciowej ani omijać mechanizmów bezpieczeństwa PDS-NP.

## 19. Żądania, odpowiedzi i błędy

PDS-Client-API definiuje wspólny model logiczny żądań i odpowiedzi.

Konkretny format reprezentacji zależy od sposobu dostępu:

- In-process może korzystać z bezpośrednich wywołań API.
- Local IPC korzysta z komunikatów przesyłanych przez lokalny mechanizm komunikacji.

Różnice w reprezentacji nie mogą wpływać na semantykę operacji.

### 19.1 Żądanie

Żądanie powinno zawierać:

- identyfikator operacji;
- parametry wejściowe;
- kontekst sesji klienta;
- opcjonalny identyfikator korelacyjny;
- opcjonalny warunek oczekiwanej rewizji;
- opcjonalny klucz idempotencji.

### 19.2 Odpowiedź

Odpowiedź zawiera:

- wynik operacji albo informację o błędzie;
- identyfikator korelacyjny, jeżeli został przekazany;
- opcjonalne metadane;
- opcjonalny identyfikator utworzonej rewizji lub zdarzenia.

### 19.3 Kategorie błędów

| Kod | Znaczenie |
|---|---|
| `Unauthenticated` | Klient nie został uwierzytelniony |
| `Unauthorized` | Brak uprawnienia do operacji |
| `NotFound` | Nie znaleziono zasobu |
| `AlreadyExists` | Zasób już istnieje |
| `ValidationFailed` | Dane nie spełniają reguł walidacji |
| `Conflict` | Konflikt rewizji lub stanu |
| `InvalidArgument` | Nieprawidłowe parametry |
| `ResourceLimit` | Przekroczono limit zasobów |
| `Unavailable` | Operacja lub zasób chwilowo niedostępny |
| `Cancelled` | Operacja została anulowana |
| `Internal` | Nieoczekiwany błąd PDS |

Kategorie błędów uwierzytelniania mogą dodatkowo obejmować:

| Kod | Znaczenie |
|---|---|
| `UnknownClient` | Nieznany identyfikator klienta |
| `ClientPending` | Klient oczekuje na zatwierdzenie |
| `ClientSuspended` | Klient jest zawieszony |
| `ClientRevoked` | Klient został unieważniony |
| `InvalidSignature` | Podpis klienta jest nieprawidłowy |
| `ChallengeExpired` | Wyzwanie wygasło |
| `ChallengeReplayed` | Wykryto ponowne wykorzystanie wyzwania |
| `SessionExpired` | Sesja wygasła |
| `SessionRevoked` | Sesja została unieważniona |

Błąd powinien zawierać stabilny kod, zwięzły opis i opcjonalne szczegóły przydatne dla aplikacji.

Komunikaty błędów nie mogą ujawniać sekretów ani danych, do których klient nie ma dostępu.

## 20. Spójność i współbieżność

API musi określać semantykę odczytu i zapisu.

### 20.1 Kontrola rewizji

Operacje zapisu powinny umożliwiać przekazanie oczekiwanej rewizji obiektu lub równoważnego warunku spójności.

Jeżeli stan obiektu zmienił się od czasu odczytu, PDS powinien zwrócić błąd konfliktu zamiast po cichu nadpisywać nowsze dane.

### 20.2 Obsługa konfliktów

Po otrzymaniu konfliktu klient powinien:

1. Pobrać aktualny stan obiektu.
2. Rozpoznać różnice.
3. Przygotować nową operację.
4. Ponowić zapis z uwzględnieniem aktualnej rewizji.

PDS nie powinien samodzielnie rozstrzygać konfliktów w sposób powodujący utratę danych.

### 20.3 Operacje wieloetapowe

Operacje wieloetapowe muszą mieć jawnie określoną semantykę częściowego powodzenia.

API nie może sugerować atomowości, której PDS nie gwarantuje.

Jeżeli PDS udostępnia operacje grupowe lub transakcyjne, ich zakres i gwarancje muszą być jawnie określone.

## 21. Wersjonowanie API

PDS-Client-API jest wersjonowane niezależnie od PDS-NP.

Zmiana wersji PDS-Client-API nie musi oznaczać zmiany protokołu komunikacji między instancjami PDS.

Wszystkie sposoby dostępu obsługujące tę samą wersję API muszą zapewniać zgodną semantykę operacji.

Rozszerzenia powinny być kompatybilne wstecz, o ile nie zmieniają znaczenia istniejących operacji.

Klient powinien mieć możliwość sprawdzenia:

- wersji API;
- obsługiwanych możliwości;
- wersji modelu danych;
- statusu lokalnej instancji PDS.

Nieznane funkcje opcjonalne mogą być ignorowane. Funkcje wymagane przez klienta muszą być sprawdzone przed wykonaniem operacji, które od nich zależą.

## 22. Bezpieczeństwo i prywatność

Implementacja musi:

- egzekwować uprawnienia dla każdej operacji;
- ograniczać dostęp do IPC na poziomie systemu operacyjnego;
- chronić klucze prywatne klientów;
- nie ujawniać sekretów w logach;
- walidować wszystkie dane wejściowe;
- ograniczać rozmiary żądań i transferów;
- ograniczać koszt zapytań;
- izolować błędy jednego klienta od pozostałych;
- rejestrować istotne operacje zgodnie z polityką historii i audytu PDS;
- umożliwiać unieważnienie klienta oraz jego aktywnych sesji.

### 22.1 Granica zaufania systemu operacyjnego

Mechanizmy uwierzytelniania PDS nie zastępują zabezpieczeń systemu operacyjnego.

Jeżeli złośliwy proces działa z tym samym użytkownikiem systemowym i może uzyskać dostęp do kluczy prywatnych aplikacji, może próbować podszyć się pod te aplikacje.

Dlatego implementacja powinna:

- przechowywać klucze w możliwie bezpiecznym magazynie;
- stosować uprawnienia systemu plików i ACL;
- ograniczać dostęp do kanałów IPC;
- nie współdzielić kluczy pomiędzy niezależnymi aplikacjami;
- stosować osobne procesy dla komponentów niezaufanych.

Lokalny dostęp nie oznacza automatycznie zaufania.

## 23. Zgodność

Implementacja zgodna z PDS-Client-API:

- udostępnia publiczny kontrakt operacji PDS;
- obsługuje co najmniej jeden sposób lokalnego dostępu;
- rozdziela In-process i Local IPC od PDS-NP;
- nie wymaga handshake PDS-NP od lokalnego klienta;
- rozdziela uwierzytelnianie klienta od uwierzytelniania peerów;
- obsługuje rejestrację klientów;
- obsługuje uwierzytelnianie klientów przez challenge-response;
- weryfikuje podpisy klientów;
- egzekwuje autoryzację dla wszystkich operacji;
- obsługuje sesje i ich unieważnianie;
- zachowuje semantykę historii i pochodzenia PDS;
- obsługuje kontrolę konfliktów przy zapisie;
- udostępnia spójny model błędów;
- nie wymaga od klienta bezpośredniego dostępu do wewnętrznego magazynu danych;
- nie pozwala klientowi ominąć mechanizmów bezpieczeństwa PDS.

Jeżeli implementacja obsługuje więcej niż jeden sposób dostępu, musi zapewniać zgodność ich zachowania.

## 24. Granica odpowiedzialności

PDS-Client-API definiuje sposób korzystania z lokalnej instancji PDS przez aplikacje.

PDS-NP definiuje sposób komunikacji pomiędzy niezależnymi instancjami PDS.

Lokalny klient:

- nie jest peerem PDS-NP;
- nie przechodzi handshake PDS-NP;
- nie musi posiadać sieciowej tożsamości PDS;
- nie uczestniczy bezpośrednio w wymianie wiadomości synchronizacyjnych;
- nie ma bezpośredniej kontroli nad transportem peer-to-peer.

Lokalna instancja PDS odpowiada za dostęp klienta do danych, rejestrację i uwierzytelnianie klientów, autoryzację, trwałość zmian, historię, pochodzenie oraz ewentualną komunikację z innymi instancjami.
