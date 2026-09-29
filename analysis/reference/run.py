#!/usr/bin/env python3
"""Runs the SameBoy reference runner over the same ROMs as run_profiles.sh.

    analysis/reference/build.sh <sameboy-checkout> [rgbds-dir/]   # once
    python3 analysis/reference/run.py <gbdev-database-checkout> zero|random

"zero" starts RAM at 0 like matcha and writes analysis/data/reference.jsonl;
"random" uses SameBoy's DMG power-on noise (fixed seed) and writes
analysis/data/reference_random.jsonl. One JSON object per ROM, corpus order.
Uses every core; about 25 minutes per run on two.
"""

import csv
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
EXE = REPO / "target/reference/sameboy_profile"
BOOT = REPO / "target/reference/dmg_boot.bin"
SECONDS = "60"


def main(db: Path, ram: str) -> None:
    if not EXE.exists():
        sys.exit("build the runner first: analysis/reference/build.sh <sameboy-checkout>")
    with open(REPO / "analysis/data/corpus.csv", newline="") as f:
        roms = [row["rom_path"] for row in csv.DictReader(f) if row["dmg_runnable"] == "1"]
    chunks = [roms[i:i + 10] for i in range(0, len(roms), 10)]

    def run(chunk):
        out = subprocess.run([str(EXE), str(BOOT), SECONDS, ram, *chunk], cwd=db,
                             capture_output=True, text=True, check=True).stdout
        return [json.loads(line) for line in out.splitlines()]

    records = {}
    with ThreadPoolExecutor(os.cpu_count() or 2) as pool:
        for done, batch in enumerate(pool.map(run, chunks), 1):
            records.update((r["rom"], r) for r in batch)
            print(f"\r{min(done * 10, len(roms))}/{len(roms)}", end="", file=sys.stderr)
    print(file=sys.stderr)
    missing = [r for r in roms if r not in records]
    if missing:
        sys.exit(f"no result for {len(missing)} ROMs, e.g. {missing[0]}")
    name = "reference.jsonl" if ram == "zero" else "reference_random.jsonl"
    with open(REPO / "analysis/data" / name, "w") as f:
        for rom in roms:
            f.write(json.dumps(records[rom], separators=(",", ":")) + "\n")


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[2] not in ("zero", "random"):
        sys.exit(__doc__)
    main(Path(sys.argv[1]), sys.argv[2])
