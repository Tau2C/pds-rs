# Personal Data Store (PDS)

## Specyfikacja ogólna

**Wersja:** 1.0.0  
**Status:** Specyfikacja architektury  
**Zakres:** Uniwersalny, osobisty magazyn danych dla wielu aplikacji

---

## 1. Cel i przeznaczenie

Personal Data Store (PDS) jest uniwersalnym, należącym do użytkownika magazynem danych, z którego może korzystać wiele niezależnych aplikacji. Jego zadaniem jest przechowywanie danych w sposób trwały, przenośny i niezależny od cyklu życia aplikacji, która je utworzyła.

PDS stanowi wspólną warstwę danych dla aplikacji użytkownika. Aplikacje mogą dodawać własne typy, właściwości i reguły interpretacji danych, ale nie przejmują własności nad podstawowym magazynem ani nie definiują jego ogólnego modelu.

Dane zapisane w PDS powinny pozostać dostępne po odinstalowaniu, przepisaniu lub zastąpieniu aplikacji, która je pierwotnie utworzyła. Inna uprawniona aplikacja powinna móc odczytać te same dane i wykorzystać je w innym kontekście.

PDS nie jest aplikacją do konkretnego zadania. Jest infrastrukturą danych, na której takie aplikacje mogą działać.

## 2. Założenia i zasady nadrzędne

1. **Własność użytkownika.** Dane należą do właściciela osobistej przestrzeni danych, a nie do aplikacji.
2. **Niezależność od aplikacji.** Dane i ich historia nie mogą zależeć od istnienia konkretnego klienta.
3. **Wspólny model.** Aplikacje korzystają ze wspólnego modelu encji, właściwości, relacji, plików i historii.
4. **Elastyczność.** PDS nie narzuca jednej dziedziny ani zamkniętego słownika typów.
5. **Jawne uprawnienia.** Dostęp aplikacji i urządzeń jest kontrolowany i możliwy do cofnięcia.
6. **Praca offline.** Użytkownik może korzystać z danych bez stałego połączenia z innymi urządzeniami lub usługami.
7. **Synchronizacja historii.** Zmiany powstałe na różnych zaufanych urządzeniach powinny dać się połączyć bez bezwarunkowego nadpisywania jednej wersji drugą.
8. **Pochodzenie danych.** Istotne zmiany, źródła i procesy, które wytworzyły dane, powinny być możliwe do prześledzenia.
9. **Trwałość.** Historia jest zachowywana bezterminowo, o ile właściciel nie podejmie świadomej decyzji o zmianie polityki retencji.
10. **Przenośność.** Dane muszą dać się wyeksportować i odtworzyć bez uzależnienia od konkretnego dostawcy lub aplikacji.

## 3. Zakres odpowiedzialności

### 3.1. PDS odpowiada za

- trwałe przechowywanie danych należących do użytkownika;
- identyfikację i organizację danych w postaci encji i relacji;
- przechowywanie właściwości o określonych wartościach;
- przechowywanie plików i innych dużych treści oraz łączenie ich z danymi;
- historię zmian, rewizje i możliwość odtworzenia wcześniejszego stanu;
- reprezentowanie pochodzenia danych i wyników przetwarzania;
- kontrolę dostępu aplikacji;
- współpracę z zaufanymi urządzeniami i synchronizację;
- ochronę integralności, kopie zapasowe, eksport i odtwarzanie danych.

### 3.2. PDS nie odpowiada za

- konkretne interfejsy użytkownika i procesy biznesowe;
- interpretację znaczenia danych w określonej dziedzinie;
- OCR, parsowanie dokumentów, rozpoznawanie obiektów ani uruchamianie modeli AI;
- automatyczne podejmowanie decyzji domenowych;
- definiowanie jednego obowiązującego słownika kategorii i typów dla wszystkich aplikacji;
- utrzymywanie indeksów wyszukiwania specyficznych dla konkretnej aplikacji;
- gwarantowanie poprawności semantycznej danych wprowadzonych przez aplikację.

PDS może udostępniać ogólne mechanizmy, z których aplikacje korzystają przy realizacji tych zadań. Nie powinien jednak zawierać logiki należącej do konkretnej aplikacji.

