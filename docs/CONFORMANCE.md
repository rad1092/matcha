| Suite | Passed | Total | |
|---|---:|---:|---|
| blargg | 38 | 44 | CPU, timing, sound and OAM-bug tests by Shay Green |
| mooneye | 94 | 94 | Mooneye Test Suite: acceptance + emulator-only (DMG-applicable) |
| dmg-acid2 | 1 | 1 | PPU rendering torture test |
| mealybug | 1 | 24 | Mealybug Tearoom: mid-scanline PPU effects (needs a pixel FIFO) |
| **all** | **134** | **163** | |

<details><summary>Not passing</summary>

| ROM | Result |
|---|---|
| `blargg/oam_bug/oam_bug.gb` | fail: status 0x02: oam_bug 01:ok 02:02 03:ok 04:03 05:02 06:ok 07:01 08:02 Run failed tests individually for more details. Failed #2 |
| `blargg/oam_bug/rom_singles/2-causes.gb` | fail: status 0x02: 2-causes LD DE,$FE00 : INC DE Failed #2 |
| `blargg/oam_bug/rom_singles/4-scanline_timing.gb` | fail: status 0x03: 4-scanline_timing INC DE at first corruption Failed #3 |
| `blargg/oam_bug/rom_singles/5-timing_bug.gb` | fail: status 0x02: 5-timing_bug Should corrupt at beginning of first scanline Failed #2 |
| `blargg/oam_bug/rom_singles/7-timing_effect.gb` | fail: status 0x01: 7-timing_effect 00000000 Failed |
| `blargg/oam_bug/rom_singles/8-instr_effect.gb` | fail: status 0x02: 8-instr_effect 00000000 INC/DEC rp pattern is wrong Failed #2 |
| `mealybug-tearoom-tests/ppu/m3_bgp_change.gb` | fail: 6192 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_bgp_change_sprites.gb` | fail: 5262 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_en_change.gb` | fail: 2189 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_map_change.gb` | fail: 1234 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change.gb` | fail: 146 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change_variant.gb` | fail: 1334 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change.gb` | fail: 310 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change_scx.gb` | fail: 190 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_change.gb` | fail: 1422 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_win_change.gb` | fail: 1738 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple.gb` | fail: 8982 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple_wx.gb` | fail: 6020 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_map_change.gb` | fail: 1156 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_obp0_change.gb` | fail: 432 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scx_high_5_bits.gb` | fail: 6426 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scx_low_3_bits.gb` | fail: 540 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scy_change.gb` | fail: 9873 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_window_timing.gb` | fail: 1215 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_window_timing_wx_0.gb` | fail: 954 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change.gb` | fail: 229 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change_sprites.gb` | fail: 10 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_5_change.gb` | fail: 638 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_6_change.gb` | fail: 13799 pixels differ from reference |

</details>
