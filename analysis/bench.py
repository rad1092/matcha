#!/usr/bin/env python3
"""Measures emulation speed on a random sample of the corpus.

    python3 analysis/bench.py <gbdev-database-checkout> [sample-size]

Runs `matcha run --seconds 60 --input monkey` (profiler off, audio off) one
ROM at a time, so each run has a core to itself; run it on an otherwise idle
machine. Writes analysis/data/bench.csv (rom_path, seconds, speed).
"""

import csv
import random
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
EXE = REPO / "target/release/matcha"
LINE = re.compile(r"in ([0-9.]+)s — ([0-9.]+)x realtime")


def main(db: Path, n: int) -> None:
    subprocess.run(["cargo", "build", "--release", "-p", "matcha-cli", "--manifest-path", str(REPO / "Cargo.toml")],
                   check=True)
    with open(REPO / "analysis/data/corpus.csv", newline="") as f:
        roms = [row["rom_path"] for row in csv.DictReader(f) if row["dmg_runnable"] == "1"]
    sample = sorted(random.Random(1).sample(roms, min(n, len(roms))))
    rows = []
    for i, rom in enumerate(sample, 1):
        out = subprocess.run([str(EXE), "run", rom, "--seconds", "60", "--input", "monkey"], cwd=db,
                             capture_output=True, text=True, check=True).stderr
        seconds, _ = LINE.search(out).groups()
        rows.append({"rom_path": rom, "seconds": seconds, "speed": round(60.0 / float(seconds), 2)})
        print(f"\r{i}/{len(sample)}", end="", file=sys.stderr)
    print(file=sys.stderr)
    with open(REPO / "analysis/data/bench.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["rom_path", "seconds", "speed"])
        w.writeheader()
        w.writerows(rows)


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    main(Path(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) == 3 else 150)
