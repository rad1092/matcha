#!/usr/bin/env bash
# Builds the SameBoy reference runner used by the corpus analysis.
#
#   analysis/reference/build.sh <sameboy-checkout> [rgbds-dir/]
#
# Needs a C compiler and RGBDS (for SameBoy's boot ROMs). Produces
# target/reference/sameboy_profile and target/reference/dmg_boot.bin.
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
sb="$(cd "${1:?usage: build.sh <sameboy-checkout> [rgbds-dir/]}" && pwd)"
rgbds="${2:-}"

make -C "$sb" -j2 lib "$sb/build/bin/BootROMs/dmg_boot.bin" CONF=release RGBDS="$rgbds"
mkdir -p "$repo/target/reference"
cc -O2 -std=gnu11 -I"$sb" -o "$repo/target/reference/sameboy_profile" \
  "$repo/analysis/reference/sameboy_profile.c" "$sb/build/lib/libsameboy.a" -lm
cp "$sb/build/bin/BootROMs/dmg_boot.bin" "$repo/target/reference/"
echo "target/reference/sameboy_profile (SameBoy $(git -C "$sb" describe --tags --always 2>/dev/null || echo '?'))"