## 4. Model pojęciowy danych

PDS przechowuje dane jako graf powiązanych elementów. Podstawowe pojęcia są wspólne dla wszystkich aplikacji.

### 4.1. Przestrzeń danych

Przestrzeń danych jest niezależnym obszarem należącym do użytkownika. Określa granicę własności, zaufania, uprawnień, historii i synchronizacji.

Jedna przestrzeń może być używana przez wiele aplikacji i wiele zaufanych urządzeń. Dostęp aplikacji nie oznacza automatycznie dostępu do wszystkich danych ani uprawnień administracyjnych.

### 4.2. Encja

Encja reprezentuje dowolny identyfikowalny element informacji. Może oznaczać rzecz, zdarzenie, osobę, dokument, konto, miejsce, obserwację, zasób cyfrowy lub pojęcie domenowe.

Każda encja posiada trwałą tożsamość i może mieć:

- typ lub klasyfikację;
- właściwości;
- relacje z innymi encjami;
- powiązane treści lub pliki;
- historię zmian;
- informacje o pochodzeniu.

PDS nie wymaga wcześniejszego zarejestrowania typu encji, aby można było ją utworzyć. Nieznane typy i właściwości muszą być zachowywane.

### 4.3. Właściwości

Właściwości opisują cechy encji. Każda właściwość ma nazwę i wartość. Model wartości powinien umożliwiać przechowywanie danych tekstowych, liczbowych, logicznych, dat, czasu, kwot, ilości, odwołań do innych encji, plików, kolekcji i struktur złożonych.

Nazwy właściwości mogą korzystać z przestrzeni nazw. PDS nie narzuca jednego schematu nazewnictwa ani znaczenia poszczególnych nazw.

Aplikacje mogą definiować schematy opisujące oczekiwane typy, wymagane pola, dozwolone wartości i reguły walidacji. Schematy ułatwiają interpretację i walidację, ale nie powinny powodować automatycznego usuwania nieznanych danych ani uniemożliwiać zachowania danych pochodzących z innej aplikacji.

Wartości wieloelementowe mogą być reprezentowane jako kolekcje albo przez osobne encje i relacje — zależnie od znaczenia danych. PDS nie powinien wymuszać kodowania list w tekstach za pomocą arbitralnych separatorów.

### 4.4. Relacje

Relacja łączy co najmniej dwie encje i opisuje związek między nimi. Relacje są elementami pierwszej klasy, a nie tekstowymi identyfikatorami ukrytymi we właściwościach.

Relacja posiada własną tożsamość, wskazuje encje, które łączy, i ma typ opisujący jej znaczenie. Może również posiadać własne właściwości, np. datę, źródło, rolę, kwotę, poziom pewności lub komentarz.

Typy relacji są otwarte. PDS może udostępniać ogólne konwencje i opcjonalne schematy, ale nie powinien narzucać jednej zamkniętej ontologii.

### 4.5. Pliki i treści

PDS może przechowywać treści binarne, dokumenty, zdjęcia, nagrania i inne duże zasoby jako niezależne obiekty treści. Encje mogą odwoływać się do tych zasobów z określeniem ich roli.

Jeden zasób może być używany przez wiele encji. Identyczne bajty mogą być przechowywane fizycznie tylko raz, o ile nie narusza to granic bezpieczeństwa ani polityki prywatności.

Tożsamość logiczna zasobu jest odrębna od jego skrótu kontrolnego. Skrót służy do sprawdzania integralności i wykrywania identycznej zawartości; nie zastępuje tożsamości encji ani zasobu.

### 4.6. Schematy

Schemat opisuje konwencję danych: typy encji, właściwości, typy wartości, reguły walidacji i konwencje relacji. Schematy są wersjonowane i mogą być rozwijane niezależnie od danych.

Zmiana schematu nie może po cichu niszczyć danych zgodnych ze starszą wersją ani danych nieznanych danemu schematowi. Migracje semantyczne powinny być jawne i możliwe do prześledzenia.

## 5. Historia i integralność zmian

Zmiany danych powinny tworzyć trwałą historię. Aktualny stan encji jest widokiem wynikającym z zapisanych zmian, a nie jedyną zachowaną reprezentacją danych.

Historia powinna umożliwiać:

