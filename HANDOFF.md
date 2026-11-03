# HANDOFF: ewolucja

Stan na koniec dnia 2026-11-03 (PC). Ten plik pozwala zacząć pracę na dowolnym komputerze (laptop albo PC). Szczegółowe wyniki są w `CLAUDE.md`, który jest źródłem prawdy.

## Cel

Neuroewolucja w symulacji plemion: 4 plemiona walczą o teren, każdy agent ma własną małą sieć neuronową, a wagi ewoluują przez dziedziczenie i mutację (bez gradientów). Teraz liczy się dobra podstawa, nie dopracowane zachowanie. Nie sterujemy ewolucją ręcznie: selekcja ma iść przez plemiona i rundy, bez dodatkowych nagród za konkretne zachowania.

## Stack

- Rust (edycja 2024), `macroquad` (okno i grafika), `noise` (mapa), `serde`/`serde_json` (modele).
- Python + `matplotlib` tylko do narzędzi w `tools/`.

## Uruchomienie

```powershell
git clone https://github.com/Kendrej/evolution.git
cd evolution
cargo run --release
```

- Menu: trening (nowa populacja albo zapisany model) lub demo. Spacja zmienia prędkość, Esc wraca do menu.
- Bez okna: `cargo run --release -- --headless <ziarno> <kroki> <inherit|random> [kroki_rundy]` i `--match <model_1> <model_2> <ziarno> <kroki>` (mecz ma oblężenia baz).
- 2 mln kroków bez okna to ok. 2,5 minuty.

**Uwaga:** po sklonowaniu nie ma działającego modelu. `models/main.json` ma starą kolejność wejść i się nie wczyta. Populacje z PC (`Trening 2026-11-03`, `Trening 2027`) są poza gitem, więc na laptopie trzeba wytrenować nową i oznaczyć ją gwiazdką.

## Co zrobione dziś (2026-11-03)

- Mecze `--match` mają oblężenia baz. Z oblężeniami wynik meczu się odwraca: model, który umie zdobywać bazy, wygrał 5/6.
- W oknie widać ataki: czerwona linia to trafienie, blady okrąg pudło.
- Kolumna `conqueror_parents` w CSV: ilu zdobywców baz jest na liście rodziców.
- Dopełnianie wag zerami (`pad_weights` w `src/brain.rs`): model z mniejszą liczbą wejść wczytuje się i decyduje tak samo. **Nowe wejścia dopisywać tylko na końcu `Senses::to_inputs`.**

## Najważniejsze wyniki dnia

- Zdobywanie baz pojawia się rzadko (1–2 ziarna na 7) i zwykle zanika po 2. ćwiartce treningu.
- Liczba neuronów ukrytych (8–32) nic pewnego nie zmienia, 32 jest najsłabsze. Zostajemy przy 8.
- Zdobywcy to najwyżej 1 z 20 miejsc na liście rodziców, wypychają ich rodzice z rozmnażania.
- 10 mln kroków: bazy padają coraz rzadziej, narodziny rosną. Modele po 10 mln wygrały 27/30 meczów z modelami po 2 mln, choć zdobyły mniej baz. Długi trening uczy obrony i przetrwania.
- Odrzucony pomysł: rezerwowanie miejsc dla zdobywców na liście rodziców (to ręczne sterowanie ewolucją).

## Co dalej

1. **Następny krok:** nowe wejścia albo cechy agentów (np. zmysł „ilu moich jest w bazie”). Dopisywać na końcu listy wejść.
2. Grafika później, bo nie wpływa na uczenie.
3. Trening na noc bez grafiki: odłożony do momentu, gdy wejścia i zasady będą przemyślane i gotowe. Ustalenia: wybór „bez grafiki” przy starcie treningu, okno pokazuje tylko kroki, prędkość, narodziny i odrodzenia, zapis jak w zwykłym treningu. Przypomnieć o wyłączeniu usypiania komputera.

## Ważne decyzje i ustalenia

- Baza to serce plemienia. W bazie nie ma jedzenia. Jedzenie z zajętego pola zjada tylko właściciel, z neutralnego każdy.
- Ranking (ustalony przez użytkownika, nie otwierać ponownie): zwycięzca zajmuje 2 miejsca, 2. i 3. zostają, ostatni odpada. Wygrywa plemię, które najdłużej zostało na mapie.
- Pamięć agenta na razie niepotrzebna.
- Testy: użytkownik prosi „odpal sobie trening”, czyli Claude sam puszcza przebieg bez okna i krótko raportuje. Przy prośbie o komentarze co 10% Claude czyta CSV i pisze jedno zdanie na odcinek.

## Zasady współpracy z Claude

- Po polsku, krótko i konkretnie: wynik w 1–2 zdaniach, jedna propozycja, liczby tylko kluczowe. Długie odpowiedzi z listą wariantów to „slop”.
- Tryb nadzorowany: Claude pisze kod małymi krokami, potem opisuje, co zmienił w którym pliku i jakich funkcjach, bez wklejania kodu.
- Commit i push tylko na prośbę, bezpośrednio na `master`, bez gałęzi, **bez** `Co-Authored-By`.
- Przy zmianach wyników aktualizować `CLAUDE.md`: krótkie punkty, mała tabela, jednozdaniowy wniosek.
- **Koniec dnia:** gdy użytkownik kończy pracę, Claude aktualizuje ten plik (stan, wyniki, co dalej), commituje i pushuje, żeby dało się zacząć na laptopie albo na PC.
