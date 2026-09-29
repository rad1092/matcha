| Suite | Passed | Total | |
|---|---:|---:|---|
| blargg-cgb | 22 | 22 | Blargg CPU and timing on CGB |
| mooneye-cgb | 81 | 83 | Mooneye acceptance, emulator-only and CGB misc cases |
| cgb-acid2 | 1 | 1 | CGB rendering and priority |
| cgb-acid-hell | 1 | 1 | CGB rendering stress test |
| gambatte-cgb | 2287 | 3218 | CGB-C hardware references; Gambatte color conversion |
| same-suite-cgb | 6 | 6 | SameSuite CGB DMA, palette and interrupt tests |
| **all** | **2398** | **3331** | |

Native CGB screenshots use CGB C references where revision-specific. Common CPU/timer/interrupt tests also run as diagnostics. CGB DMG-compatibility mode is not implemented. Gambatte audio-result tests and SameSuite APU/SGB tests are outside this scoreboard.

33 incompatible cases excluded from the total (not counted as passes):

- `mooneye-test-suite/misc/boot_div-cgbABCDE.gb`: requires CGB DMG-compatibility boot state; native CGB skips that boot path
- `mooneye-test-suite/misc/boot_hwio-C.gb`: requires CGB DMG-compatibility boot state; native CGB skips that boot path
- `mooneye-test-suite/misc/boot_regs-cgb.gb`: requires CGB DMG-compatibility boot state; native CGB skips that boot path
- `gambatte/m2int_m3stat/nobg/m2int_nobg_m3stat_1_cgb04c_out3.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `gambatte/m2int_m3stat/nobg/m2int_nobg_m3stat_2_cgb04c_out0.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `gambatte/m2int_m3stat/nobg/m2int_nobg_scx7_m3stat_1_cgb04c_out3.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m2_win_en_toggle.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_bgp_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_bgp_change_sprites.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_bg_en_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_bg_en_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_bg_map_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_bg_map_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change_variant.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change_scx.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_win_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_win_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_win_map_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_lcdc_win_map_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_obp0_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_scx_high_5_bits.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_scx_high_5_bits_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_scx_low_3_bits.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_scy_change.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_scy_change2.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_window_timing.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_window_timing_wx_0.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode
- `mealybug-tearoom-tests/ppu/m3_wx_4_change_sprites.gb`: header 0x143 has no CGB flag; its CGB hardware reference requires unsupported DMG-compatibility mode

<details><summary>Not passing</summary>

| ROM | Result |
|---|---|
| `mooneye-test-suite/acceptance/timer/rapid_toggle.gb` | fail: test reported failure (registers = 0x42) |
| `mooneye-test-suite/misc/bits/unused_hwio-C.gb` | fail: test reported failure (registers = 0x42) |
| `gambatte/bgtiledata/bgtiledata_spx08_1.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_3.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_ds_3.gbc` | fail: 1152 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_ds_4.gbc` | fail: 1040 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_1.gbc` | fail: 1032 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_3.gbc` | fail: 1032 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_ds_1.gbc` | fail: 120 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_ds_3.gbc` | fail: 1032 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_ds_4.gbc` | fail: 1152 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_1.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_3.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_2.gbc` | fail: 8 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_ds_1.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_ds_4.gbc` | fail: 1024 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_ds_1.gbc` | fail: 1152 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_ds_4.gbc` | fail: 1152 pixels differ from reference |
| `gambatte/cgbpal_m3/cgbpal_m3end_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_ds_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_ds_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx2_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx2_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx3_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx3_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx5_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx5_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx5_ds_1_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx5_ds_2_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3end_scx5_ds_3_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/cgbpal_m3/cgbpal_m3start_ds_1_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/cgbpal_m3/cgbpal_read_m3start_ds_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/cgbpal_m3/cgbpal_read_m3start_lcdoffset1_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/cgbpal_m3/cgbpal_write_m3start_ds_1_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/cgbpal_m3/cgbpal_write_m3start_lcdoffset1_1_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/display_startstate/ly_dmg08_out00_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/display_startstate/stat_1_cgb04c_out87.gbc` | fail: printed 82, expected 87 |
| `gambatte/display_startstate/stat_2_cgb04c_out84.gbc` | fail: printed 82, expected 84 |
| `gambatte/display_startstate/stat_scx2_1_cgb04c_out87.gbc` | fail: printed 82, expected 87 |
| `gambatte/display_startstate/stat_scx2_2_cgb04c_out84.gbc` | fail: printed 82, expected 84 |
| `gambatte/display_startstate/stat_scx3_1_cgb04c_out87.gbc` | fail: printed 82, expected 87 |
| `gambatte/display_startstate/stat_scx3_2_cgb04c_out84.gbc` | fail: printed 82, expected 84 |
| `gambatte/display_startstate/stat_scx5_1_cgb04c_out87.gbc` | fail: printed 82, expected 87 |
| `gambatte/display_startstate/stat_scx5_2_cgb04c_out84.gbc` | fail: printed 82, expected 84 |
| `gambatte/div/start_inc_1_cgb04c_out1E.gbc` | fail: printed 00, expected 1E |
| `gambatte/div/start_inc_2_cgb04c_out1F.gbc` | fail: printed 00, expected 1F |
| `gambatte/dma/gdma_cycles_2xshort_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/gdma_cycles_2xshort_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/gdma_cycles_long_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/gdma_cycles_short_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/gdma_cycles_short_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/hdma_cycles_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/dma/hdma_disable_display_1_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/dma/hdma_late_destl_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_late_disable_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_disable_ds_1_cgb04c_out0.gbc` | fail: printed 7, expected 0 |
| `gambatte/dma/hdma_late_disable_ds_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_disable_scx2_1_cgb04c_out0.gbc` | fail: printed 7, expected 0 |
| `gambatte/dma/hdma_late_disable_scx2_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_disable_scx3_1_cgb04c_out0.gbc` | fail: printed 7, expected 0 |
| `gambatte/dma/hdma_late_disable_scx3_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_disable_scx5_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_disable_scx5_ds_1_cgb04c_out0.gbc` | fail: printed 7, expected 0 |
| `gambatte/dma/hdma_late_disable_scx5_ds_2_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_ei_m3halt_m2unhalt_ly_scx1_4_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/dma/hdma_late_enable_1_cgb04c_out1.gbc` | fail: printed 7, expected 1 |
| `gambatte/dma/hdma_late_enable_2_cgb04c_out0.gbc` | fail: printed 7, expected 0 |
| `gambatte/dma/hdma_late_enable_ds_2_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_late_enable_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_late_enable_lcdoffset3_1_cgb04c_out1.gbc` | fail: column 0: the screen does not show a hex digit |
| `gambatte/dma/hdma_late_enable_lcdoffset3_2_cgb04c_out0.gbc` | fail: column 0: the screen does not show a hex digit |
| `gambatte/dma/hdma_late_if_and_ie_halt_1_cgb04c_out00.gbc` | fail: printed 02, expected 00 |
| `gambatte/dma/hdma_late_length_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_late_m0halt_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m0halt_ds_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m0halt_ds_lcdoffset1_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m0halt_lcdoffset3_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_inc_scx1_2_cgb04c_out02.gbc` | fail: printed 01, expected 02 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_inc_scx2_2_cgb04c_out02.gbc` | fail: printed 01, expected 02 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_ly_scx1_4_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_ly_scx2_2_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_ly_scx2_4_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/dma/hdma_late_m3halt_m2unhalt_scx2_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_hdma5_scx1_2_cgb04c_out80.gbc` | fail: printed FF, expected 80 |
| `gambatte/dma/hdma_late_m3speedchange_hdma5_scx1_ds_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_hdma5_scx2_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_hdma5_scx2_2_cgb04c_out80.gbc` | fail: printed FF, expected 80 |
| `gambatte/dma/hdma_late_m3speedchange_hdma5_scx2_ds_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_inc_scx1_2_cgb04c_out02.gbc` | fail: printed 01, expected 02 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_1_cgb04c_out92.gbc` | fail: printed 0C, expected 92 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_2_cgb04c_out93.gbc` | fail: printed 0C, expected 93 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_3_cgb04c_out92.gbc` | fail: printed 0C, expected 92 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_4_cgb04c_out93.gbc` | fail: printed 0C, expected 93 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_5_cgb04c_out92.gbc` | fail: printed 0C, expected 92 |
| `gambatte/dma/hdma_late_m3speedchange_ly_scx1_6_cgb04c_out93.gbc` | fail: printed 0C, expected 93 |
| `gambatte/dma/hdma_late_m3speedchange_read_hdmadst00_scx1_1_cgb04c_out00.gbc` | fail: printed 9F, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_read_hdmadst00_scx1_ds_1_cgb04c_out00.gbc` | fail: printed 9F, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_read_hdmadst00_scx2_1_cgb04c_out00.gbc` | fail: printed 9F, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_read_hdmadst00_scx2_ds_1_cgb04c_out00.gbc` | fail: printed 9F, expected 00 |
| `gambatte/dma/hdma_late_m3speedchange_tima_scx1_ds_1_cgb04c_outF3.gbc` | fail: printed F7, expected F3 |
| `gambatte/dma/hdma_late_m3speedchange_tima_scx1_ds_2_cgb04c_outF4.gbc` | fail: printed F7, expected F4 |
| `gambatte/dma/hdma_late_m3speedchange_tima_scx1_ds_3_cgb04c_outF6.gbc` | fail: printed F8, expected F6 |
| `gambatte/dma/hdma_late_m3speedchange_tima_scx1_ds_4_cgb04c_outF7.gbc` | fail: printed F8, expected F7 |
| `gambatte/dma/hdma_late_m3speedchange_tima_scx1_ds_6_cgb04c_outF9.gbc` | fail: printed F8, expected F9 |
| `gambatte/dma/hdma_late_speedchange_inc_scx1_ds_2_cgb04c_out02.gbc` | fail: printed 01, expected 02 |
| `gambatte/dma/hdma_late_wrambank_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_m0halt_late_m3unhalt_scx1_2_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_m0speedchange_late_m3wakeup_scx1_1_cgb04c_outFF.gbc` | fail: printed 00, expected FF |
| `gambatte/dma/hdma_m0speedchange_late_m3wakeup_scx2_1_cgb04c_outFF.gbc` | fail: printed 00, expected FF |
| `gambatte/dma/hdma_m3speedchange_late_m0wakeup_1_cgb04c_outFF.gbc` | fail: printed 00, expected FF |
| `gambatte/dma/hdma_pc_7ffe_cgb04c_out02.gbc` | fail: printed 80, expected 02 |
| `gambatte/dma/hdma_start_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_ds_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_ly0_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_scx2_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_scx3_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_scx5_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_start_scx5_ds_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/dma/hdma_transition_7fffhalt_inc_m3unhalt_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/dma/hdma_transition_ei_halt_late_unhalt_ldaaimm_hdma_scx1_1_cgb04c_out00.gbc` | fail: printed 84, expected 00 |
| `gambatte/dma/hdma_transition_ei_halt_late_unhalt_ldaaimm_hdma_scx1_2_cgb04c_out02.gbc` | fail: printed 85, expected 02 |
| `gambatte/dma/hdma_transition_ei_halt_late_unhalt_scx1_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_transition_halt_late_unhalt_ldaaimm_hdma_scx1_1_cgb04c_out00.gbc` | fail: printed 01, expected 00 |
| `gambatte/dma/hdma_transition_halt_late_unhalt_ldaaimm_hdma_scx1_2_cgb04c_out02.gbc` | fail: printed FF, expected 02 |
| `gambatte/dma/hdma_transition_halt_late_unhalt_scx1_1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_transition_halt_m0unhalt_ldaaimm_scx1_cgb04c_out02.gbc` | fail: printed 01, expected 02 |
| `gambatte/dma/hdma_transition_oamdma_1_cgb04c_out509E529C.gbc` | fail: printed 50515253, expected 509E529C |
| `gambatte/dma/hdma_transition_oamdma_2_cgb04c_out67.gbc` | fail: printed E0, expected 67 |
| `gambatte/dma/hdma_transition_speedchange_hdmalen00_hdma5_scx1_cgb04c_out80.gbc` | fail: printed FF, expected 80 |
| `gambatte/dma/hdma_transition_speedchange_hdmalen01_hdma5_scx1_cgb04c_out81.gbc` | fail: printed 00, expected 81 |
| `gambatte/dma/hdma_transition_speedchange_hdmalen01_hdmadst10_scx1_cgb04c_out00.gbc` | fail: printed 01, expected 00 |
| `gambatte/dma/hdma_transition_speedchange_hdmalen7f_hdma5_scx1_cgb04c_outFF.gbc` | fail: printed 7E, expected FF |
| `gambatte/dma/hdma_transition_speedchange_hdmalen7f_hdmadst10_scx1_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/dma/hdma_transition_speedchange_ldaaimm_scx1_cgb04c_outFF.gbc` | fail: printed 31, expected FF |
| `gambatte/dma/hdma_transition_speedchange_ldaaimm_scx1_ds_cgb04c_out03.gbc` | fail: printed 31, expected 03 |
| `gambatte/dma/hdma_transition_speedchange_oamdma_cgb04c_out71.gbc` | fail: printed 6A, expected 71 |
| `gambatte/dma/late_gdma_pc_7ffe_1_cgb04c_out02.gbc` | fail: printed 00, expected 02 |
| `gambatte/enable_display/enable_display_ly0_sprites_m0stat_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/enable_display/frame0_m2stat_count_ds_1_cgb04c_out91.gbc` | fail: printed 01, expected 91 |
| `gambatte/enable_display/frame1_ly_count_ds_1_cgb04c_out99.gbc` | fail: printed 00, expected 99 |
| `gambatte/enable_display/frame1_m2stat_count_ds_1_cgb04c_out91.gbc` | fail: printed 00, expected 91 |
| `gambatte/enable_display/ly0_late_cgbpr_1_cgb04c_out55.gbc` | fail: printed FF, expected 55 |
| `gambatte/enable_display/ly0_late_cgbpr_ds_1_cgb04c_out55.gbc` | fail: printed FF, expected 55 |
| `gambatte/enable_display/ly0_late_cgbpw_1_cgb04c_outAA.gbc` | fail: printed 55, expected AA |
| `gambatte/enable_display/ly0_late_cgbpw_ds_1_cgb04c_outAA.gbc` | fail: printed 55, expected AA |
| `gambatte/enable_display/ly0_late_vramr_2_dmg08_outFF_cgb04c_out55.gbc` | fail: printed FF, expected 55 |
| `gambatte/enable_display/ly0_late_vramw_2_dmg08_out55_cgb04c_outAA.gbc` | fail: printed 55, expected AA |
| `gambatte/irq_precedence/hdma_vs_m0_scx2_cgb04c_out0183.gbc` | fail: printed 1234, expected 0183 |
| `gambatte/irq_precedence/late_hdma_vs_ei_scx1_1_cgb04c_out102E.gbc` | fail: printed 1234, expected 102E |
| `gambatte/irq_precedence/late_hdma_vs_ei_scx2_1_cgb04c_out102F.gbc` | fail: printed 1234, expected 102F |
| `gambatte/irq_precedence/late_hdma_vs_ie_scx1_1_cgb04c_out102E.gbc` | fail: printed 1234, expected 102E |
| `gambatte/irq_precedence/late_hdma_vs_ie_scx2_1_cgb04c_out102F.gbc` | fail: printed 1234, expected 102F |
| `gambatte/irq_precedence/late_hdma_vs_tima_scx1_2_cgb04c_out11E9.gbc` | fail: printed 1234, expected 11E9 |
| `gambatte/irq_precedence/late_hdma_vs_tima_scx1_halt_2_cgb04c_out11C9.gbc` | fail: printed 1234, expected 11C9 |
| `gambatte/irq_precedence/late_hdma_vs_tima_scx2_2_cgb04c_out11E9.gbc` | fail: printed 1234, expected 11E9 |
| `gambatte/irq_precedence/late_hdma_vs_tima_scx2_halt_2_cgb04c_out11C9.gbc` | fail: printed 1234, expected 11C9 |
| `gambatte/lcd_offset/offset1_lyc8fint_m1stat_1_cgb04c_outC4.gbc` | fail: printed C0, expected C4 |
| `gambatte/lcd_offset/offset1_lyc98int_ly_count_1_cgb04c_out99.gbc` | fail: printed 00, expected 99 |
| `gambatte/lcd_offset/offset1_lyc98int_ly_count_2_cgb04c_out9A.gbc` | fail: printed 99, expected 9A |
| `gambatte/lcd_offset/offset1_lyc98int_ly_count_ds_2_cgb04c_out9A.gbc` | fail: printed 02, expected 9A |
| `gambatte/lcd_offset/offset1_lyc99int_m0irq_count_scx2_ds_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset1_lyc99int_m0stat_count_scx1_ds_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset1_lyc99int_m0stat_count_scx2_ds_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset1_lyc99int_m0stat_count_scx3_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset1_lyc99int_m2irq_count_1_cgb04c_out98.gbc` | fail: printed 01, expected 98 |
| `gambatte/lcd_offset/offset1_lyc99int_m2irq_count_ds_1_cgb04c_out98.gbc` | fail: printed 00, expected 98 |
| `gambatte/lcd_offset/offset1_lyc99int_m2irq_count_ds_2_cgb04c_out91.gbc` | fail: printed 90, expected 91 |
| `gambatte/lcd_offset/offset1_lyc99int_m2stat_count_1_cgb04c_out91.gbc` | fail: printed 00, expected 91 |
| `gambatte/lcd_offset/offset1_lyc99int_m2stat_count_ds_1_cgb04c_out91.gbc` | fail: printed 00, expected 91 |
| `gambatte/lcd_offset/offset2_lyc8fint_m1stat_1_cgb04c_outC4.gbc` | fail: printed C0, expected C4 |
| `gambatte/lcd_offset/offset2_lyc98int_ly_count_1_cgb04c_out99.gbc` | fail: printed 00, expected 99 |
| `gambatte/lcd_offset/offset2_lyc98int_ly_count_2_cgb04c_out9A.gbc` | fail: printed 01, expected 9A |
| `gambatte/lcd_offset/offset2_lyc98int_ly_count_3_cgb04c_out9A.gbc` | fail: printed 99, expected 9A |
| `gambatte/lcd_offset/offset2_lyc99int_m0stat_count_scx2_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset2_lyc99int_m2irq_count_1_cgb04c_out98.gbc` | fail: printed 01, expected 98 |
| `gambatte/lcd_offset/offset3_lyc98int_ly_count_1_cgb04c_out99.gbc` | fail: printed 00, expected 99 |
| `gambatte/lcd_offset/offset3_lyc98int_ly_count_2_cgb04c_out9A.gbc` | fail: printed 00, expected 9A |
| `gambatte/lcd_offset/offset3_lyc99int_m0stat_count_scx1_1_cgb04c_out90.gbc` | fail: printed 00, expected 90 |
| `gambatte/lcd_offset/offset3_lyc99int_m2irq_count_1_cgb04c_out98.gbc` | fail: printed 00, expected 98 |
| `gambatte/lcd_offset/offset3_lyc99int_m2irq_count_2_cgb04c_out91.gbc` | fail: printed 90, expected 91 |
| `gambatte/ly0/lycint152_ly0stat_ds_2_cgb04c_outC1.gbc` | fail: printed C0, expected C1 |
| `gambatte/ly0/lycint152_ly153_ds_4_cgb04c_out99.gbc` | fail: printed 00, expected 99 |
| `gambatte/ly0/lycint152_lyc0flag_ds_3_cgb04c_outC4.gbc` | fail: printed C0, expected C4 |
| `gambatte/ly0/lycint152_lyc153flag_ds_2_cgb04c_outC5.gbc` | fail: printed C1, expected C5 |
| `gambatte/ly0/lycint152_lyc153flag_ds_3_cgb04c_outC5.gbc` | fail: printed C1, expected C5 |
| `gambatte/ly0/lycint152_lyc153irq_ds_2_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/ly0/lycint152_lyc153irq_late_retrigger_2_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_ifw_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_ifw_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_late_retrigger_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/ff41_disable_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/ff45_disable_2_dmg08_out1_cgb04c_out3.gbc` | fail: printed 1, expected 3 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_3_dmg08_out1_cgb04c_out3.gbc` | fail: printed 1, expected 3 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_ds_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_ds_lcdoffset1_3_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/ff45_enable_weirdpoint_ds_lcdoffset1_4_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/late_ff41_enable_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/late_ff41_enable_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/late_ff45_enable_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/lycEnable/late_ff45_enable_ds_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/lycEnable/late_ff45_enable_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/lcdoff_lycirqen_1_dmg08_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lyc0_ff41_disable_2_dmg08_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lyc0_ff45_disable_2_dmg08_outE0_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lyc0_late_ff45_enable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc0_m1disable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_enable_m1disable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_ff41_enable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_ff41_enable_ds_lcdoffset1_2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_ff45_enable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_ff45_enable_3_dmg08_outE0_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lyc153_late_ff45_enable_4_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_ff45_enable_ds_4_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lyc153_late_ff45_enable_ds_lcdoffset1_2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_late_m1disable_2_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc153_m1disable_ds_2_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycEnable/lyc_ff45_disable2_2_dmg08_out1_cgb04c_out3.gbc` | fail: printed 1, expected 3 |
| `gambatte/lycEnable/lyc_ff45_trigger_delay_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/lycwirq_trigger_ly00_stat50_2_dmg08_outE0_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lycwirq_trigger_ly00_stat50_ds_1_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycint_lycflag/lycint_lycflag_ds_3_cgb04c_out4.gbc` | fail: printed 0, expected 4 |
| `gambatte/m0enable/disable_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/disable_scx3_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/disable_scx4_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/disable_scx5_ds_2_cgb04c_out3.gbc` | fail: printed 1, expected 3 |
| `gambatte/m0enable/disable_scx7_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/enable_wxA6_2x_spxA7_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/late_enable_lcdoffset1_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/lycdisable_ff41_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff41_scx1_ds_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff41_scx3_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_3_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_scx1_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_scx1_ds_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_scx2_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_scx3_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0enable/lycdisable_ff45_scx3_3_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m1/ly143_late_m0enable_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/ly143_late_m0enable_ds_lcdoffset1_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/ly143_late_m2enable_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/ly143_late_m2enable_ds_lcdoffset1_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/lyc143_late_m2enable_lycdisable_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/lyc143_late_m2enable_lycdisable_ds_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_enable_after_lyc144_2_dmg08_out1_cgb04c_out3.gbc` | fail: printed 1, expected 3 |
| `gambatte/m1/m1irq_late_enable_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m1/m1irq_m0disable_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2disable_lycdisable_2_dmg08_out3_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2disable_lycdisable_3_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2disable_lycdisable_ds_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_1_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_ds_1_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m2m1irq_ifw_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m2m1irq_ifw_ds_2_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m2enable/late_enable_after_lycint_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_after_lycint_disable_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_ly0_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_ly0_ds_lcdoffset1_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_m0disable_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_m1disable_ly0_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2enable/late_enable_m1disable_ly0_ds_1_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_m1disable_ly0_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2enable/lyc0_late_m2enable_lycdisable_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2enable/lyc1_late_m2enable_lycdisable_1_dmg08_out0_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/lyc1_late_m2enable_lycdisable_ds_3_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/lyc1_m2irq_late_lyc255_2_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2enable/lyc1_m2irq_late_lyc255_ds_2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m3stat/scx/late_scx4_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/miscmstatirq/lycstatwirq_trigger_ly00_10_50_ds_1_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/miscmstatirq/lycstatwirq_trigger_m0_late_ly44_lyc44_08_40_4_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/miscmstatirq/m1statwirq_trigger_ly94_lyc94_40_50_2_dmg08_outE0_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/oam_access/midwrite_2_dmg08_out1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oam_access/postread_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oam_access/postwrite_scx1_ds_1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oam_access/preread_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oam_access/preread_lcdoffset1_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oam_access/prewrite_2_dmg08_out1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oam_access/prewrite_ds_2_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oam_access/prewrite_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/late_sp00x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp00x_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp00y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp00y_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/late_sp01x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp01x_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp01y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp01y_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/late_sp02x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp02y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39x_3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39x_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/late_sp39y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39y_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/oamdma_late_halt_stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_late_speedchange_stat_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/oamdma_src0000_busycallAFFF_dmg08_cgb04c_outFF8F.gbc` | fail: printed 0304, expected FF8F |
| `gambatte/oamdma/oamdma_src0000_busyint0002_dmg08_cgb04c_outFF941234.gbc` | fail: printed 76871234, expected FF941234 |
| `gambatte/oamdma/oamdma_src0000_busypopDFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 657600AA, expected 657655AA |
| `gambatte/oamdma/oamdma_src0000_busypopEFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 65765500, expected 657655AA |
| `gambatte/oamdma/oamdma_src0000_busypopFDFF_dmg08_out657665FF_cgb04c_out657655FF.gbc` | fail: printed 657600FF, expected 657655FF |
| `gambatte/oamdma/oamdma_src0000_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src0000_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src0000_busypushA001_2_dmg08_cgb04c_out5576AAFF.gbc` | fail: printed 6576AAFF, expected 5576AAFF |
| `gambatte/oamdma/oamdma_src0000_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src0000_busypushC001_2_dmg08_out55AAFF34_cgb04c_out65AAFF55.gbc` | fail: printed 6576FF55, expected 65AAFF55 |
| `gambatte/oamdma/oamdma_src0000_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src0000_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761255, expected 6576AA55 |
| `gambatte/oamdma/oamdma_src0000_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 6576AA34, expected 6576AA55 |
| `gambatte/oamdma/oamdma_src0000_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65761298, expected 6576AA98 |
| `gambatte/oamdma/oamdma_src0000_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_src0000_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_src0000_busyrst0002_dmg08_cgb04c_outFF8DFA9E.gbc` | fail: printed 7687FA9E, expected FF8DFA9E |
| `gambatte/oamdma/oamdma_src7F00_busypopBFFF_2_dmg08_out65766576_cgb04c_out657665AA.gbc` | fail: printed 65766500, expected 657665AA |
| `gambatte/oamdma/oamdma_src7F00_busypopBFFF_dmg08_out65766576_cgb04c_out657665AA.gbc` | fail: printed 65766500, expected 657665AA |
| `gambatte/oamdma/oamdma_src7F00_busypopDFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 65765500, expected 657655AA |
| `gambatte/oamdma/oamdma_src7F00_busypopEFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 657600AA, expected 657655AA |
| `gambatte/oamdma/oamdma_src7F00_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src7F00_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src7F00_busypushA001_2_dmg08_cgb04c_out5576AAFF.gbc` | fail: printed 6576AAFF, expected 5576AAFF |
| `gambatte/oamdma/oamdma_src7F00_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src7F00_busypushC001_2_dmg08_out55AAFF34_cgb04c_out65AAFF55.gbc` | fail: printed 6576FF34, expected 65AAFF55 |
| `gambatte/oamdma/oamdma_src7F00_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761234, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src7F00_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 6576AA34, expected 6576AA55 |
| `gambatte/oamdma/oamdma_src7F00_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761255, expected 6576AA55 |
| `gambatte/oamdma/oamdma_src7F00_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_src7F00_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_src8000_busypop7FFF_dmg08_out65765576_cgb04c_out65005576.gbc` | fail: printed 65765576, expected 65005576 |
| `gambatte/oamdma/oamdma_src8000_busypop9FFF_2_dmg08_out657665FF_cgb04c_out007665FF.gbc` | fail: printed 657665FF, expected 007665FF |
| `gambatte/oamdma/oamdma_src8000_busypop9FFF_dmg08_out657665AA_cgb04c_out007665AA.gbc` | fail: printed 657665AA, expected 007665AA |
| `gambatte/oamdma/oamdma_src8000_busypush8001_dmg08_out55761234_cgb04c_out00761234.gbc` | fail: printed 65761234, expected 00761234 |
| `gambatte/oamdma/oamdma_src8000_busypushA001_2_dmg08_out65AA12FF_cgb04c_out650012FF.gbc` | fail: printed 657612FF, expected 650012FF |
| `gambatte/oamdma/oamdma_src8000_busypushA001_dmg08_out65AA1255_cgb04c_out65001255.gbc` | fail: printed 65761255, expected 65001255 |
| `gambatte/oamdma/oamdma_src8000_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_src8000_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_src8000_busywrite8000_dmg08_cgb04c_out0.gbc` | fail: printed 4, expected 0 |
| `gambatte/oamdma/oamdma_src8000_srcchange0000_busyinc_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_src8000_vrambankchange_2_cgb04c_out4.gbc` | fail: printed 0, expected 4 |
| `gambatte/oamdma/oamdma_src8000_vrambankchange_4_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oamdma/oamdma_src9F00_busypop7FFF_dmg08_out65765576_cgb04c_out65005576.gbc` | fail: printed 65765576, expected 65005576 |
| `gambatte/oamdma/oamdma_src9F00_busypop9FFF_2_dmg08_out657665FF_cgb04c_out007665FF.gbc` | fail: printed 657665FF, expected 007665FF |
| `gambatte/oamdma/oamdma_src9F00_busypop9FFF_dmg08_out657665AA_cgb04c_out007665AA.gbc` | fail: printed 657665AA, expected 007665AA |
| `gambatte/oamdma/oamdma_src9F00_busypush8001_dmg08_out55761234_cgb04c_out00761234.gbc` | fail: printed 65761234, expected 00761234 |
| `gambatte/oamdma/oamdma_src9F00_busypushA001_2_dmg08_out65AA12FF_cgb04c_out650012FF.gbc` | fail: printed 657612FF, expected 650012FF |
| `gambatte/oamdma/oamdma_src9F00_busypushA001_dmg08_out65AA1255_cgb04c_out65001255.gbc` | fail: printed 65761255, expected 65001255 |
| `gambatte/oamdma/oamdma_src9F00_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_src9F00_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_srcA000_busypopDFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 657600AA, expected 657655AA |
| `gambatte/oamdma/oamdma_srcA000_busypopEFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 65765500, expected 657655AA |
| `gambatte/oamdma/oamdma_srcA000_busypopFDFF_dmg08_out657665FF_cgb04c_out657655FF.gbc` | fail: printed 657600FF, expected 657655FF |
| `gambatte/oamdma/oamdma_srcA000_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcA000_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcA000_busypushA001_2_dmg08_cgb04c_out55FFAAFF.gbc` | fail: printed FFFFAAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcA000_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcA000_busypushC001_2_dmg08_out55AAFF34_cgb04c_outFFAAFF55.gbc` | fail: printed FFFFFF55, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcA000_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcA000_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761255, expected 6576AA55 |
| `gambatte/oamdma/oamdma_srcA000_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 6576AA34, expected 6576AA55 |
| `gambatte/oamdma/oamdma_srcA000_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65761298, expected 6576AA98 |
| `gambatte/oamdma/oamdma_srcA000_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_srcA000_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_srcA000_busywrite4000_dmg08_cgb04c_out2.gbc` | fail: printed 4, expected 2 |
| `gambatte/oamdma/oamdma_srcBF00_busypopBFFF_2_dmg08_outFFFFFFFF_cgb04c_outFFFFFFAA.gbc` | fail: printed FFFFFF00, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcBF00_busypopBFFF_dmg08_out65766576_cgb04c_out657665AA.gbc` | fail: printed 65766500, expected 657665AA |
| `gambatte/oamdma/oamdma_srcBF00_busypopDFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 65765500, expected 657655AA |
| `gambatte/oamdma/oamdma_srcBF00_busypopEFFF_dmg08_out65766576_cgb04c_out657655AA.gbc` | fail: printed 657600AA, expected 657655AA |
| `gambatte/oamdma/oamdma_srcBF00_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcBF00_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcBF00_busypushA001_2_dmg08_cgb04c_out55FFAAFF.gbc` | fail: printed FFFFAAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcBF00_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcBF00_busypushC001_2_dmg08_out55AAFF34_cgb04c_outFFAAFF55.gbc` | fail: printed FFFFFF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcBF00_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761234, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcBF00_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 6576AA34, expected 6576AA55 |
| `gambatte/oamdma/oamdma_srcBF00_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761255, expected 6576AA55 |
| `gambatte/oamdma/oamdma_srcBF00_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_srcBF00_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_srcC000_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_srcC000_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_srcD000_wrambankchange_2_cgb04c_out4.gbc` | fail: printed 0, expected 4 |
| `gambatte/oamdma/oamdma_srcDF00_busypushFEA1_dmg08_out65768700_cgb04c_out65768734.gbc` | fail: printed 65768700, expected 65768734 |
| `gambatte/oamdma/oamdma_srcDF00_busypushFF01_dmg08_out657600DF_cgb04c_out657612DF.gbc` | fail: printed 657600DF, expected 657612DF |
| `gambatte/oamdma/oamdma_srcE000_busypop7FFF_dmg08_out657665AA_cgb04c_outFFFFFFAA.gbc` | fail: printed 657655AA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcE000_busypop9FFF_2_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcE000_busypop9FFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcE000_busypopBFFF_2_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FF76, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcE000_busypopBFFF_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 65765576, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcE000_busypopDFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcE000_busypopEFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcE000_busypopFDFF_dmg08_out657665FF_cgb04c_outFFFF55FF.gbc` | fail: printed 657665FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcE000_busypopFE9F_dmg08_out6576FFFF_cgb04c_outFFFFFFFF.gbc` | fail: printed 6576FFFF, expected FFFFFFFF |
| `gambatte/oamdma/oamdma_srcE000_busypopFEFF_dmg08_out6576FFEF_cgb04c_outFFFFFFEF.gbc` | fail: printed 6576FFEF, expected FFFFFFEF |
| `gambatte/oamdma/oamdma_srcE000_busypopFF7F_dmg08_out6576FFAA_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FFAA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcE000_busypopFFFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcE000_busypush0001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA34, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcE000_busypush8001_dmg08_out65221255_cgb04c_outFFAA1255.gbc` | fail: printed 65761255, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcE000_busypushA001_2_dmg08_out4576AAFF_cgb04c_out55FFAAFF.gbc` | fail: printed 6576AAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcE000_busypushA001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA55, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcE000_busypushC001_2_dmg08_out4522FF34_cgb04c_outFFAAFF55.gbc` | fail: printed 6576FF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcE000_busypushC001_dmg08_out45221234_cgb04c_outFFAA1255.gbc` | fail: printed 6576AA34, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcE000_busypushE001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcE000_busypushF001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcE000_busypushFE01_dmg08_out65221298_cgb04c_outFFFFAAFF.gbc` | fail: printed 65761298, expected FFFFAAFF |
| `gambatte/oamdma/oamdma_srcE000_busypushFEA1_dmg08_out65768700_cgb04c_outFFFFFF34.gbc` | fail: printed 65768700, expected FFFFFF34 |
| `gambatte/oamdma/oamdma_srcE000_busypushFF01_dmg08_out657600DF_cgb04c_outFFFF12DF.gbc` | fail: printed 657600DF, expected FFFF12DF |
| `gambatte/oamdma/oamdma_srcE000_busypushFF81_dmg08_out6576FF55_cgb04c_outFFFFFF55.gbc` | fail: printed 6576FF55, expected FFFFFF55 |
| `gambatte/oamdma/oamdma_srcE000_readFE00_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/oamdma/oamdma_srcEF00_busypop7FFF_dmg08_out657665AA_cgb04c_outFFFFFFAA.gbc` | fail: printed 657655AA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcEF00_busypop9FFF_2_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcEF00_busypop9FFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcEF00_busypopBFFF_2_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FF76, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcEF00_busypopBFFF_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 65765576, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcEF00_busypopDFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcEF00_busypopEFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcEF00_busypopFDFF_dmg08_out657665FF_cgb04c_outFFFF55FF.gbc` | fail: printed 657665FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcEF00_busypopFE9F_dmg08_out6576FFFF_cgb04c_outFFFFFFFF.gbc` | fail: printed 6576FFFF, expected FFFFFFFF |
| `gambatte/oamdma/oamdma_srcEF00_busypopFEFF_dmg08_out6576FFEF_cgb04c_outFFFFFFEF.gbc` | fail: printed 6576FFEF, expected FFFFFFEF |
| `gambatte/oamdma/oamdma_srcEF00_busypopFF7F_dmg08_out6576FFAA_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FFAA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcEF00_busypopFFFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcEF00_busypush0001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA34, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcEF00_busypush8001_dmg08_out65221255_cgb04c_outFFAA1255.gbc` | fail: printed 65761255, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcEF00_busypushA001_2_dmg08_out4576AAFF_cgb04c_out55FFAAFF.gbc` | fail: printed 6576AAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcEF00_busypushA001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA55, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcEF00_busypushC001_2_dmg08_out4522FF34_cgb04c_outFFAAFF55.gbc` | fail: printed 6576FF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcEF00_busypushC001_dmg08_out45221234_cgb04c_outFFAA1255.gbc` | fail: printed 6576AA34, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcEF00_busypushE001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcEF00_busypushF001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcEF00_busypushFE01_dmg08_out65221298_cgb04c_outFFFFAAFF.gbc` | fail: printed 65761298, expected FFFFAAFF |
| `gambatte/oamdma/oamdma_srcEF00_busypushFEA1_dmg08_out65768700_cgb04c_outFFFFFF34.gbc` | fail: printed 65768700, expected FFFFFF34 |
| `gambatte/oamdma/oamdma_srcEF00_busypushFF01_dmg08_out657600DF_cgb04c_outFFFF12DF.gbc` | fail: printed 657600DF, expected FFFF12DF |
| `gambatte/oamdma/oamdma_srcEF00_busypushFF81_dmg08_out6576FF55_cgb04c_outFFFFFF55.gbc` | fail: printed 6576FF55, expected FFFFFF55 |
| `gambatte/oamdma/oamdma_srcF000_busypop7FFF_dmg08_out657665AA_cgb04c_outFFFFFFAA.gbc` | fail: printed 657655AA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcF000_busypop9FFF_2_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcF000_busypop9FFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcF000_busypopBFFF_2_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FF76, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcF000_busypopBFFF_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 65765576, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcF000_busypopDFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcF000_busypopEFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcF000_busypopFDFF_dmg08_out657665FF_cgb04c_outFFFF55FF.gbc` | fail: printed 657665FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcF000_busypopFE9F_dmg08_out6576FFFF_cgb04c_outFFFFFFFF.gbc` | fail: printed 6576FFFF, expected FFFFFFFF |
| `gambatte/oamdma/oamdma_srcF000_busypopFEFF_dmg08_out6576FFEF_cgb04c_outFFFFFFEF.gbc` | fail: printed 6576FFEF, expected FFFFFFEF |
| `gambatte/oamdma/oamdma_srcF000_busypopFF7F_dmg08_out6576FFAA_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FFAA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcF000_busypopFFFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcF000_busypush0001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA34, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcF000_busypush8001_dmg08_out65221255_cgb04c_outFFAA1255.gbc` | fail: printed 65761255, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcF000_busypushA001_2_dmg08_out4576AAFF_cgb04c_out55FFAAFF.gbc` | fail: printed 6576AAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcF000_busypushA001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA55, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcF000_busypushC001_2_dmg08_out4522FF34_cgb04c_outFFAAFF55.gbc` | fail: printed 6576FF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcF000_busypushC001_dmg08_out45221234_cgb04c_outFFAA1255.gbc` | fail: printed 6576AA34, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcF000_busypushE001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcF000_busypushF001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcF000_busypushFE01_dmg08_out65221298_cgb04c_outFFFFAAFF.gbc` | fail: printed 65761298, expected FFFFAAFF |
| `gambatte/oamdma/oamdma_srcF000_busypushFEA1_dmg08_out65768700_cgb04c_outFFFFFF34.gbc` | fail: printed 65768700, expected FFFFFF34 |
| `gambatte/oamdma/oamdma_srcF000_busypushFF01_dmg08_out657600DF_cgb04c_outFFFF12DF.gbc` | fail: printed 657600DF, expected FFFF12DF |
| `gambatte/oamdma/oamdma_srcF000_busypushFF81_dmg08_out6576FF55_cgb04c_outFFFFFF55.gbc` | fail: printed 6576FF55, expected FFFFFF55 |
| `gambatte/oamdma/oamdma_srcF000_busyread0000_1_dmg08_out9_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcF000_busyreadA000_dmg08_out6_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcFD00_readFE00_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_busypop7FFF_dmg08_out657665AA_cgb04c_outFFFFFFAA.gbc` | fail: printed 657655AA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFE00_busypop9FFF_2_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFE00_busypop9FFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFE00_busypopBFFF_2_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FF76, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFE00_busypopBFFF_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 65765576, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFE00_busypopDFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcFE00_busypopEFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcFE00_busypopFDFF_dmg08_out657665FF_cgb04c_outFFFF55FF.gbc` | fail: printed 657665FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFE00_busypopFE9F_dmg08_out6576FFFF_cgb04c_outFFFFFFFF.gbc` | fail: printed 6576FFFF, expected FFFFFFFF |
| `gambatte/oamdma/oamdma_srcFE00_busypopFEFF_dmg08_out6576FFEF_cgb04c_outFFFFFFEF.gbc` | fail: printed 6576FFEF, expected FFFFFFEF |
| `gambatte/oamdma/oamdma_srcFE00_busypopFF7F_dmg08_out6576FFAA_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FFAA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFE00_busypopFFFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFE00_busypush0001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA34, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcFE00_busypush8001_dmg08_out65221255_cgb04c_outFFAA1255.gbc` | fail: printed 65761255, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcFE00_busypushA001_2_dmg08_out4576AAFF_cgb04c_out55FFAAFF.gbc` | fail: printed 6576AAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcFE00_busypushA001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA55, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcFE00_busypushC001_2_dmg08_out4522FF34_cgb04c_outFFAAFF55.gbc` | fail: printed 6576FF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcFE00_busypushC001_dmg08_out45221234_cgb04c_outFFAA1255.gbc` | fail: printed 6576AA34, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcFE00_busypushE001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcFE00_busypushF001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcFE00_busypushFE01_dmg08_out65221298_cgb04c_outFFFFAAFF.gbc` | fail: printed 65761298, expected FFFFAAFF |
| `gambatte/oamdma/oamdma_srcFE00_busypushFEA1_dmg08_out65768700_cgb04c_outFFFFFF34.gbc` | fail: printed 65768700, expected FFFFFF34 |
| `gambatte/oamdma/oamdma_srcFE00_busypushFF01_dmg08_out657600DF_cgb04c_outFFFF12DF.gbc` | fail: printed 657600DF, expected FFFF12DF |
| `gambatte/oamdma/oamdma_srcFE00_busypushFF81_dmg08_out6576FF55_cgb04c_outFFFFFF55.gbc` | fail: printed 6576FF55, expected FFFFFF55 |
| `gambatte/oamdma/oamdma_srcFE00_busyread0000_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_busyreadA000_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_readFE00_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_srcFF00_busypop7FFF_dmg08_out657665AA_cgb04c_outFFFFFFAA.gbc` | fail: printed 657655AA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFF00_busypop9FFF_2_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFF00_busypop9FFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFF00_busypopBFFF_2_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FF76, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFF00_busypopBFFF_dmg08_out65766576_cgb04c_outFFFFFFAA.gbc` | fail: printed 65765576, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFF00_busypopDFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcFF00_busypopEFFF_dmg08_out65766576_cgb04c_outFFFF55AA.gbc` | fail: printed 65766576, expected FFFF55AA |
| `gambatte/oamdma/oamdma_srcFF00_busypopFDFF_dmg08_out657665FF_cgb04c_outFFFF55FF.gbc` | fail: printed 657665FF, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFF00_busypopFE9F_dmg08_out6576FFFF_cgb04c_outFFFFFFFF.gbc` | fail: printed 6576FFFF, expected FFFFFFFF |
| `gambatte/oamdma/oamdma_srcFF00_busypopFEFF_dmg08_out6576FFEF_cgb04c_outFFFFFFEF.gbc` | fail: printed 6576FFEF, expected FFFFFFEF |
| `gambatte/oamdma/oamdma_srcFF00_busypopFF7F_dmg08_out6576FFAA_cgb04c_outFFFFFFAA.gbc` | fail: printed 6576FFAA, expected FFFFFFAA |
| `gambatte/oamdma/oamdma_srcFF00_busypopFFFF_dmg08_out65765576_cgb04c_outFFFF55FF.gbc` | fail: printed 657655AA, expected FFFF55FF |
| `gambatte/oamdma/oamdma_srcFF00_busypush0001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA34, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcFF00_busypush8001_dmg08_out65221255_cgb04c_outFFAA1255.gbc` | fail: printed 65761255, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcFF00_busypushA001_2_dmg08_out4576AAFF_cgb04c_out55FFAAFF.gbc` | fail: printed 6576AAFF, expected 55FFAAFF |
| `gambatte/oamdma/oamdma_srcFF00_busypushA001_dmg08_out4576AA34_cgb04c_out55FFAA34.gbc` | fail: printed 6576AA55, expected 55FFAA34 |
| `gambatte/oamdma/oamdma_srcFF00_busypushC001_2_dmg08_out4522FF34_cgb04c_outFFAAFF55.gbc` | fail: printed 6576FF34, expected FFAAFF55 |
| `gambatte/oamdma/oamdma_srcFF00_busypushC001_dmg08_out45221234_cgb04c_outFFAA1255.gbc` | fail: printed 6576AA34, expected FFAA1255 |
| `gambatte/oamdma/oamdma_srcFF00_busypushE001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcFF00_busypushF001_dmg08_out45221234_cgb04c_outFFFFAA55.gbc` | fail: printed 65761234, expected FFFFAA55 |
| `gambatte/oamdma/oamdma_srcFF00_busypushFE01_dmg08_out65221298_cgb04c_outFFFFAAFF.gbc` | fail: printed 65761298, expected FFFFAAFF |
| `gambatte/oamdma/oamdma_srcFF00_busypushFEA1_dmg08_out65768700_cgb04c_outFFFFFF34.gbc` | fail: printed 65768700, expected FFFFFF34 |
| `gambatte/oamdma/oamdma_srcFF00_busypushFF01_dmg08_out657600DF_cgb04c_outFFFF12DF.gbc` | fail: printed 657600DF, expected FFFF12DF |
| `gambatte/oamdma/oamdma_srcFF00_busypushFF81_dmg08_out6576FF55_cgb04c_outFFFFFF55.gbc` | fail: printed 6576FF55, expected FFFFFF55 |
| `gambatte/oamdma/oamdma_srcFF00_busyread0000_dmg08_out1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcFF00_busyreadA000_dmg08_out1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_srcFF00_readFE00_dmg08_out1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_srcFF00_readFE45_dmg08_out1_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdmasrc80_halt_lycirq_read8000_dmg08_cgb04c_out81.gbc` | fail: printed A0, expected 81 |
| `gambatte/oamdma/oamdmasrc80_halt_m2irq_read8000_dmg08_cgb04c_out81.gbc` | fail: printed 2A, expected 81 |
| `gambatte/oamdma/oamdmasrcC000_hdmasrc0000_cgb04c_out0A940C0D.gbc` | fail: printed 0A0B0C0D, expected 0A940C0D |
| `gambatte/oamdma/oamdmasrcC0_speedchange_readC000_cgb04c_out11.gbc` | fail: printed 10, expected 11 |
| `gambatte/scx_during_m3/scx1_scx0_during_m3_1.gbc` | fail: 13960 pixels differ from reference |
| `gambatte/scx_during_m3/scx2_scx1_during_m3_1.gbc` | fail: 14104 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_1.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_2.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_3.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_5.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_1.gbc` | fail: 67 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_2.gbc` | fail: 9581 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_3.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_5.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_1.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_2.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_3.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_5.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_1.gbc` | fail: 5 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_2.gbc` | fail: 715 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_3.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_5.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_1.gbc` | fail: 1 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_2.gbc` | fail: 143 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_3.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_4.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_5.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_1.gbc` | fail: 144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_2.gbc` | fail: 1296 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_3.gbc` | fail: 2448 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_4.gbc` | fail: 3575 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_1.gbc` | fail: 151 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_2.gbc` | fail: 1153 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_3.gbc` | fail: 2440 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_4.gbc` | fail: 2448 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_5.gbc` | fail: 2439 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_6.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_7.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_ds_8.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_attrib_during_m3_spx1_ds.gbc` | fail: 1508 pixels differ from reference |
| `gambatte/scx_during_m3/scx_attrib_during_m3_spx2_ds.gbc` | fail: 1508 pixels differ from reference |
| `gambatte/scx_during_m3/scx_during_m3_spx2.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scx_during_m3/scx_during_m3_spx2_ds.gbc` | fail: 1096 pixels differ from reference |
| `gambatte/scx_during_m3/scx_m3_extend_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/scy/scy_during_m3_ds_1.gbc` | fail: 8 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_2.gbc` | fail: 1144 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_3.gbc` | fail: 1008 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_4.gbc` | fail: 1008 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_5.gbc` | fail: 144 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_6.gbc` | fail: 1152 pixels differ from reference |
| `gambatte/scy/scy_during_m3_ds_7.gbc` | fail: 1008 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_4.gbc` | fail: 100 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_ds_1.gbc` | fail: 896 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_ds_2.gbc` | fail: 896 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_ds_3.gbc` | fail: 128 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_ds_4.gbc` | fail: 960 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_4.gbc` | fail: 100 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_ds_1.gbc` | fail: 1128 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_ds_2.gbc` | fail: 904 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_ds_3.gbc` | fail: 136 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_ds_4.gbc` | fail: 1080 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_4.gbc` | fail: 100 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0B_4.gbc` | fail: 100 pixels differ from reference |
| `gambatte/serial/nopx1_start83_wait_read_if_1_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/nopx2_start_wait_read_if_1_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start83_late_div_write_wait_read_if_1a_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start_late_div_write_wait_read_if_2b_dmg08_cgb04c_outE8.gbc` | fail: printed E0, expected E8 |
| `gambatte/serial/start_late_div_write_wait_read_if_3a_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start_wait_trigger_int8_read_if_2_dmg08_outE8_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start_wait_trigger_int8_read_if_ds_2_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/sound/ch1_init_reset_sweep_counter_timing_nr52_3_dmg08_out0_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/sound/ch2_init_reset_length_counter_timing_nr52_1_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_init_reset_length_counter_timing_nr52_2_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_init_reset_length_counter_timing_nr52_4_dmg08_out2_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_late_div_write_nr52_ds_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/sound/ch2_late_reset_nr52_2b_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_late_reset_nr52_ds_1b_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_late_reset_nr52_ds_2b_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sound/ch2_reset_length_counter_timing_nr52_ds_2_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/sound/ch3_reset_nop_nr4init_freq7ff_read_ff30_ds_2_cgb04c_out32.gbc` | fail: printed 10, expected 32 |
| `gambatte/speedchange/m2int_m3stat_scx1_lcdoffds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/speedchange/speedchange2_ch2_nr52_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange2_ch2_nr52_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange2_ch2_nr52_ds_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange2_ch2_nr52_ds_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange2_div_2_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_div_nop_2_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_frame1_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_lcdoff_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_lcdoff_nop_m2int_m3stat_scx4_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_lcdoff_nopx2_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_ly44_m3_ly_1_cgb04c_out25.gbc` | fail: printed 5F, expected 25 |
| `gambatte/speedchange/speedchange2_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_m3stat_scx3_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_nop_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_nop_m3stat_scx4_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_nopx2_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_nopx2_m3stat_scx3_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_stat_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_ly44_m3_stat_4_cgb04c_outC2.gbc` | fail: printed C0, expected C2 |
| `gambatte/speedchange/speedchange2_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_nop_lcdoff_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_nop_lcdoff_nop_m2int_m3stat_scx4_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_nop_lcdoff_nopx2_m2int_m3stat_scx2_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/speedchange/speedchange2_nop_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_nop_ly44_m3_m3stat_scx3_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_nop_ly44_m3_nop_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_nop_ly44_m3_nop_m3stat_scx4_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange2_nop_m2int_m3stat_scx1_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/speedchange/speedchange2_tima00_1b_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_tima00_2a_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_tima00_2b_cgb04c_out02.gbc` | fail: printed 00, expected 02 |
| `gambatte/speedchange/speedchange2_tima01_1_cgb04c_out09.gbc` | fail: printed 08, expected 09 |
| `gambatte/speedchange/speedchange2_tima01_2_cgb04c_out0A.gbc` | fail: printed 08, expected 0A |
| `gambatte/speedchange/speedchange2_tima01_nop_1_cgb04c_out0A.gbc` | fail: printed 08, expected 0A |
| `gambatte/speedchange/speedchange2_tima01_nop_2_cgb04c_out0B.gbc` | fail: printed 08, expected 0B |
| `gambatte/speedchange/speedchange2_tima02_1b_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/speedchange/speedchange2_tima02_2a_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/speedchange/speedchange2_tima02_2b_cgb04c_out04.gbc` | fail: printed 02, expected 04 |
| `gambatte/speedchange/speedchange2_tima03_1b_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_tima03_2a_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange2_tima03_2b_cgb04c_out02.gbc` | fail: printed 00, expected 02 |
| `gambatte/speedchange/speedchange3_ch2_nr52_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange3_ch2_nr52_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange3_ly44_m3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange3_ly44_m3_m3stat_scx2_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange3_ly44_m3_nop_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange3_ly44_m3_nop_m3stat_scx2_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange3_nop_ly44_m3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange3_nop_ly44_m3_m3stat_scx2_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange4_ch2_nr52_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange4_ch2_nr52_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange4_ly44_m3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_ly44_m3_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange4_ly44_m3_m3stat_scx2_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange4_ly44_m3_nop_m3stat_scx3_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_ly44_m3_nop_m3stat_scx3_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange4_ly44_m3_nop_m3stat_scx4_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_ly44_m3_nop_m3stat_scx4_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange4_nop_ly44_m3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_nop_ly44_m3_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange4_nop_ly44_m3_m3stat_scx2_1_cgb04c_outC3.gbc` | fail: printed C2, expected C3 |
| `gambatte/speedchange/speedchange4_nop_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C2, expected C0 |
| `gambatte/speedchange/speedchange5_ch2_nr52_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange5_ch2_nr52_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange5_ly44_m3_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange5_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange5_ly44_m3_nop_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange5_ly44_m3_nop_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange5_nop_ly44_m3_m3stat_scx1_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange5_nop_ly44_m3_m3stat_scx2_2_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange_ch2_nr52_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange_ch2_nr52_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange_ch2_nr52_ds_1b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange_ch2_nr52_ds_2b_cgb04c_outF0.gbc` | fail: printed F2, expected F0 |
| `gambatte/speedchange/speedchange_div_2_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange_div_nop_2_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange_lcdoff_tima00_1_cgb04c_out80.gbc` | fail: printed 00, expected 80 |
| `gambatte/speedchange/speedchange_lcdoff_tima01_2_cgb04c_out09.gbc` | fail: printed 08, expected 09 |
| `gambatte/speedchange/speedchange_ly44_m3_ly_cgb04c_out39.gbc` | fail: printed 4D, expected 39 |
| `gambatte/speedchange/speedchange_ly44_m3_m3stat_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nop_m3stat_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nop_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx2_m3stat_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx2_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx3_m3stat_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx3_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx4_m3stat_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_nopx4_m3stat_scx1_1_cgb04c_outC3.gbc` | fail: printed C0, expected C3 |
| `gambatte/speedchange/speedchange_ly44_m3_stat_1_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange_ly44_m3_stat_2_cgb04c_outC2.gbc` | fail: printed C3, expected C2 |
| `gambatte/speedchange/speedchange_ly44_m3_stat_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange_ly97_ly_cgb04c_out8C.gbc` | fail: printed 06, expected 8C |
| `gambatte/speedchange/speedchange_ly97_stat_cgb04c_outC0.gbc` | fail: printed C3, expected C0 |
| `gambatte/speedchange/speedchange_tima00_1a_cgb04c_out80.gbc` | fail: printed 00, expected 80 |
| `gambatte/speedchange/speedchange_tima00_1b_cgb04c_out81.gbc` | fail: printed 00, expected 81 |
| `gambatte/speedchange/speedchange_tima00_2a_cgb04c_out81.gbc` | fail: printed 01, expected 81 |
| `gambatte/speedchange/speedchange_tima00_2b_cgb04c_out82.gbc` | fail: printed 01, expected 82 |
| `gambatte/speedchange/speedchange_tima01_2_cgb04c_out08.gbc` | fail: printed 07, expected 08 |
| `gambatte/speedchange/speedchange_tima01_nop_1_cgb04c_out07.gbc` | fail: printed 06, expected 07 |
| `gambatte/speedchange/speedchange_tima01_nop_2_cgb04c_out08.gbc` | fail: printed 06, expected 08 |
| `gambatte/speedchange/speedchange_tima02_1b_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/speedchange/speedchange_tima02_2a_cgb04c_out03.gbc` | fail: printed 02, expected 03 |
| `gambatte/speedchange/speedchange_tima02_2b_cgb04c_out04.gbc` | fail: printed 02, expected 04 |
| `gambatte/speedchange/speedchange_tima03_1b_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange_tima03_2a_cgb04c_out01.gbc` | fail: printed 00, expected 01 |
| `gambatte/speedchange/speedchange_tima03_2b_cgb04c_out02.gbc` | fail: printed 00, expected 02 |
| `gambatte/sprites/10spritesPrLine_10xposA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/sprites/10spritesprline_1xposa1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_1xposa3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_1xposa5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_1xposa6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_1xposa7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap2_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2overlap8_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/10spritesprline_2xposa2overlap8_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/1spritesPrLine_1sprite8pBgCover_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/1spritesPrLine_1sprite8pBgPrior_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/1spritesPrLine_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/3spritesPrLine_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/5spritesPrLine_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/7spritesPrLine_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/9spritesPrLine_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange2_sp00_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp00_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange2_sp01_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp01_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange2_sp02_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp39_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_4_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange_sp00_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp01_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp01_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp01_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp02_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp39_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp39_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_late_scx4_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/space/10spritesPrLine_nr10space11_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space13_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_nr10space9_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/10spritesPrLine_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap1_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap1_offset5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap3_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap5_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap6_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap7_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap8_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/2overlap8_offset4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap1_scx6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap1_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap2_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap2_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap2_scx4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap2_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap2_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap3_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap3_scx4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap3_scx6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap4_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap4_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap4_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap4_scx6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap4_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap5_rev_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap5_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap5_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_rev_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx2_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap6_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_rev_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx2_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap7_scx6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx1_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx2_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx3_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx4_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/3overlap8_scx7_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx08_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx08_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx09_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx0A_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx0B_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx0C_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx0D_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/space/9pos8_wx0E_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/tima/tc00_irq_late_retrigger_2_dmg08_outE4_cgb04c_outE0.gbc` | fail: printed E4, expected E0 |
| `gambatte/tima/tc00_irq_late_retrigger_ds_2_cgb04c_outE0.gbc` | fail: printed E4, expected E0 |
| `gambatte/tima/tc00_late_stop_inc_2_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc00_late_stop_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc00_late_stop_of_2_dmg08_cgb04c_outFE.gbc` | fail: printed FF, expected FE |
| `gambatte/tima/tc00_start_2_cgb04c_outF1.gbc` | fail: printed F0, expected F1 |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_1_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/tima/tc01_1stopstart_offset1_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_1_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/tima/tc01_1stopstart_offset2_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_late_stop_inc_2_dmg08_cgb04c_outFE.gbc` | fail: printed FD, expected FE |
| `gambatte/tima/tc01_late_stop_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_late_stop_of_2_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/vram_m3/postread_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/vram_m3/preread_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/vram_m3/preread_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/vram_m3/preread_ds_lcdoffset1_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/vram_m3/preread_lcdoffset1_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/vram_m3/preread_lcdoffset2_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/vram_m3/prewrite_lcdoffset2_1_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/vramw_m3end/vramw_m3end_scx5_ds_2_cgb04c_out7.gbc` | fail: printed 0, expected 7 |
| `gambatte/vramw_m3end/vramw_m3end_scx5_ds_4_cgb04c_out0.gbc` | fail: printed 5, expected 0 |
| `gambatte/window/arg/late_enable_afterVblank_4_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_10to0_ly1_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_1toFF_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_1toFF_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_1toFF_ds_lcdoffset1_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_2toFF_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_FFto0_ly2_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto1_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_FFto2_ly2_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx3_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx00_wx0f_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx00_wx11_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx00_wx0f_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx00_wx10_ds_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx03_wx0f_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx03_wx10_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx03_wx11_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx03_wx12_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_scx2_0_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_scx2_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_scx3_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_scx5_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_scx5_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_wx0f_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_enable_afterVblank_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_enable_afterVblank_ds_lcdoffset1_2_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_enable_ly0_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx3_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_reenable_scx5_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_scx_late_disable_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_wx_scx3_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_scx5_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wy_ds_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wy_ds_lcdoffset1_2_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wy_lcdoffset1_1_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/m2int_wx03_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx07_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_firstline_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_oambusyread_2_dmg08_out5_cgb04c_out0.gbc` | fail: printed 5, expected 0 |
| `gambatte/window/m2int_wxA6_scx2_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx2_m3stat_3_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx3_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx3_m3stat_3_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx5_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx5_m3stat_ds_1_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_spxA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/window/m2int_wxA6_spxA7_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_spxA7_m3stat_4_dmg08_out0_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_vrambusyread_2_dmg08_out5_cgb04c_out0.gbc` | fail: printed 5, expected 0 |
| `gambatte/window/on_screen/wxA6_3.gbc` | fail: 10628 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_1.gbc` | fail: 19084 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_2.gbc` | fail: 19084 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_3.gbc` | fail: 120 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_4.gbc` | fail: 120 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_scx7.gbc` | fail: 10840 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy00.gbc` | fail: 21816 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01.gbc` | fail: 21505 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_weoff_ly02_weon_ly60.gbc` | fail: 6995 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_wxA5_ly02.gbc` | fail: 2 pixels differ from reference |

</details>
