#!/usr/bin/env bash
# Builds crates/matcha-wasm for wasm32 and copies it to web/matcha.wasm
# (used by the web player, the MCP server and the plugin).
#
#   rustup target add wasm32-unknown-unknown   # once
#   scripts/build-wasm.sh
#
# Offline toolchains without the prebuilt wasm32 std can set
# MATCHA_BUILD_STD=1 to compile std from rust-src instead.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

target=wasm32-unknown-unknown
if [ "${MATCHA_BUILD_STD:-0}" = "1" ]; then
  RUSTC_BOOTSTRAP=1 cargo build --release -p matcha-wasm --target "$target" -Zbuild-std=std,panic_abort
elif rustup target list --installed 2>/dev/null | grep -qx "$target"; then
  cargo build --release -p matcha-wasm --target "$target"
else
  echo "The $target target is not installed. Run: rustup target add $target" >&2
  exit 1
fi

cp "target/$target/release/matcha_wasm.wasm" web/matcha.wasm
echo "web/matcha.wasm $(du -h web/matcha.wasm | cut -f1)"
