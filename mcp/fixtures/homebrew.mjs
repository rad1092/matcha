// Self-authored, redistributable test program. No boot ROM or game assets.
// At $0150: write $42 to WRAM $C000, increment A, write $43 to $C001,
// then loop. The header deliberately omits Nintendo's logo: matcha skips boot.
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

export function homebrewRom() {
  const rom = Buffer.alloc(32 * 1024);
  rom.set([0x00, 0xc3, 0x50, 0x01], 0x100); // nop; jp $0150
  rom.write("MATCHA-DEBUG", 0x134, "ascii");
  rom.set([0x3e, 0x42, 0xea, 0x00, 0xc0, 0x3c, 0xea, 0x01, 0xc0, 0x18, 0xfe], 0x150);
  let checksum = 0;
  for (let i = 0x134; i <= 0x14c; i++) checksum = (checksum - rom[i] - 1) & 255;
  rom[0x14d] = checksum;
  return rom;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (!process.argv[2]) throw new Error("Usage: node mcp/fixtures/homebrew.mjs OUTPUT.gb");
  writeFileSync(process.argv[2], homebrewRom(), { flag: "wx" });
}
