#!/usr/bin/env bash
# Assembles plugin/ (server + core + sample ROMs), validates it, and zips
# dist/matcha.plugin. Run scripts/build-wasm.sh first if the core changed.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

[ -f web/matcha.wasm ] || { echo "missing web/matcha.wasm: run scripts/build-wasm.sh" >&2; exit 1; }

rm -rf plugin/server
mkdir -p plugin/server/roms
cp mcp/server.mjs web/matcha.js web/matcha.wasm plugin/server/
cp web/roms/*.gb web/roms/LICENSES.md plugin/server/roms/

# Smoke-test the packaged server exactly as the plugin will launch it.
node plugin/server/server.mjs --self-test plugin/server/roms/libbet.gb > /dev/null

if command -v claude > /dev/null 2>&1; then
  claude plugin validate plugin/.claude-plugin/plugin.json
else
  node -e 'const p=require("./plugin/.claude-plugin/plugin.json"); if(!/^[a-z0-9-]+$/.test(p.name)) throw new Error("plugin name must be kebab-case")'
  for d in plugin/skills/*/; do [ -f "$d/SKILL.md" ] || { echo "missing $d/SKILL.md" >&2; exit 1; }; done
  echo "structure ok (claude CLI not found; ran the manual checks)"
fi

mkdir -p dist
rm -f dist/matcha.plugin
tmp="$(mktemp -d)/matcha.plugin"
(cd plugin && zip -qr "$tmp" . -x "*.DS_Store")
mv "$tmp" dist/matcha.plugin
echo "dist/matcha.plugin $(du -h dist/matcha.plugin | cut -f1)"
