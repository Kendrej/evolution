"""Rysuje wykresy z pliku CSV przebiegu i zapisuje je jako PNG obok niego.

Uzycie:
    python tools/plot_run.py                  najnowszy przebieg z runs/
    python tools/plot_run.py runs/plik.csv    wskazany przebieg
"""
import csv
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

RUNS_DIR = Path(__file__).resolve().parent.parent / "runs"
REQUIRED_COLUMN = "respawns"
SMOOTHING_WINDOW = 10
TRIBE_COLORS = ["#eb4b46", "#46c8e1", "#f0c841", "#cd5fc8"]

# Kolumna, tytul, czy wygladzac
TRIBE_PANELS = [
    ("alive", "Żywi agenci", False),
    ("owned_tiles", "Zajęte pola", False),
    ("births", "Narodziny", True),
    ("respawns", "Odrodzenia", True),
    ("deaths", "Śmierci", True),
    ("avg_lifetime", "Średnia długość życia (kroki)", True),
    ("food_eaten", "Zjedzone jedzenie", True),
]


def read_rows(path):
    with open(path, newline="") as file:
        reader = csv.DictReader(file)
        rows = list(reader)
        return reader.fieldnames or [], rows


def find_latest_run():
    runs = sorted(RUNS_DIR.glob("*.csv"), key=lambda p: p.stat().st_mtime, reverse=True)
    for path in runs:
        columns, rows = read_rows(path)
        if REQUIRED_COLUMN not in columns:
            print(f"Pomijam stary format: {path.name}")
        elif not rows:
            print(f"Pomijam plik bez pomiarów: {path.name}")
        else:
            return path
    return None


def smooth(values):
    result = []
    for i in range(len(values)):
        window = [v for v in values[max(0, i - SMOOTHING_WINDOW + 1):i + 1] if v is not None]
        result.append(sum(window) / len(window) if window else None)
    return result


def tribe_series(rows, tribe, column):
    values = []
    for row in rows:
        if int(row["tribe"]) != tribe:
            continue
        value = float(row[column])
        # Sredni wiek 0 oznacza, ze w tym odcinku nikt nie umarl
        values.append(None if column == "avg_lifetime" and row["deaths"] == "0" else value)
    return values


def total_series(rows, column):
    totals = {}
    for row in rows:
        step = int(row["step"])
        totals[step] = totals.get(step, 0.0) + float(row[column])
    return [totals[step] for step in sorted(totals)]


def plot(path):
    columns, rows = read_rows(path)
    if REQUIRED_COLUMN not in columns:
        sys.exit(f"Plik ma stary format, bez kolumny {REQUIRED_COLUMN}: {path}")
    if not rows:
        sys.exit(f"Plik nie ma jeszcze pomiarów: {path}")

    steps = sorted({int(row["step"]) for row in rows})
    tribes = sorted({int(row["tribe"]) for row in rows})

    figure, axes = plt.subplots(4, 2, figsize=(14, 14), sharex=True)
    axes = axes.flatten()

    births = smooth(total_series(rows, "births"))
    respawns = smooth(total_series(rows, "respawns"))
    axes[0].plot(steps, births, color="#2a9d4a", label="Narodziny")
    axes[0].plot(steps, respawns, color="#555555", label="Odrodzenia")
    axes[0].set_title("Główna miara uczenia: wszystkie plemiona")
    axes[0].legend()

    for axis, (column, title, smoothed) in zip(axes[1:], TRIBE_PANELS):
        for tribe in tribes:
            values = tribe_series(rows, tribe, column)
            if smoothed:
                values = smooth(values)
            values = [float("nan") if v is None else v for v in values]
            axis.plot(steps[:len(values)], values, color=TRIBE_COLORS[tribe % len(TRIBE_COLORS)], label=f"Plemię {tribe + 1}")
        axis.set_title(title)

    axes[1].legend()
    for axis in axes:
        axis.grid(alpha=0.25)
    for axis in axes[-2:]:
        axis.set_xlabel("Krok symulacji")

    figure.suptitle(f"{path.name}\nWartości zdarzeń są wygładzone po {SMOOTHING_WINDOW} pomiarach", fontsize=11)
    figure.tight_layout()

    output = path.with_suffix(".png")
    figure.savefig(output, dpi=110)
    print(f"Zapisano: {output}")


def main():
    if len(sys.argv) > 1:
        path = Path(sys.argv[1])
        if not path.exists():
            sys.exit(f"Nie ma takiego pliku: {path}")
    else:
        path = find_latest_run()
        if path is None:
            sys.exit(f"Brak przebiegów do narysowania w {RUNS_DIR}")
    plot(path)


if __name__ == "__main__":
    main()
