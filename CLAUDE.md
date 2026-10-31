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
- Pomiary zapisują się co 1000 kroków do plików CSV w `runs/`, poza repozytorium.
- `python tools/plot_run.py` rysuje wykresy najnowszego przebiegu i zapisuje PNG obok pliku CSV. Wymaga `matplotlib`.

## Stan projektu
Zrobione: statystyki i zapis do CSV, losowa mapa, minimalna populacja plemienia, przyspieszenie, struktura `Brain` i `Action`, zmysły agenta `Senses`, wykresy z plików CSV.

`Brain` to sieć neuronowa z 11 wejściami, 8 neuronami ukrytymi i 2 wyjściami (114 wag). Dzieci i odrodzeni agenci dziedziczą wagi z mutacją.

Porównanie 5 ziaren po 500 000 kroków (`python tools/compare_inheritance.py 500000 5`) nie pokazało jeszcze wyraźnego efektu dziedziczenia: pod koniec narodziny wynoszą 0,30 z dziedziczeniem i 0,28 bez niego na 1000 kroków, a większość narodzin z dziedziczeniem pochodzi z jednego ziarna (1000).

## Następne kroki
1. Silniejszy dobór: odrodzeni agenci nie powinni kopiować wag od losowego żyjącego członka plemienia. Sprawdzić też, co wydarzyło się na ziarnie 1000.
2. Koszt energii za ruch i strojenie. Dobry znak to rosnące narodziny przy spadających odrodzeniach.
3. Sprawdzenie, czy agenci sami przejmują pola (`owned_tiles` w CSV). Jeśli nie, rozważyć małą premię energii za przejęcie.
