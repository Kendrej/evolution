"""Porownuje przebiegi z dziedziczeniem wag i bez niego na tych samych ziarnach.

Uzycie:
    python tools/compare_inheritance.py [kroki] [liczba_ziaren] [opis]

Kazde porownanie ma folder runs/comparisons/<data>_<opis>/ z plikami CSV, wykresem
i tabela z ostatniej czesci przebiegow, ktora pojawia sie tez w konsoli.
"""
import csv
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parent.parent
EXECUTABLE = ROOT / "target" / "release" / "ewolucja.exe"
MODES = ["inherit", "random"]
MODE_NAMES = {"inherit": "Z dziedziczeniem", "random": "Bez dziedziczenia"}
MODE_COLORS = {"inherit": "#2a9d4a", "random": "#555555"}
METRICS = [
    ("births", "Narodziny"),
    ("respawns", "Odrodzenia"),
    ("deaths", "Śmierci"),
    ("alive", "Żywi agenci"),
    ("food_eaten", "Zjedzone jedzenie"),
    ("owned_tiles", "Zajęte pola"),
    ("in_base", "Agenci we własnej bazie"),
    ("ready_in_base", "Gotowi do rozmnażania w bazie"),
    ("ready", "Wszyscy gotowi do rozmnażania"),
]
FIRST_SEED = 1000
SMOOTHING_WINDOW = 20
FINAL_PART = 0.25


def run_simulation(seed, steps, mode):
    result = subprocess.run(
        [str(EXECUTABLE), "--headless", str(seed), str(steps), mode],
        cwd=ROOT, capture_output=True, text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"Przebieg {seed} {mode} nie powiodl sie: {result.stderr}")
    for line in result.stdout.splitlines():
        if line.startswith("Statystyki: "):
            return ROOT / line.removeprefix("Statystyki: ")
    raise RuntimeError(f"Przebieg {seed} {mode} nie zapisal statystyk")


def read_totals(path):
    totals = {}
    with open(path, newline="") as file:
        for row in csv.DictReader(file):
            step = totals.setdefault(int(row["step"]), {name: 0.0 for name, _ in METRICS})
            for name, _ in METRICS:
                step[name] += float(row[name])
    return totals


def average_runs(paths):
    runs = [read_totals(path) for path in paths]
    steps = sorted(set.intersection(*(set(run) for run in runs)))
    return steps, {
        name: [sum(run[step][name] for run in runs) / len(runs) for step in steps]
        for name, _ in METRICS
    }


def smooth(values):
    result = []
    for i in range(len(values)):
        window = values[max(0, i - SMOOTHING_WINDOW + 1):i + 1]
        result.append(sum(window) / len(window))
    return result


def main():
    steps_count = int(sys.argv[1]) if len(sys.argv) > 1 else 200000
    seed_count = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    seeds = [FIRST_SEED + i for i in range(seed_count)]
    folder_name = datetime.now().strftime("%Y-%m-%d_%H-%M")
    if len(sys.argv) > 3:
        folder_name += f"_{sys.argv[3]}"
    output_dir = ROOT / "runs" / "comparisons" / folder_name

    subprocess.run(["cargo", "build", "--release"], cwd=ROOT, check=True)

    jobs = [(seed, steps_count, mode) for seed in seeds for mode in MODES]
    started = time.time()
    with ThreadPoolExecutor() as pool:
        paths = list(pool.map(lambda job: run_simulation(*job), jobs))
    print(f"{len(jobs)} przebiegow w {time.time() - started:.0f} s")

    output_dir.mkdir(parents=True, exist_ok=True)
    paths = [Path(shutil.move(path, output_dir / path.name)) for path in paths]

    results = {}
    for mode in MODES:
        mode_paths = [path for job, path in zip(jobs, paths) if job[2] == mode]
        results[mode] = average_runs(mode_paths)

    figure, axes = plt.subplots(5, 2, figsize=(14, 17), sharex=True)
    for axis, (name, title) in zip(axes.flatten(), METRICS):
        for mode in MODES:
            steps, values = results[mode]
            axis.plot(steps, smooth(values[name]), color=MODE_COLORS[mode], label=MODE_NAMES[mode])
        axis.set_title(title)
        axis.grid(alpha=0.25)
    axes[0][0].legend()
    for axis in axes[-1]:
        axis.set_xlabel("Krok symulacji")
    figure.suptitle(f"Średnia z {seed_count} ziaren, suma plemion na 1000 kroków, wygładzone po {SMOOTHING_WINDOW} pomiarach", fontsize=11)
    figure.tight_layout()
    figure.savefig(output_dir / "wykres.png", dpi=110)
    print(f"Zapisano: {output_dir}")

    lines = [f"Srednia z ostatnich {FINAL_PART:.0%} krokow:"]
    for name, _ in METRICS:
        cells = []
        for mode in MODES:
            values = results[mode][1][name]
            final = values[int(len(values) * (1 - FINAL_PART)):]
            cells.append(f"{mode} {sum(final) / len(final):8.2f}")
        lines.append(f"  {name:12} {'  '.join(cells)}")
    summary = "\n".join(lines)
    print(summary)
    (output_dir / "wyniki.txt").write_text(summary + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
