#!/usr/bin/env python3
"""Checks why programs spend the minute in STOP: for each one, replay the
profiler's input with `matcha trace` and look for a write to KEY1 ($FF4D, the
Game Boy Color's speed-switch register) just before the CPU stopped.

    python3 analysis/stop_key1.py <gbdev-database-checkout>

Reads analysis/data/profiles.json; writes analysis/data/stop_key1.json.
"""

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
EXE = REPO / "target/release/matcha"
LOOKBACK = 12  # instructions before STOP searched for the KEY1 write


def wrote_key1_before_stop(db: Path, rom: str) -> bool:
    out = subprocess.run([str(EXE), "trace", rom, "--input", "monkey", "--last", "40"], cwd=db,
                         capture_output=True, text=True, check=True).stdout.splitlines()
    stops = [i for i, line in enumerate(out) if line.endswith("stopped")]
    return bool(stops) and any("$ff4d" in line for line in out[max(0, stops[-1] - LOOKBACK):stops[-1]])


def main(db: Path) -> None:
    profiles = json.loads((REPO / "analysis/data/profiles.json").read_text())
    stuck = [p["rom"] for p in profiles if p["stopped_cycles"] > p["total_cycles"] / 2]
    missing = [rom for rom in stuck if not wrote_key1_before_stop(db, rom)]
    result = {"stuck_in_stop": len(stuck), "wrote_key1_before_stop": len(stuck) - len(missing),
              "without_key1_write": missing}
    (REPO / "analysis/data/stop_key1.json").write_text(json.dumps(result, indent=1) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(Path(sys.argv[1]))
