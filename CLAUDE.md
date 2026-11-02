# Ewolucja

Symulacja plemion w Ruście (macroquad). Celem jest neuroewolucja: każdy agent ma własną sieć neuronową, a jej wagi ewoluują przez dobór i mutację.

## Sposób pracy
- Odpowiadaj po polsku, prostym językiem.
- Claude pisze kod, a użytkownik go nadzoruje. Rób małe zmiany pojedynczo.
- Po zmianie opisz strukturę: co zmieniło się w którym pliku i jakie funkcje. Nie wklejaj kodu, chyba że użytkownik o to poprosi.
- Commituj i pushuj tylko na prośbę, bezpośrednio na `master`, bez dodatkowych gałęzi.
- Nie dodawaj do commitów linijki `Co-Authored-By`.

## Założenia
- Teraz liczy się dobra podstawa pod sieć, a nie dopracowane zachowanie. Decyzje agentów mają dać się łatwo zastąpić siecią.
- Agenci rozmnażają się, gdy dwaj z tego samego plemienia dotkną się na polu należącym do plemienia, także w bazie.
- Baza to serce plemienia: jej utrata oznacza wymarcie plemienia do końca rundy.
- W bazach nie ma jedzenia. Jedzenie z pola zajętego może zjeść tylko właściciel, a z neutralnego każdy.

## Uruchamianie
- `cargo run --release` do dłuższych przebiegów. Spacja zmienia prędkość symulacji (x1, x10, x50, x100).
- Po starcie menu pozwala wybrać trening (nowa populacja albo zapisany model) lub demo (oglądanie modelu bez nauki i bez zapisu). Esc w trakcie, po potwierdzeniu, wraca do menu.
- Modele to wagi żywych agentów z podziałem na plemiona, zapisywane co 50 000 kroków. Każda nazwana populacja ma folder `models/<czas>_seed_<ziarno>_<tryb>/`, a kontynuacja treningu zapisuje do tego samego folderu.
- Gwiazdka w menu oznacza model główny. Jego kopia to `models/main.json`, jedyny model w gicie, aktualizowany przy każdym zapisie tej populacji. Foldery populacji są poza repozytorium.
- Przebiegi bez okna zapisują się w `models/comparisons/`, nie ma ich w menu i nie zmieniają modelu głównego.
- Ziarno jest losowe. Żeby powtórzyć przebieg, ustaw `FIXED_SEED` w `src/main.rs`.
- Pomiary zapisują się co 1000 kroków do plików CSV w `runs/`, poza repozytorium:
  - `runs/latest/` to ostatni trening z okna. Po jego zakończeniu wykres PNG rysuje się sam, a nowy trening przenosi poprzedni do `runs/training/`. Pliki bez pomiarów są wtedy usuwane.
  - `runs/comparisons/` zawiera porównania. Każde ma folder `<data>_<opis>/` z plikami CSV, `wykres.png` i `wyniki.txt`.
- `python tools/plot_run.py` ręcznie rysuje wykresy ostatniego treningu i zapisuje PNG obok pliku CSV. Wymaga `matplotlib`.
- `python tools/compare_inheritance.py <kroki> <liczba_ziaren> <opis>` uruchamia porównanie.
- `python tools/compare_rounds.py <kroki> <liczba_ziaren> <opis>` trenuje warianty długości rundy i ciągły świat, a potem wystawia ich modele w meczach bez nauki.
- Bez okna: `--headless <ziarno> <kroki> <inherit|random> [kroki_rundy, 0 to bez rund]` oraz `--match <model_1> <model_2> <ziarno> <kroki>`.

## Stan projektu
Zrobione: statystyki i zapis do CSV, losowa mapa, minimalna populacja plemienia, przyspieszenie, struktura `Brain` i `Action`, zmysły agenta `Senses`, wykresy z plików CSV.

`Brain` to sieć neuronowa z 16 wejściami, 8 neuronami ukrytymi i 3 wyjściami (163 wagi). `Senses` zawiera kierunek i odległość do najbliższego gotowego partnera z plemienia oraz kierunek, odległość i energię najbliższego wroga. Modele zapisane przed dodaniem walki się nie wczytają.

Walka: trzecie wyjście sieci to atak na najbliższego wroga w zasięgu, można go użyć co 30 kroków i kosztuje 0,25 energii także przy pudle. Trafienie zabiera 25 energii, a połowa trafia do atakującego. Zabójca trafia na listę rodziców z szansą 25%. Wolno atakować także w cudzej bazie. Ranking plemion liczy zajęte pola, a przy remisie żywych. Jest w panelu statystyk i na końcu przebiegu bez okna. `MIN_TRIBE_POPULATION` nadal nie pozwala plemieniu wymrzeć.

Rundy: co 50 000 kroków (poza demo) powstaje nowa mapa i nowe jedzenie, a każde plemię zaczyna od 10 agentów w bazie. Wagi bierze z listy rodziców i żywych swojego plemienia, z mutacją, ale ostatnie w rankingu dostaje wagi i listę rodziców zwycięzcy. W CSV `owned_tiles` i `alive` spadają więc na początku każdej rundy.

Zdobywanie baz (tylko w treningu z rundami): gdy w bazie są wrogowie, a nie ma obrońców, oblężenie rośnie o 1 na krok, a w każdej innej sytuacji spada o 1. Po 600 krokach baza upada. Plemię ginie wtedy do końca rundy, jego pola stają się neutralne, a baza zostaje przyciemniona. Zdobywcą jest plemię z największą liczbą agentów w bazie, co trafia do kolumny `bases_taken` w CSV. Oblężona baza nie przyjmuje odrodzeń. Runda kończy się wcześniej, gdy zostanie jedno plemię. Pierwszy test (ziarno 1001, 300 000 kroków) dał 4 upadki baz w 6 rundach, choć agenci nie widzą wrogich baz.

