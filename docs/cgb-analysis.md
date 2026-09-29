# DMG and Game Boy Color corpus revalidation

All **1,235 supported-mapper ROMs** from the pinned 1,266-entry corpus were run for 60 emulated seconds with identical deterministic input in matcha and SameBoy. The cartridge header selects DMG or native CGB. 31 unsupported-mapper entries are explicitly excluded.

**1,229/1,235 coarse outcomes agree.** This only distinguishes a changing picture, a static picture, blank output, a locked CPU, and boot failure. It does not establish correct pixels, sound, gameplay, or complete game compatibility.

| Cartridge group | Tested | Outcome agreement | matcha running / static / blank / crashed |
|---|---:|---:|---|
| DMG | 562 | 561/562 | 544 / 16 / 0 / 2 |
| CGB enhanced | 306 | 301/306 | 290 / 9 / 5 / 2 |
| CGB only | 367 | 367/367 | 362 / 2 / 0 / 3 |

## Method and limits

- Database: [gbdev/database at `5029355`](https://github.com/gbdev/database/tree/50293559a496a3e20382fbf6a2e84b70ec622f88). Every default ROM is included if its mapper is implemented; nested ROM paths and bytes against the pinned Git tree are checked. SHA-256 per ROM is in `analysis/data/cgb-roms.csv`.
- Reference: [SameBoy `213a12c`](https://github.com/LIJI32/SameBoy/tree/213a12ce93d66b105a113debd9396306066a7cfc), DMG-B or CGB-C, its open-source boot ROMs, system-RAM randomness disabled, colour correction disabled. matcha starts at post-boot state. Palette and fresh cartridge RAM defaults differ (notably cartridge RAM 00 in matcha versus FF in SameBoy); these startup-input differences can affect outcomes.
- Inputs: the existing FNV-seeded monkey schedule. Distinct frames are sampled every 15 frames; at most two non-uniform frames count as blank. CGB hashes include all RGBA channels; DMG retains the historical shade-index hash.
- Elapsed time uses the base 4,194,304 Hz clock, so double-speed CPU execution does not halve the test duration. Runner errors, missing records, duplicate keys, model mismatches, and changed ROM inputs abort publication.
- Cartridge logo/header checks are recorded in the per-ROM CSV but do not filter supported-mapper inputs. Invalid images can therefore appear as crashes or blank output; neither is automatically an emulator defect.
- The historical DMG study is retained in `docs/analysis.md`. Its comparison with this automatic-model run changes both the emulator and, for enhanced cartridges, the console model; those changes cannot be attributed solely to a timing fix.
- The `ram: zero` configuration means system-RAM initialization; it does not assert identical palette RAM, cartridge RAM or boot-written state. Follow-up probes below isolate these differences without changing the main results.
- Raw compact results and executable/boot-ROM hashes: `analysis/data/cgb-matcha.json` and `analysis/data/cgb-reference.json`. Reproduce with `analysis/cgb_study.py`; see `analysis/README.md`.

## Disagreements to investigate

| ROM | Model | matcha | SameBoy |
|---|---|---|---|
| bomber | cgb | blank | running |
| font-demo-waugh | cgb | running | static |
| gb-pda | cgb | running | blank |
| headache-boy | cgb | blank | running |
| hybrid-gbplot | dmg | static | running |
| vip | cgb | crashed | blank |

## Follow-up diagnosis

These controlled probes explain some disagreements; they do not alter the original results or turn an agreement into a compatibility claim. Measurements and intervention details are recorded in `analysis/data/cgb-triage.json`.

- **bomber — Uninitialized OBJ palette.** SameBoy with randomness disabled starts OBJ palette RAM at 00; matcha uses FF. Changing only those 64 bytes to 00 changes non-uniform output from 0 to 292 of 600 frames; final shade-index pixel counts are unchanged. Hardware OBJ palette colors are uninitialized, so neither deterministic choice is a hardware reference.
- **font-demo-waugh — Boot handoff P1 value.** The ROM waits for joypad IF before selecting a row. SameBoy hands off P1=FF and matcha P1=CF. Changing only reference P1 to CF gives 12 distinct sampled frames instead of 1 over 12 seconds and reproduces the first five static hashes observed in matcha.
- **gb-pda — Fresh cartridge RAM value.** matcha initializes fresh cartridge RAM to 00; SameBoy uses FF. The ROM reads a saved palette color from cartridge configuration and complements it. Setting only reference cartridge RAM to 00 gives a nonblank UI, 3 sampled pictures and the exact matcha static hash a2ac73363d56bba4. A two-second snapshot finds both 8 KiB VRAM banks byte-identical; palette contents differ under default startup inputs.
- **headache-boy — Uninitialized OBJ palette.** The same controlled OBJ-palette change gives 595 non-uniform frames instead of 0 in 600 frames, with unchanged final shade-index pixel counts.
- **hybrid-gbplot — Historical boot-phase difference.** The earlier DMG investigation traces its branch to the STAT mode read just after boot; the current run retains the same mismatch. This cause is inherited from the historical study, not newly established by this revalidation.
- **vip — Invalid cartridge; cause not isolated.** Both logo and header checksum are invalid. matcha reaches opcode FD at PC006B after 89 instructions. The distinct SameBoy blank outcome is recorded; its precise cause is unresolved.

CGB OBJ colors are uninitialized after boot according to [Pan Docs](https://gbdev.io/pandocs/Palettes.html). Reference RAM randomization follows the pinned [SameBoy initialization](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/gb.c). The original DMG diagnosis is preserved in `docs/analysis.md`.
