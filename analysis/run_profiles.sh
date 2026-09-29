#!/usr/bin/env bash
# Profiles every DMG-runnable ROM in the corpus: 60 emulated seconds each with
# the deterministic "monkey" input, execution profiler on.
#
#   analysis/run_profiles.sh <gbdev-database-checkout>
#
# Reads analysis/data/corpus.csv (from build_manifest.py) and writes
# analysis/data/profiles.json. About 10 minutes on two cores.
set -euo pipefail
repo="$(cd "$(dirname "$0")/.." && pwd)"
db="${1:?usage: analysis/run_profiles.sh <gbdev-database-checkout>}"

cargo build --release -p matcha-cli --manifest-path "$repo/Cargo.toml"
python3 - "$repo/analysis/data/corpus.csv" > "$repo/target/profile-roms.txt" <<'EOF'
import csv, sys
for row in csv.DictReader(open(sys.argv[1])):
    if row["dmg_runnable"] == "1":
        print(row["rom_path"])
EOF
roms=()
while IFS= read -r rom; do roms+=("$rom"); done < "$repo/target/profile-roms.txt"
# Run from the database root so the JSON records relative paths.
cd "$db"
"$repo/target/release/matcha" profile --seconds 60 --input monkey \
  --json "$repo/analysis/data/profiles.json" "${roms[@]}"
