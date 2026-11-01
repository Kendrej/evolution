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
- Agenci rozmnażają się tylko wtedy, gdy dwaj z tego samego plemienia dotkną się we własnej bazie. Sieć ma się nauczyć tam wracać, więc nie dodajemy ręcznego powrotu do bazy.
- W bazach nie ma jedzenia. Jedzenie z pola zajętego może zjeść tylko właściciel, a z neutralnego każdy.

## Uruchamianie
- `cargo run --release` do dłuższych przebiegów. Spacja zmienia prędkość symulacji (x1, x10, x50).
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

## Stan projektu
Zrobione: statystyki i zapis do CSV, losowa mapa, minimalna populacja plemienia, przyspieszenie, struktura `Brain` i `Action`, zmysły agenta `Senses`, wykresy z plików CSV.

`Brain` to sieć neuronowa z 13 wejściami, 8 neuronami ukrytymi i 2 wyjściami (130 wag). `Senses` zawiera kierunek i odległość do najbliższego gotowego partnera z plemienia.

Dzieci dziedziczą wagi po rodzicach z mutacją. W treningu odrodzeni agenci dostają wagi z listy 20 ostatnich rodziców swojego plemienia, z mutacją, a przy pustej liście nową, losową sieć. Lista jest zapisywana w modelu.

Stare modele usunięto przy dodaniu zmysłu partnera, więc `models/main.json` powstanie dopiero po oznaczeniu nowej populacji gwiazdką.

Porównanie 5 ziaren po 500 000 kroków (`python tools/compare_inheritance.py 500000 5`, suma z 5 ziaren): lista rodziców dała 2214 narodzin wobec 392 przy kopiowaniu od losowego żyjącego i 667 bez dziedziczenia. W ostatnich 100 000 kroków było to 550 narodzin wobec 52. Odrodzenia nie spadły (1253 wobec 1088 w ostatnich 100 000 kroków), a zajęte pola spadły o około 30%.

Pomiar `in_base` i `ready_in_base` (średnio na plemię, ostatnie 100 000 kroków): z listą rodziców w bazie jest 1,39 z 6,05 żywych, a gotowych 0,27, bez dziedziczenia 1,08 z 5,04 i 0,23. Agenci nie trzymają się więc bazy bardziej niż losowe sieci.

Długi test, 5 ziaren po 2 000 000 kroków (`runs/comparisons/2026-11-01_10-23_dlugi_test/`), pokazał uczenie: w ostatnich 500 000 kroków narodziny wynoszą 6914 z listą rodziców i 609 bez dziedziczenia, odrodzenia 3551 i 6330, a żywi na plemię 13,4 i 5,1. Bez dziedziczenia nic się nie zmienia w czasie. Narodziny przegoniły odrodzenia w sumie, ale nie na ziarnie 1004, a wzrost zwalnia. Gotowych jest 8,4 na plemię, z czego w bazie tylko 0,44, więc gotowy agent rozmnaża się średnio raz na około 6000 kroków, choć przerwa trwa 600. Zajęte pola są na końcu podobne w obu trybach (721 i 795 na plemię), więc wcześniejszy spadek był przejściowy.

## Następne kroki
1. Limit wieku agenta (np. 15 000 kroków), żeby szybciej wymieniać pokolenia i mocniej nagradzać powrót do bazy. Sprawdzić na tych samych ziarnach przez 2 000 000 kroków.
2. Koszt energii za ruch i strojenie. Dobry znak to rosnące narodziny przy spadających odrodzeniach.
3. Sprawdzenie, czy przejmowanie pól jest celowe. Pola zajmują też losowe sieci, więc samo `owned_tiles` tego nie rozstrzyga.
