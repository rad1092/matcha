#!/usr/bin/env python3
"""Builds the corpus manifest from a checkout of gbdev/database.

For every entry with a ROM, picks the default .gb/.gbc file, parses the
cartridge header, fingerprints the toolchain, and writes one CSV row.

    python3 analysis/build_manifest.py ~/src/gbdev-database > analysis/data/corpus.csv

Get the database with (blobless keeps it to the ROMs you actually read):
    git clone --filter=blob:none --sparse https://github.com/gbdev/database
    git -C database sparse-checkout set --no-cone '/entries/*/game.json' '/entries/*/*.gb' '/entries/*/*.gbc'
"""

import csv
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

CART_TYPES = {
    0x00: "ROM ONLY", 0x01: "MBC1", 0x02: "MBC1+RAM", 0x03: "MBC1+RAM+BATTERY",
    0x05: "MBC2", 0x06: "MBC2+BATTERY", 0x08: "ROM+RAM", 0x09: "ROM+RAM+BATTERY",
    0x0B: "MMM01", 0x0C: "MMM01+RAM", 0x0D: "MMM01+RAM+BATTERY",
    0x0F: "MBC3+TIMER+BATTERY", 0x10: "MBC3+TIMER+RAM+BATTERY", 0x11: "MBC3",
    0x12: "MBC3+RAM", 0x13: "MBC3+RAM+BATTERY", 0x19: "MBC5", 0x1A: "MBC5+RAM",
    0x1B: "MBC5+RAM+BATTERY", 0x1C: "MBC5+RUMBLE", 0x1D: "MBC5+RUMBLE+RAM",
    0x1E: "MBC5+RUMBLE+RAM+BATTERY", 0x20: "MBC6", 0x22: "MBC7", 0xFC: "POCKET CAMERA",
    0xFD: "BANDAI TAMA5", 0xFE: "HuC3", 0xFF: "HuC1+RAM+BATTERY",
}
# Cartridge types matcha-core implements (see crates/matcha-core/src/cartridge.rs).
SUPPORTED = {0x00, 0x01, 0x02, 0x03, 0x05, 0x06, 0x08, 0x09, 0x0F, 0x10, 0x11, 0x12,
             0x13, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E}

# SHA-256 of the 48-byte logo at 0x0104 that the DMG boot ROM compares
# against (taken from known-good cartridges; the bytes themselves are not
# reproduced here). A mismatch, or a bad header checksum, makes a real DMG
# lock up before the game starts.
LOGO_SHA256 = "daf4cabdc852baa0291849203f0b41fd0b4ecd58e0d7aff4a509f5de4d7f9a2e"

# Byte signatures, found by mining strings shared by entries tagged with a
# toolchain (see docs/analysis.md, "Toolchain fingerprints").
GB_STUDIO = b"KERNEL PANIC PLEASE"  # GB Studio 3+ crash handler text
GBDK_2020 = (b"*_*O*fo", b":G:O:fo", b"{YOzPG")  # GBDK-2020 library code


def mapper_family(t: int) -> str:
    name = CART_TYPES.get(t, "UNKNOWN")
    return name.split("+")[0] if name.startswith("MBC") else {"ROM ONLY": "none", "ROM+RAM": "none",
                                                               "ROM+RAM+BATTERY": "none"}.get(name, name)


def parse_year(raw) -> str:
    """Year of an entry's date. The database mixes ISO dates ("2023-05-20",
    "2023-05-20T20:05:00"), US dates ("12/01/2017", "01-29-2016"), bare years
    and Unix seconds ("1633053600", used for the gbcompo21 entries)."""
    s = str(raw or "").strip()
    if s.isdigit() and len(s) >= 9:
        return str(datetime.fromtimestamp(int(s), tz=timezone.utc).year)
    m = re.match(r"(\d{4})\b", s) or re.search(r"\b(\d{4})$", s)
    return m.group(1) if m else ""


def default_rom(entry: dict, folder: Path):
    files = [f for f in entry.get("files", []) if f.get("filename", "").lower().endswith((".gb", ".gbc"))]
    if not files:
        return None
    chosen = next((f for f in files if f.get("default")), files[0])
    path = folder / chosen["filename"]
    return path if path.exists() else None


def main(db_root: str) -> None:
    root = Path(db_root)
    writer = csv.writer(sys.stdout)
    writer.writerow([
        "slug", "title", "developer", "platform", "typetag", "year", "license", "event_tags",
        "rom_path", "rom_bytes", "header_title", "cgb_flag", "cgb_mode", "sgb", "cart_type",
        "cart_type_name", "mapper", "mapper_supported", "rom_size", "ram_size", "battery",
        "header_checksum_ok", "logo_ok", "toolchain", "dmg_runnable",
    ])
    for game_json in sorted(root.glob("entries/*/game.json")):
        entry = json.loads(game_json.read_text())
        rom = default_rom(entry, game_json.parent)
        if rom is None:
            continue
        data = rom.read_bytes()
        if len(data) < 0x150:
            continue
        cgb = data[0x143]
        cart = data[0x147]
        rom_size = 0x8000 << data[0x148] if data[0x148] <= 8 else len(data)
        ram_size = {1: 0x800, 2: 0x2000, 3: 0x8000, 4: 0x20000, 5: 0x10000}.get(data[0x149], 0)
        checksum = 0
        for b in data[0x134:0x14D]:
            checksum = (checksum - b - 1) & 0xFF
        title_end = 0x143 if cgb & 0x80 else 0x144
        header_title = bytes(b for b in data[0x134:title_end] if 0x20 <= b < 0x7F).decode("ascii").strip()
        if GB_STUDIO in data:
            toolchain = "GB Studio 3+"
        elif any(sig in data for sig in GBDK_2020):
            toolchain = "GBDK-2020 (C)"
        else:
            toolchain = "other / unknown"
        cgb_mode = "CGB only" if cgb == 0xC0 else "CGB enhanced" if cgb & 0x80 else "DMG"
        year = parse_year(entry.get("date"))
        writer.writerow([
            entry["slug"],
            entry.get("title", ""),
            entry.get("developer", ""),
            entry.get("platform") or "",
            entry.get("typetag") or "",
            year,
            entry.get("gameLicense") or entry.get("license") or "",
            ";".join(t for t in entry.get("tags", []) if "compo" in t.lower() or t.startswith("event:")),
            str(rom.relative_to(root)),
            len(data),
            header_title,
            f"0x{cgb:02X}",
            cgb_mode,
            int(data[0x146] == 0x03),
            f"0x{cart:02X}",
            CART_TYPES.get(cart, "UNKNOWN"),
            mapper_family(cart),
            int(cart in SUPPORTED),
            rom_size,
            ram_size,
            int("BATTERY" in CART_TYPES.get(cart, "")),
            int(checksum == data[0x14D]),
            int(hashlib.sha256(data[0x104:0x134]).hexdigest() == LOGO_SHA256),
            toolchain,
            int(cgb != 0xC0 and cart in SUPPORTED),
        ])


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
