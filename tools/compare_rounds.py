"""Porownuje dlugosci rund z ciaglym swiatem, wystawiajac wytrenowane modele w meczach.

Uzycie:
    python tools/compare_rounds.py [kroki] [liczba_ziaren] [opis]

Liczby narodzin nie da sie uczciwie porownac, bo rundy zeruja mape. Dlatego kazdy wariant
trenuje osobno, a potem jego mistrzowie graja bez nauki przeciw mistrzom ciaglego swiata
na mapach, ktorych nie widzieli. Wyniki sa w runs/comparisons/<data>_<opis>/wyniki.txt.
"""
import re
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXECUTABLE = ROOT / "target" / "release" / "ewolucja.exe"
CONTINUOUS = 0
ROUND_LENGTHS = [50_000, 100_000, 200_000]
FIRST_SEED = 1000
MATCH_SEEDS = [5000, 5001, 5002]
MATCH_STEPS = 100_000
RANKING_LINE = re.compile(r"Miejsce \d+: plemie (\d+), pola (\d+), zywi (\d+)")


def run(args):
    result = subprocess.run([str(EXECUTABLE), *map(str, args)], cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"Przebieg {args} nie powiodl sie: {result.stderr}")
    return result.stdout


def train(seed, steps, round_length):
    # Rundy koncza trening w polowie rundy, zeby najliczniejsze plemie bylo prawdziwym liderem
    total = steps + round_length // 2
    output = run(["--headless", seed, total, "inherit", round_length])
    lines = output.splitlines()
    csv_path = next(ROOT / line.removeprefix("Statystyki: ") for line in lines if line.startswith("Statystyki: "))
    model = next(ROOT / line.removeprefix("Model: ") for line in lines if line.startswith("Model: "))
    return csv_path, model


def play(first, second, seed):
    # Kazdy mecz gramy w obie strony, zeby pozycja na mapie nie wplywala na wynik
    tiles = [0, 0]
    alive = [0, 0]
    for swapped in (False, True):
        pair = [second, first] if swapped else [first, second]
        for line in run(["--match", *pair, seed, MATCH_STEPS]).splitlines():
            found = RANKING_LINE.match(line)
            if found:
                side = 0 if (int(found[1]) <= 2) != swapped else 1
                tiles[side] += int(found[2])
                alive[side] += int(found[3])
    return tiles, alive


def name_of(round_length):
    return "ciagly" if round_length == CONTINUOUS else f"rundy {round_length}"


def main():
    steps = int(sys.argv[1]) if len(sys.argv) > 1 else 2_000_000
    seed_count = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    folder_name = datetime.now().strftime("%Y-%m-%d_%H-%M")
    if len(sys.argv) > 3:
        folder_name += f"_{sys.argv[3]}"
    output_dir = ROOT / "runs" / "comparisons" / folder_name

    subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)

    seeds = [FIRST_SEED + i for i in range(seed_count)]
    variants = [CONTINUOUS, *ROUND_LENGTHS]
    jobs = [(seed, round_length) for round_length in variants for seed in seeds]
    started = time.time()
    with ThreadPoolExecutor() as pool:
        trained = list(pool.map(lambda job: train(job[0], steps, job[1]), jobs))
    print(f"Trening: {len(jobs)} przebiegow w {time.time() - started:.0f} s")

    output_dir.mkdir(parents=True, exist_ok=True)
    models = {}
    for (seed, round_length), (csv_path, model) in zip(jobs, trained):
        shutil.move(csv_path, output_dir / f"{name_of(round_length).replace(' ', '_')}_seed_{seed}.csv")
        models[seed, round_length] = model

    matches = [(round_length, seed, match_seed) for round_length in ROUND_LENGTHS for seed in seeds for match_seed in MATCH_SEEDS]
    started = time.time()
    with ThreadPoolExecutor() as pool:
        results = list(pool.map(lambda m: play(models[m[1], m[0]], models[m[1], CONTINUOUS], m[2]), matches))
    print(f"Mecze: {len(matches)} w {time.time() - started:.0f} s")

    lines = [f"Mecze po {MATCH_STEPS} krokow na {len(MATCH_SEEDS)} nowych mapach, mistrzowie wariantu przeciw mistrzom ciaglego swiata:"]
    for round_length in ROUND_LENGTHS:
        rows = [(m, r) for m, r in zip(matches, results) if m[0] == round_length]
        wins = sum(1 for _, (tiles, _) in rows if tiles[0] > tiles[1])
        tiles_for = sum(tiles[0] for _, (tiles, _) in rows)
        tiles_against = sum(tiles[1] for _, (tiles, _) in rows)
        alive_for = sum(alive[0] for _, (_, alive) in rows)
        alive_against = sum(alive[1] for _, (_, alive) in rows)
        lines.append(f"  {name_of(round_length):14} wygrane {wins}/{len(rows)}, pola {tiles_for} wobec {tiles_against}, zywi {alive_for} wobec {alive_against}")
        for seed in seeds:
            seed_rows = [r for m, r in rows if m[1] == seed]
            lines.append(f"    ziarno {seed}: wygrane {sum(1 for tiles, _ in seed_rows if tiles[0] > tiles[1])}/{len(seed_rows)}")
    summary = "\n".join(lines)
    print(f"Zapisano: {output_dir}")
    print(summary)
    (output_dir / "wyniki.txt").write_text(summary + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
