# Corpus analysis

Reproduces [`docs/analysis.md`](../docs/analysis.md): every DMG-runnable ROM
in the [Homebrew Hub](https://hh.gbdev.io) database, played for 60 emulated
seconds by matcha and by SameBoy with the same scripted input.

| Step | Command | Output |
|---|---|---|
| 1. Get the database (ROMs are not redistributed here) | `git clone --filter=blob:none --sparse https://github.com/gbdev/database`<br>`git -C database sparse-checkout set --no-cone '/entries/*/game.json' '/entries/*/*.gb' '/entries/*/*.gbc'` | `database/` |
| 2. Manifest: header, mapper, toolchain fingerprint | `python3 analysis/build_manifest.py database > analysis/data/corpus.csv` | `corpus.csv` |
| 3. Profile in matcha (~10 min on 2 cores) | `analysis/run_profiles.sh database` | `profiles.json` |
| 4. Build the SameBoy reference runner (needs RGBDS) | `analysis/reference/build.sh <sameboy> [<rgbds>/]` | `target/reference/` |
| 5. Reference runs (~25 min each) | `python3 analysis/reference/run.py database zero`<br>`python3 analysis/reference/run.py database random` | `reference.jsonl`, `reference_random.jsonl` |
| 6. Speed sample (idle machine) | `python3 analysis/bench.py database` | `bench.csv` |
| 7. Report | `python3 analysis/report.py` | `summary.json`, `roms.csv`, `docs/img/*.svg` |

`opcode_names.mjs` regenerates `data/opcodes.json` (mnemonics) from the
current `web/matcha.wasm`.

## The scripted input ("monkey")

Identical in both emulators (`InputDriver` in
`crates/matcha-cli/src/profile.rs`, mirrored in
`reference/sameboy_profile.c`): for the first 10 seconds, tap Start at
frames 60–65 and A at 90–95 of every 120; after that, every 8 frames pick
at most one direction plus random A/B from an xorshift generator seeded with
the ROM's FNV-1a hash, and Start with probability 1/32. Select is never
pressed.

## Data dictionary

`corpus.csv` — one row per Homebrew Hub entry with a ROM:
`slug`, `title`, `developer`, `platform` (database field), `typetag`
(game/demo/tool/music), `year` (normalised from ISO, US and Unix dates),
`license`, `event_tags` (competitions), `rom_path`, `rom_bytes`,
`header_title`, `cgb_flag`/`cgb_mode` (0x143), `sgb`, `cart_type`,
`cart_type_name`, `mapper`, `mapper_supported`, `rom_size`, `ram_size`,
`battery`, `header_checksum_ok`, `logo_ok` (the DMG boot ROM's logo check),
`toolchain` (GB Studio 3+ / GBDK-2020 / other — byte signatures, see
`build_manifest.py`), `dmg_runnable` (not CGB-only and a supported mapper).

`profiles.json` — one record per runnable ROM from `matcha profile`:
cycle counts by CPU state (`busy`, `halted`, `stopped`, `locked`,
`interrupt`), `instructions`, per-opcode counts (`opcodes`, `cb_opcodes`),
`interrupts` taken by source, memory `reads`/`writes` by region,
`covered_rom_bytes` (distinct instruction start addresses), `locked_opcode`,
and screen metrics: `lcd_on_frames`, `nonblank_frames` (not one uniform
colour), `distinct_frames_sampled` (distinct pictures among every 15th
frame), `static_screens` (FNV-1a hashes of pictures held for 30+ frames).

`reference*.jsonl` — the same screen metrics from SameBoy, plus `locked`
(SameBoy logged an illegal opcode) and `boot_ms` (time its boot ROM took).

Outcome classes (`report.py`): **crashed** (illegal opcode), **blank**
(never a non-uniform picture), **static** (one picture the whole minute),
**running** (the picture changed).
