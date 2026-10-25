#!/usr/bin/env bash
# Assembles the optional host-specific adapter in temporary storage and zips
# dist/matcha.plugin. Run scripts/build-wasm.sh first if the core changed.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

[ -f web/matcha.wasm ] || { echo "missing web/matcha.wasm: run scripts/build-wasm.sh" >&2; exit 1; }

staging="$(mktemp -d "${TMPDIR:-/tmp}/matcha-plugin-build.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
bundle="$staging/plugin"
mkdir -p "$bundle/server/roms"
cp -R plugin/.claude-plugin plugin/.mcp.json plugin/skills plugin/README.md "$bundle/"
cp mcp/AI_GUIDE.md mcp/configure.mjs mcp/package.json LICENSE-MIT LICENSE-APACHE "$bundle/"
cp mcp/server.mjs web/matcha.js web/matcha.wasm "$bundle/server/"
cp web/roms/*.gb web/roms/LICENSES.md "$bundle/server/roms/"

# Smoke-test the packaged server exactly as the plugin will launch it.
node "$bundle/server/server.mjs" --self-test "$bundle/server/roms/libbet.gb" > /dev/null

if command -v claude > /dev/null 2>&1; then
  claude plugin validate "$bundle/.claude-plugin/plugin.json"
else
  node -e 'const p=require("./plugin/.claude-plugin/plugin.json"); if(!/^[a-z0-9-]+$/.test(p.name)) throw new Error("plugin name must be kebab-case")'
  for d in "$bundle"/skills/*/; do [ -f "$d/SKILL.md" ] || { echo "missing $d/SKILL.md" >&2; exit 1; }; done
  echo "structure ok (claude CLI not found; ran the manual checks)"
fi

mkdir -p dist
(cd "$bundle" && zip -qr "$staging/matcha.plugin" . -x "*.DS_Store")
mv "$staging/matcha.plugin" dist/matcha.plugin
echo "dist/matcha.plugin $(du -h dist/matcha.plugin | cut -f1)"
