# HANDOFF: ewolucja

Stan na 2026-11-03. Ten plik zastępuje pamięć Claude z laptopa, której nie ma na innym komputerze. Szczegółowy stan i wyniki są w `CLAUDE.md`; to on jest źródłem prawdy.

## Cel

Neuroewolucja w symulacji plemion: 4 plemiona walczą o teren, każdy agent ma własną małą sieć neuronową, a wagi ewoluują przez dziedziczenie i mutację (bez gradientów). Teraz liczy się dobra podstawa, nie dopracowane zachowanie.

## Stack

- Rust (edycja 2024), `macroquad` (okno i grafika), `noise` (mapa), `serde`/`serde_json` (modele).
- Python + `matplotlib` tylko do narzędzi w `tools/` (wykresy, porównania).

## Uruchomienie

```powershell
git clone https://github.com/Kendrej/ewolucja.git
cd ewolucja
cargo run --release
```

- Menu: trening (nowa populacja albo zapisany model) lub demo. Spacja zmienia prędkość (x1, x10, x50, x100), Esc wraca do menu.
- Bez okna: `cargo run --release -- --headless <ziarno> <kroki> <inherit|random> [kroki_rundy]` i `--match <model_1> <model_2> <ziarno> <kroki>`.
- Porównania: `python tools/compare_inheritance.py ...`, `python tools/compare_rounds.py ...`, wykresy: `python tools/plot_run.py`.

**Uwaga:** po sklonowaniu nie ma działającego modelu. `models/main.json` jest ze starszej sieci i się nie wczyta, a foldery populacji i `runs/` są poza gitem (na laptopie też żaden zapis nie pasuje do obecnej sieci). Trzeba wytrenować nową populację i oznaczyć ją gwiazdką.

## Co zrobione

- Losowa mapa, jedzenie, pola zajmowane przez plemiona, bazy, statystyki do CSV i wykresy.
- `Brain`: sieć 19 wejść, 8 neuronów ukrytych, 3 wyjścia (187 wag). Zmysły: pole, baza, jedzenie, partner, wróg, wroga baza, oblężenie własnej bazy.
- Rozmnażanie dwóch agentów na polu plemienia, dziedziczenie z listy 20 ostatnich rodziców z mutacją, limit wieku.
- Walka: atak na najbliższego wroga, koszt energii, zabójca może trafić na listę rodziców.
- Rundy po 50 000 kroków z rankingiem. Oblężenia baz: baza bez obrońcy pada po 600 krokach, plemię bez bazy wypada do końca rundy.
- Zapis modeli, menu, nazwane populacje, model główny (gwiazdka), zwycięzca każdej rundy w `rounds/`, mecze modeli.

Kluczowy wynik: rundy wygrały z ciągłym światem 15/15 meczów. Zwycięzca rundy 20 pokonał zwycięzcę rundy 1 we wszystkich 6 meczach.

## Co dalej

1. **Otwarte pytanie:** agenci nie uczą się celowo zdobywać ani bronić baz. Bazy padają częściej niż przy losowych sieciach, ale nie coraz częściej. Trafia tylko ok. 1% ataków.
2. Cechy agentów (zaplanowane, dopiero gdy rundy działają).
3. Przed długim treningiem porównać 8, 12 i 16 neuronów ukrytych.
4. Rozważyć dopełnianie starych wag zerami, żeby dodanie wejścia nie kasowało modeli.
5. Później: widoczne ataki, lepsza grafika, opcja treningu bez grafiki na noc (szczegóły niżej).

## Ważne decyzje i ustalenia

- Baza to serce plemienia (wariant B). W bazie nie ma jedzenia. Jedzenie z zajętego pola zjada tylko właściciel, z neutralnego każdy.
- Ranking (ustalony przez użytkownika, nie otwierać ponownie): zwycięzca zajmuje 2 miejsca, 2. i 3. zostają, ostatni odpada. Wygrywa plemię, które najdłużej zostało na mapie.
- Pamięć agenta na razie niepotrzebna.
- Trening bez grafiki na noc: odłożony, zbudować dopiero na prośbę. Ustalenia: wybór „bez grafiki” przy starcie treningu, okno pokazuje tylko kroki, prędkość, narodziny i odrodzenia, zapis jak w zwykłym treningu. Przypomnieć o wyłączeniu usypiania komputera.
- Testy: użytkownik prosi „odpal sobie trening”, czyli Claude sam puszcza przebieg bez okna i krótko raportuje.

## Zasady współpracy z Claude

- Po polsku, prostym językiem, krótko: co się zmieniło, jeden wynik, jedna propozycja. Długie odpowiedzi z liczbami i listą wariantów użytkownik nazywa „slopem”.
- Tryb nadzorowany: Claude pisze kod małymi krokami, potem opisuje, co zmienił w którym pliku i jakich funkcjach, bez wklejania kodu. Użytkownik chce rozumieć projekt, nie vibecodować. Sieć tłumaczyć konkretnie.
- Commit i push tylko na prośbę, bezpośrednio na `master`, bez gałęzi, **bez** `Co-Authored-By`.
- Przy zmianach wyników aktualizować `CLAUDE.md`: krótkie punkty, mała tabela, jednozdaniowy wniosek.
