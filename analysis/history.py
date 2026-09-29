#!/usr/bin/env python3
"""Records each profiled ROM's outcome under several matcha versions, for the
agreement-over-time table in docs/analysis.md.

    python3 analysis/history.py <commit>=<profiles.json> [<commit>=<profiles.json> ...]

Each profiles.json is run_profiles.sh's output at that commit (oldest first;
the last one should be the current analysis/data/profiles.json). Writes
analysis/data/history.csv: rom_path, then one outcome column per commit.
"""

import csv
import json
import sys

from report import DATA, outcome


def outcomes(path):
    rows = json.load(open(path))
    return {r["rom"]: outcome(r.get("locked_opcode") is not None, r["nonblank_frames"], r["distinct_frames_sampled"])
            for r in rows}


def main(args):
    runs = [arg.split("=", 1) for arg in args]
    if not runs or any(len(r) != 2 for r in runs):
        sys.exit(__doc__)
    columns = {commit: outcomes(path) for commit, path in runs}
    roms = list(next(iter(columns.values())))
    for commit, got in columns.items():
        if set(got) != set(roms):
            sys.exit(f"{commit}: profiles cover different ROMs")
    with open(DATA / "history.csv", "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["rom_path", *columns])
        for rom in roms:
            w.writerow([rom, *(columns[c][rom] for c in columns)])
    print(f"analysis/data/history.csv: {len(roms)} ROMs x {len(columns)} versions")


if __name__ == "__main__":
    main(sys.argv[1:])