- ustalenie, co się zmieniło, kiedy i przez jakie urządzenie lub aplikację;
- odtworzenie wcześniejszej wersji encji i relacji;
- wskazanie zmian, które doprowadziły do aktualnego stanu;
- połączenie historii powstałych niezależnie na różnych urządzeniach;
- zachowanie konfliktujących zmian do czasu ich rozwiązania.

Historia ma charakter append-only: zapisane operacje i zatwierdzone jednostki zmian nie są modyfikowane w miejscu. Korekty, usunięcia i przywrócenia są kolejnymi zmianami. Zmiany powinny być grupowane w atomowe jednostki historii, które mogą tworzyć skierowany graf acykliczny (DAG). Historia może się rozgałęziać, gdy kilka urządzeń modyfikuje dane offline. Scalanie nie powinno bezwarunkowo stosować zasady „ostatni zapis wygrywa”.

Polityka rozwiązywania konfliktów zależy od semantyki danych. Możliwe jest automatyczne połączenie zmian, zachowanie kilku alternatyw albo wymaganie decyzji użytkownika lub aplikacji domenowej. Usunięcie danych powinno pozostawiać informację historyczną, a przywrócenie powinno być nową zmianą.

Integralność zmian musi być możliwa do zweryfikowania. Mechanizm kontroli integralności powinien chronić zarówno historię, jak i przechowywane pliki.

## 6. Pochodzenie danych i przetwarzanie

PDS powinien umożliwiać rejestrowanie pochodzenia informacji bez narzucania tego, jak dana aplikacja wykonuje przetwarzanie.

Proces przetwarzania może być reprezentowany jako aktywność, która wskazuje:

- wejściowe encje, pliki lub ich konkretne rewizje;
- zastosowany proces, narzędzie lub usługę;
- wersję procesu i istotną konfigurację;
- czas i kontekst wykonania;
- utworzone wyniki;
- status oraz ewentualny błąd;
- relację między wynikiem a źródłami.

PDS przechowuje opis procesu i jego pochodzenie. Nie musi sam uruchamiać procesu. Przykładowo aplikacja może wykonać OCR, analizę AI lub dopasowanie danych, a następnie zapisać w PDS wejścia, wyniki i powiązania między nimi.

Wynik pochodny nie powinien usuwać ani zastępować źródła. Ponowne przetworzenie powinno pozostawiać możliwość odtworzenia wcześniejszych rezultatów i ustalenia, na jakiej wersji danych zostały oparte.

## 7. Własność, tożsamość i uprawnienia

PDS rozróżnia właściciela przestrzeni danych, zaufane urządzenia oraz aplikacje korzystające z danych.

Właściciel kontroluje przestrzeń i decyduje o zaufaniu urządzeń oraz zakresie dostępu aplikacji. Aplikacja otrzymuje wyłącznie jawnie przyznane uprawnienia.

Uprawnienia mogą ograniczać:

- odczyt i zapis danych;
- zakres encji, typów lub przestrzeni nazw;
- dostęp do historii;
- odczyt i zapis plików;
- tworzenie relacji;
- eksport danych;
- zarządzanie uprawnieniami i urządzeniami.

Domyślną polityką jest brak dostępu bez przyznanego uprawnienia. Cofnięcie uprawnienia blokuje przyszły dostęp przez PDS, ale nie może cofnąć danych, które aplikacja wcześniej legalnie skopiowała lub wyeksportowała.

Urządzenie dopuszczone do synchronizacji i aplikacja uruchomiona na tym urządzeniu są odrębnymi podmiotami zaufania. Samo zaufanie do urządzenia nie musi oznaczać nieograniczonego dostępu każdej aplikacji na tym urządzeniu.

### 7.1. Szyfrowanie danych end-to-end (E2EE)

PDS umożliwia kryptograficzną ochronę poufności danych przed nieuprawnionym dostępem w spoczynku, w tranzycie oraz na węzłach pośredniczących (relay).

Wyróżnia się dwa tryby szyfrowania jednostek zmian (Change):

