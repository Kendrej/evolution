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
- Ziarno jest losowe. Żeby powtórzyć przebieg, ustaw `FIXED_SEED` w `src/main.rs`.
- Pomiary zapisują się co 1000 kroków do plików CSV w `runs/`, poza repozytorium.

## Stan projektu
Zrobione: statystyki i zapis do CSV, losowa mapa, minimalna populacja plemienia, przyspieszenie, struktura `Brain` i `Action`.

`Brain` to na razie tylko miejsce na sieć: `decide` losuje skręt, nie ma wag ani zmysłów. Nie sprawdzono jeszcze na danych, czy wydzielenie `Brain` nie zmieniło zachowania agentów.

## Następne kroki
1. Zmysły agenta: energia, gotowość do rozmnażania oraz kierunek i odległość do bazy i najbliższego jedzenia, które może zjeść.
2. Wykresy z plików CSV, żeby efekt sieci był widoczny od pierwszego przebiegu.
3. Sieć neuronowa zamiast losowego skrętu, na początku z losowymi wagami i bez dziedziczenia.
4. Dziedziczenie wag z mutacją oraz zapis i wczytywanie wag.
5. Koszt energii za ruch i strojenie. Dobry znak to rosnące narodziny przy spadających odrodzeniach.