Dzieci dziedziczą wagi po rodzicach z mutacją. W treningu odrodzeni agenci dostają wagi z listy 20 ostatnich rodziców swojego plemienia, z mutacją, a przy pustej liście nową, losową sieć. Lista jest zapisywana w modelu.

Stare modele usunięto przy dodaniu zmysłu partnera, więc `models/main.json` powstanie dopiero po oznaczeniu nowej populacji gwiazdką.

Porównanie 5 ziaren po 500 000 kroków (`python tools/compare_inheritance.py 500000 5`, suma z 5 ziaren): lista rodziców dała 2214 narodzin wobec 392 przy kopiowaniu od losowego żyjącego i 667 bez dziedziczenia. W ostatnich 100 000 kroków było to 550 narodzin wobec 52. Odrodzenia nie spadły (1253 wobec 1088 w ostatnich 100 000 kroków), a zajęte pola spadły o około 30%.

Pomiar `in_base` i `ready_in_base` (średnio na plemię, ostatnie 100 000 kroków): z listą rodziców w bazie jest 1,39 z 6,05 żywych, a gotowych 0,27, bez dziedziczenia 1,08 z 5,04 i 0,23. Agenci nie trzymają się więc bazy bardziej niż losowe sieci.

Długi test, 5 ziaren po 2 000 000 kroków (`runs/comparisons/2026-11-01_10-23_dlugi_test/`), pokazał uczenie: w ostatnich 500 000 kroków narodziny wynoszą 6914 z listą rodziców i 609 bez dziedziczenia, odrodzenia 3551 i 6330, a żywi na plemię 13,4 i 5,1. Bez dziedziczenia nic się nie zmienia w czasie. Narodziny przegoniły odrodzenia w sumie, ale nie na ziarnie 1004, a wzrost zwalnia. Gotowych jest 8,4 na plemię, z czego w bazie tylko 0,44, więc gotowy agent rozmnaża się średnio raz na około 6000 kroków, choć przerwa trwa 600. Zajęte pola są na końcu podobne w obu trybach (721 i 795 na plemię), więc wcześniejszy spadek był przejściowy.

Limit wieku 15 000 kroków (`runs/comparisons/2026-11-01_16-54_limit_wieku/`, ostatnie 500 000 kroków): narodziny 7699 wobec 6914 bez limitu, odrodzenia 6253 wobec 3551, żywi na plemię 8,5 wobec 13,4.

Walka i mniej jedzenia (`runs/comparisons/2026-11-01_18-40_walka_dlugi/`, ostatnie 500 000 kroków): narodziny 1853 przy dziedziczeniu i 137 bez niego, odrodzenia 19341 i 20105, zabójstwa 1112 i 346. Populacja stoi na minimum (5,1 na plemię), a 1052 z 1112 zabójstw pochodzi z 2 ziaren. Narodziny są ponad 4 razy niższe niż przed walką.

Strojenie (5 ziaren po 2 000 000 kroków, ostatnie 500 000 kroków). Koszt ataku 0,25 i jedzenie dające 40 energii (`2026-11-01_20-26_dlugi_d_prog60`, obecne ustawienia): narodziny 2488 wobec 348 bez dziedziczenia, odrodzenia 12307 i 14036, zabójstwa 543 i 478, żywi na plemię 5,3. Gotowych jest 9,2 z 21 żywych, ale w bazie tylko 0,77 w sumie, więc ograniczeniem jest teraz powrót do bazy, a nie energia. Próg rozmnażania 70 energii (`2026-11-01_20-09_dlugi_e_prog70`): zabójstwa 1736 wobec 488, ale narodziny tylko 593. Około 0,4% ataków trafia, więc agenci atakują prawie na ślepo. Przy limicie jedzenia poniżej 2000 agenci głodują, a 3000 nic nie zmienia.

Rozmnażanie na własnych polach (`2026-11-01_20-52_dlugi_wlasne_pola`, ostatnie 500 000 kroków): narodziny 26513 wobec 1451 bez dziedziczenia, odrodzenia 8296 i 13701, zabójstwa 2325 i 515, żywi na plemię 14,5 i 5,1. Narodziny przewyższają odrodzenia na każdym ziarnie, ale ich wzrost prawie stanął w ostatnim okresie. Na końcu jedno plemię często dominuje (np. 2792 pola wobec 45), a bez dziedziczenia zwykle są wyrównane. Trafia około 1% ataków.

Rundy wobec ciągłego świata (`2026-11-01_22-23_rundy_dlugi`, 5 ziaren po 2 000 000 kroków, potem mecze po 100 000 kroków w obie strony na 3 nowych mapach): rundy 50 000 wygrały 15/15 (pola 69018 wobec 17450), 100 000 i 200 000 po 12/15, oba przegrywając na ziarnie 1001. Mecz przypomina start rundy, więc format sprzyja modelom z rund. Około 1% ataków nadal trafia.

## Następne kroki
1. Zmysł kierunku do najbliższej wrogiej bazy (2 nowe wejścia, trening od nowa) i pomiar, czy agenci uczą się atakować bazy.
2. Zachowywanie najlepszych modeli z rund.
3. Cechy agentów dopiero po działających rundach.