1. **Tryb hybrydowy (`EncryptedPayload`)**:
   - Struktura grafu DAG (ChangeId, rodzice) oraz typy encji pozostają jawne i indeksowalne.
   - Wartości właściwości (`PropertyValue`) oraz zawartość powiązanych obiektów binarnych (bloby) są szyfrowane symetrycznie za pomocą AEAD (np. ChaCha20-Poly1305).
   - Klucze szyfrujące (DEK) są udostępniane wyłącznie uprawnionym aplikacjom i urządzeniom.

2. **Tryb ślepego grafu (`BlindChange`)**:
   - Wszystkie operacje, typy encji, właściwości oraz metadane domenowe są zaszyfrowane w całości wewnątrz jednego nienaruszalnego bloku AEAD.
   - Węzły sieciowe i pośrednicy synchronizacji widzą wyłącznie identyfikatory zmian oraz skierowane krawędzie grafu DAG, co pozwala na replikację historii bez znajomości jakiejkolwiek treści.


## 8. Synchronizacja i praca offline

PDS powinien umożliwiać pracę na wielu zaufanych urządzeniach bez wymogu stałej łączności.

Każde urządzenie może lokalnie rejestrować zmiany. Po uzyskaniu łączności urządzenia wymieniają brakujące elementy historii, wymagane zasoby treści i informacje potrzebne do uzgodnienia stanu.

Synchronizacja może wykorzystywać bezpośrednią komunikację peer-to-peer lub mechanizmy pośredniczące, ale nie może wymagać jednego centralnego serwera jako jedynego źródła prawdy.

Synchronizacja powinna:

- zachowywać pochodzenie zmian;
- weryfikować integralność otrzymanych danych;
- obsługiwać rozgałęzienia i scalanie historii;
- nie uznawać zegara urządzenia za jedyne źródło kolejności zmian;
- wznawiać przerwane transfery;
- wykrywać urządzenia cofnięte lub odwołane;
- nie wymagać centralnego serwera jako jedynego źródła prawdy.

Synchronizowany jest model danych i historia, a nie kopia wewnętrznej bazy konkretnej implementacji. Indeksy, cache i inne widoki pochodne mogą być odtwarzane lokalnie.

Zakres replikacji i dostęp aplikacji to dwa osobne zagadnienia. Urządzenie może przechowywać kopię przestrzeni danych, podczas gdy konkretna aplikacja otrzymuje tylko ograniczone uprawnienia.

## 9. Prywatność i bezpieczeństwo

PDS musi chronić dane przed nieuprawnionym odczytem, modyfikacją i przypadkową utratą.

Wymagania ogólne:

- opcjonalne szyfrowanie danych przechowywanych i przesyłanych;
- uwierzytelnianie urządzeń i aplikacji;
- autoryzacja wszystkich operacji dostępu;
- ochrona integralności historii i zasobów;
- bezpieczne zarządzanie kluczami oraz możliwość ich rotacji;
- jawne procedury odzyskiwania danych i kluczy;
- ograniczanie ujawniania metadanych wrażliwych;
- izolacja danych pomiędzy przestrzeniami;
- brak domyślnego wysyłania danych do usług zewnętrznych.

Szyfrowanie nie zastępuje kopii zapasowej. Utrata wszystkich kopii kluczy może uniemożliwić odzyskanie zaszyfrowanych danych.

## 10. Kopie zapasowe, eksport i odtwarzanie

Właściciel musi móc utworzyć kompletną kopię przestrzeni danych i odtworzyć ją na innym urządzeniu lub w innej zgodnej implementacji.

Kopia powinna obejmować:

- encje, właściwości i relacje;
- historię i informacje o rewizjach;
- schematy i ich wersje;
- zasoby treści oraz powiązania;
- informacje o pochodzeniu i przetwarzaniu;
- niezbędne metadane przestrzeni danych.

Dane zewnętrzne, które nie zostały zaimportowane do PDS, powinny być jawnie oznaczone jako zależności, a nie jako elementy objęte kopią.

Eksport musi mieć udokumentowany, wersjonowany format. Odtwarzanie powinno weryfikować kompletność i integralność danych oraz raportować brakujące zasoby.

## 11. Cykl życia i retencja

Dane należą do właściciela, a nie do aplikacji. Usunięcie aplikacji nie usuwa danych zapisanych przez nią w PDS.

