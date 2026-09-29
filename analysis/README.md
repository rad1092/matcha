# Corpus analysis

Reproduces [`docs/analysis.md`](../docs/analysis.md): every DMG-runnable ROM
in the [Homebrew Hub](https://hh.gbdev.io) database, played for 60 emulated
seconds by matcha and by SameBoy with the same scripted input.

| Step | Command | Output |
|---|---|---|
| 1. Get the database (ROMs are not redistributed here) | `git clone --filter=blob:none --sparse https://github.com/gbdev/database`<br>`git -C database sparse-checkout set --no-cone '/entries/*/game.json' '/entries/**/*.gb' '/entries/**/*.gbc' '/entries/**/*.GB' '/entries/**/*.GBC'` | `database/` |
| 2. Manifest: header, mapper, toolchain fingerprint | `python3 analysis/build_manifest.py database > analysis/data/corpus.csv` | `corpus.csv` |
| 3. Profile in matcha (~10 min on 2 cores) | `analysis/run_profiles.sh database` | `profiles.json` |
| 4. Build the SameBoy reference runner (needs RGBDS) | `analysis/reference/build.sh <sameboy> [<rgbds>/]` | `target/reference/` |
| 5. Reference runs (~25 min each) | `python3 analysis/reference/run.py database zero`<br>`python3 analysis/reference/run.py database random` | `reference.jsonl`, `reference_random.jsonl` |
| 6. Speed sample (idle machine) | `python3 analysis/bench.py database` | `bench.csv` |
| 7. Why programs sit in STOP (replays each with `matcha trace`) | `python3 analysis/stop_key1.py database` | `stop_key1.json` |
| 8. Outcomes under earlier matcha versions (profiles from `run_profiles.sh` at each commit) | `python3 analysis/history.py 22c3fcc=old0.json 9e9b8be=old1.json 2f864c7=analysis/data/profiles.json` | `history.csv` |
| 9. Numbers and charts | `python3 analysis/report.py` | `summary.json`, `roms.csv`, `docs/img/*.svg` |
| 10. Report | `python3 analysis/publish.py` | `docs/analysis.md`, `dist/analysis.html` |

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
`build_manifest.py`), `tool_tag` (the tool the authors tagged, if any — the
ground truth for the signatures), `dmg_runnable` (not CGB-only and a
supported mapper).

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

`history.csv` — each ROM's outcome under matcha as of commits `22c3fcc`
(before the comparison), `9e9b8be` (boot logo, STOP) and `2f864c7` (OAM DMA
bus conflicts, the historical pre-FIFO core).

`stop_key1.json` — programs that spend over half the minute in STOP, and how
many wrote KEY1 within the 12 instructions before stopping.

Outcome classes (`report.py`): **crashed** (illegal opcode), **blank**
(at most two frames that are not one uniform colour — the boot logo can
linger for a frame), **static** (one picture the whole minute), **running**
(the picture changed).

## Native CGB revalidation

The historical study above remains a record of its original DMG-only run.
[`docs/cgb-analysis.md`](../docs/cgb-analysis.md) revalidates all **1,235
supported-mapper default ROMs**, including 367 CGB-only and 306 CGB-enhanced
cartridges. It compares coarse outcomes, not pixel/gameplay compatibility.
Thirty-one unsupported-mapper entries remain explicitly excluded.

Pin the database before building the manifest or running the study:

```sh
git -C database checkout 50293559a496a3e20382fbf6a2e84b70ec622f88
git -C database sparse-checkout set --no-cone '/entries/*/game.json' '/entries/**/*.gb' '/entries/**/*.gbc' '/entries/**/*.GB' '/entries/**/*.GBC'
python3 analysis/build_manifest.py database > analysis/data/corpus.csv
```

Build SameBoy at `213a12ce93d66b105a113debd9396306066a7cfc` with RGBDS
(verified with v1.0.4), and build matcha before profiling. Do not replace
executables during the run.

```sh
analysis/reference/build.sh /path/to/SameBoy /path/to/rgbds/
cargo build --release -p matcha-cli
python3 analysis/cgb_study.py database --phase reference --jobs 4
python3 analysis/cgb_study.py database --phase matcha
python3 analysis/cgb_study.py database --phase report
```

The study checks the database commit, every ROM's presence/size/header and bytes against its pinned Git tree,
unique paths, exact runner coverage, selected model and elapsed time.
ROM SHA-256 values identify the actual input bytes. Both emulators use
60 seconds of deterministic monkey input with system-RAM randomness disabled.
Fresh cartridge RAM differs (matcha 00, SameBoy FF), as do uninitialized
OBJ palettes and boot-written state; `ram: zero` is not a claim that all
initial machine bytes are identical. SameBoy
runs its open-source DMG-B/CGB-C boot ROM; matcha starts after boot, so
boot-state differences remain a confounder. CGB uses raw RGBA8 frame hashes
with colour correction disabled; DMG retains shade-index hashes.

Outputs:

- `data/cgb-matcha.json`, `data/cgb-reference.json`: compact per-ROM results,
  model, clock/sample information, executable hashes and pinned inputs.
- `data/cgb-summary.json`, `data/cgb-roms.csv`: group totals, all disagreements,
  per-ROM SHA-256 and outcomes. Historical outcome transitions mix emulator
  changes with DMG-to-CGB model changes and are not a causal estimate.
- `docs/cgb-analysis.md`, `dist/cgb-analysis.html`: generated report. Rendering
  or sound errors can exist even when both outcomes are “running”.

`data/cgb-triage.json` records controlled startup-state probes for each
outcome mismatch. `reference/palette_boot_probe.rs` reproduces the OBJ
palette intervention for Bomber and Headache Boy without changing the core.
The other probes change only the stated handoff byte or fresh SRAM fill in
a scratch reference runner; primary corpus records always retain defaults.
