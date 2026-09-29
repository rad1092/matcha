#!/usr/bin/env bash
# Downloads the conformance test data into testdata/ (never committed; see
# docs/adr/0008-verification.md). Safe to re-run: finished parts are skipped.
#
#   testdata/sst/v1  SingleStepTests/sm83 JSON cases (~160 MB), pinned commit
#   testdata/roms    c-sp/gameboy-test-roms v7.0 (Blargg, Mooneye, dmg-acid2,
#                    Mealybug, ...), checksum-verified
#
# Then:
#   cargo test --release -p matcha-core --test sst
#   cargo run --release -p matcha-cli -- test testdata/roms
set -euo pipefail
cd "$(dirname "$0")/.."

SST_REPO=https://github.com/SingleStepTests/sm83
SST_COMMIT=f9c30210245dd691661db39f5ace022c465ecc2f
ROMS_URL=https://github.com/c-sp/gameboy-test-roms/releases/download/v7.0/game-boy-test-roms-v7.0.zip
ROMS_SHA256=b9a9d7a1075aa35a3d07c07c34974048672d8520dca9e07a50178f5860c3832c

sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

mkdir -p testdata

if [ -f testdata/sst/v1/ff.json ]; then
  echo "sst: present"
else
  echo "sst: fetching $SST_REPO@${SST_COMMIT:0:10} (v1 only)"
  rm -rf testdata/sst
  git init -q testdata/sst
  git -C testdata/sst remote add origin "$SST_REPO"
  git -C testdata/sst sparse-checkout set v1
  git -C testdata/sst fetch -q --depth 1 --filter=blob:none origin "$SST_COMMIT"
  git -C testdata/sst checkout -q FETCH_HEAD
fi

if [ -f testdata/roms/README.md ]; then
  echo "roms: present"
else
  echo "roms: fetching $ROMS_URL"
  rm -rf testdata/roms
  mkdir -p testdata/roms
  zip=testdata/roms/game-boy-test-roms-v7.0.zip
  curl -fsSL -o "$zip" "$ROMS_URL"
  actual=$(sha256 "$zip")
  if [ "$actual" != "$ROMS_SHA256" ]; then
    echo "roms: checksum mismatch (got $actual)" >&2
    rm -rf testdata/roms
    exit 1
  fi
  if command -v unzip >/dev/null; then unzip -q "$zip" -d testdata/roms
  else python3 -m zipfile -e "$zip" testdata/roms
  fi
  rm "$zip"
fi

echo "testdata ready"