Usunięcie encji powinno być reprezentowane w historii. Trwałe usunięcie danych i ich historii może być realizowane wyłącznie przez jawne działanie właściciela i musi uwzględniać powiązane zasoby, kopie oraz ograniczenia synchronizacji.

Domyślnie historia nie podlega automatycznemu czyszczeniu po upływie czasu. Mechanizmy ograniczania rozmiaru danych nie mogą po cichu usuwać jedynych kopii informacji lub historii.

## 12. Integracja z aplikacjami

Aplikacja korzystająca z PDS jest klientem i właścicielem swojej logiki domenowej, ale nie niezależnym źródłem prawdy o danych, które zapisała w PDS.

Aplikacja może:

- tworzyć i odczytywać encje;
- dodawać i aktualizować właściwości;
- tworzyć relacje;
- dołączać i odczytywać zasoby treści;
- odczytywać historię w zakresie przyznanych uprawnień;
- zapisywać wyniki procesów i ich pochodzenie;
- definiować własne konwencje typów, atrybutów i relacji.

Aplikacja nie powinna:

- modyfikować wewnętrznego magazynu PDS poza jego publiczną granicą;
- tworzyć własnej, konkurencyjnej kopii autorytatywnego modelu danych;
- zakładać, że nieznane właściwości lub typy można usunąć;
- traktować lokalnego indeksu albo cache jako jedynego źródła trwałych danych.

PDS może być udostępniany klientom w różnych formach, np. jako biblioteka osadzona w aplikacji albo usługa lokalna wspólna dla kilku aplikacji. Są to sposoby dostępu do tego samego modelu i zasad własności, a nie odrębne systemy danych.

## 13. Wymagania jakościowe

PDS powinien być:

- **trwały** — zatwierdzone dane nie znikają po restarcie ani usunięciu aplikacji;
- **spójny** — operacje wieloelementowe są zapisywane jako całość;
- **rozszerzalny** — nowe dziedziny nie wymagają zmiany ogólnego modelu;
- **przenośny** — dane można eksportować i odtwarzać;
- **audytowalny** — można ustalić historię i pochodzenie zmian;
- **bezpieczny** — aplikacje mają tylko jawnie przyznany dostęp;
- **odporny na brak łączności** — praca lokalna nie wymaga dostępności innych urządzeń;
- **odporny na konflikty** — niezależne zmiany nie są bezpowrotnie gubione;
- **niezależny technologicznie** — format danych i kontrakt nie są tożsame z konkretną bazą, językiem lub protokołem.

## 14. Kryteria akceptacji architektury

Architektura PDS spełnia założenia, jeżeli można wykazać, że:

1. Dwie niezależne aplikacje mogą korzystać z tych samych encji i relacji.
2. Usunięcie jednej aplikacji nie powoduje usunięcia danych.
3. Nowy, nieznany typ encji i właściwość mogą zostać zachowane.
4. Plik może być powiązany z wieloma encjami bez niepotrzebnego powielania treści.
5. Zmiany danych pozostawiają historię pozwalającą odtworzyć wcześniejszy stan.
6. Niezależne zmiany z dwóch urządzeń mogą zostać zsynchronizowane bez cichej utraty konfliktujących danych.
7. Aplikacja bez odpowiedniego uprawnienia nie może odczytać ani zmienić danych.
8. Cofnięcie uprawnienia blokuje dalszy dostęp aplikacji.
9. Wynik przetwarzania może wskazywać dokładne dane wejściowe i źródło.
10. Pełny eksport może zostać odtworzony bez oryginalnej aplikacji.
11. Brak zewnętrznego pliku jest raportowany jako brak zależności, a nie jako usunięcie danych.
12. Historia nie jest automatycznie usuwana na podstawie samego wieku.

## 15. Zasada architektoniczna

PDS przechowuje **dane, ich relacje, historię, pliki, uprawnienia i pochodzenie**. Aplikacje definiują **znaczenie domenowe, sposób przetwarzania i interakcję z użytkownikiem**.

PDS ma pozostać użyteczny niezależnie od tego, jakie aplikacje z niego korzystają. Żadna aplikacja — w tym OSP Hub — nie może stać się warunkiem odczytu, zachowania lub odzyskania danych zapisanych w PDS.
