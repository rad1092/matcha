| Suite | Passed | Total | |
|---|---:|---:|---|
| blargg | 43 | 43 | CPU, timing, sound and OAM-bug tests by Shay Green |
| mooneye | 94 | 94 | Mooneye Test Suite: acceptance + emulator-only (DMG-applicable) |
| dmg-acid2 | 1 | 1 | PPU rendering torture test |
| gambatte | 1353 | 1783 | Gambatte hardware-verified tests: DMG hex-result and screenshot cases |
| mealybug | 1 | 24 | Mealybug Tearoom: mid-scanline PPU effects (needs a pixel FIFO) |
| **all** | **1492** | **1945** | |

Left out because they cannot pass on hardware either:

- `blargg/oam_bug/rom_singles/7-timing_effect.gb`: its cartridge-RAM log of 19 OAM dumps outgrows the 8 KiB RAM and overwrites the test's own code in WRAM (SameBoy crashes the same way); the same test passes inside `oam_bug.gb`

<details><summary>Not passing</summary>

| ROM | Result |
|---|---|
| `gambatte/bgtiledata/bgtiledata_spx08_1.gbc` | fail: 18312 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_2.gbc` | fail: 16248 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_3.gbc` | fail: 16008 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_4.gbc` | fail: 13944 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_1.gbc` | fail: 18432 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_2.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_3.gbc` | fail: 16128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx09_4.gbc` | fail: 13952 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_1.gbc` | fail: 18432 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_2.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_3.gbc` | fail: 16128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_4.gbc` | fail: 13952 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_1.gbc` | fail: 18424 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_2.gbc` | fail: 16136 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_3.gbc` | fail: 16120 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0B_4.gbc` | fail: 13832 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_1.gbc` | fail: 17400 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_2.gbc` | fail: 17160 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_3.gbc` | fail: 15096 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx08_4.gbc` | fail: 14856 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_1.gbc` | fail: 17400 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_2.gbc` | fail: 17160 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_3.gbc` | fail: 15096 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx09_4.gbc` | fail: 14856 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_1.gbc` | fail: 17400 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_2.gbc` | fail: 17160 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_3.gbc` | fail: 15096 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_4.gbc` | fail: 14856 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0B_1.gbc` | fail: 17288 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0B_2.gbc` | fail: 17272 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0B_3.gbc` | fail: 14984 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0B_4.gbc` | fail: 14968 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_1.gb` | fail: 23040 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_2.gb` | fail: 22611 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_3.gb` | fail: 22035 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_4.gb` | fail: 21312 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_5.gb` | fail: 20160 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_scx1_1.gb` | fail: 23040 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_scx1_4.gb` | fail: 21312 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_scx2_1.gb` | fail: 22897 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_1.gb` | fail: 22451 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_2.gb` | fail: 21879 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_3.gb` | fail: 21164 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_4.gb` | fail: 20020 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_1.gb` | fail: 22754 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_2.gb` | fail: 22180 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_3.gb` | fail: 21604 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_4.gb` | fail: 21026 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_5.gb` | fail: 20160 pixels differ from reference |
| `gambatte/enable_display/enable_display_ly0_sprites_m0stat_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/enable_display/enable_display_ly0_wemaster_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/enable_display/ly0_late_scx7_m3stat_scx0_1_dmg08_cgb04c_out87.gbc` | fail: printed 84, expected 87 |
| `gambatte/enable_display/ly0_late_scx7_m3stat_scx0_2_dmg08_out87_cgb04c_out84.gbc` | fail: printed 84, expected 87 |
| `gambatte/enable_display/ly0_late_scx7_m3stat_scx1_1_dmg08_cgb04c_out87.gbc` | fail: printed 84, expected 87 |
| `gambatte/enable_display/ly0_late_scx7_m3stat_scx3_1_dmg08_cgb04c_out87.gbc` | fail: printed 84, expected 87 |
| `gambatte/halt/ifandie_ei_halt_m2int_m0stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/late_m0int_halt_m0stat_scx2_3a_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/late_m0int_halt_m0stat_scx3_2b_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/halt/late_m0irq_halt_dec_scx3_1_dmg08_cgb04c_out7.gbc` | fail: printed 6, expected 7 |
| `gambatte/halt/late_m0irq_halt_m0stat_scx2_3a_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/late_m0irq_halt_m0stat_scx2_4a_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/late_m0irq_halt_m0stat_scx3_3a_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/late_m0irq_halt_m0stat_scx3_4a_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/halt/noime_ifandie_m2int_m0stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/irq_precedence/late_m0irq_retrigger_scx1_1_dmg08_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx2_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx2_halt_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx3_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx3_halt_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/ly0/lycint152_lyc0irq_late_retrigger_2_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/ly0/lycint152_lyc153irq_late_retrigger_2_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_ifw_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/ff41_disable_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/lcdoff_lycirqen_1_dmg08_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lcdoff_lycirqen_4_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lycwirq_trigger_ly00_stat50_2_dmg08_outE0_cgb04c_outE2.gbc` | fail: printed E2, expected E0 |
| `gambatte/lycm2int/lycm2int_m0stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycm2int/lycm2int_m2irq_1_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/lycm2int/m2irq_before_lycint_1_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m0enable/enable_wxA6_2x_spxA7_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m0enable/lycdisable_ff45_scx3_3_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m0int_m0stat/m0int_m0stat_scx2_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m1/lyc143_late_m2enable_lycdisable_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/lycint143_m1irq_late_retrigger_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/lycint_vblankirq_late_retrigger_2_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/m1/m1irq_m2disable_lycdisable_3_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_1_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_2_dmg08_out1_cgb04c_out3.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m2m1irq_ifw_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m2enable/late_enable_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2enable/late_enable_after_lycint_disable_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2int_m0irq/m2int_m0irq_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx2_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx2_ei_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx2_reti_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx3_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx3_di_1_dmg08_cgb04c_out0.gbc` | fail: printed 8, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx3_ei_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx3_ie_1_dmg08_cgb04c_out0.gbc` | fail: printed 8, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx3_reti_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx4_ifw_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx4_ifw_3_dmg08_cgb04c_out8.gbc` | fail: printed 0, expected 8 |
| `gambatte/m2int_m0irq/m2int_m0irq_scx5_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m0stat/m2int_m0stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m2irq/m2int_m2irq_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m2int_m2irq/m2int_m2irq_ifw_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2int_m2irq/m2int_m2irq_late_retrigger_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/m2int_m2stat/m2int_m2stat_1_dmg08_cgb04c_out2.gbc` | fail: printed 3, expected 2 |
| `gambatte/m2int_m3stat/m2int_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/late_scx4_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx2_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx3_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/m2int_m3stat/scx/m2int_scx5_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/miscmstatirq/lycstatwirq_trigger_ly00_10_50_1_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/miscmstatirq/lycstatwirq_trigger_m0_late_ly44_lyc44_08_40_4_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/miscmstatirq/m1statwirq_3_dmg08_out2.gb` | fail: printed 0, expected 2 |
| `gambatte/oam_access/postread_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oam_access/postread_scx2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oam_access/postread_scx3_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oam_access/postread_scx5_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/oam_access/postwrite_1_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oam_access/preread_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oam_access/prewrite_2_dmg08_out1_cgb04c_out0.gbc` | fail: printed 0, expected 1 |
| `gambatte/oamdma/late_sp00x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp00y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp01x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp01y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp02x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp02y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39x_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/late_sp39y_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_late_halt_stat_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/oamdma/oamdma_src0000_busycallAFFF_dmg08_cgb04c_outFF8F.gbc` | fail: printed 0000, expected FF8F |
| `gambatte/oamdma/oamdma_src0000_busyint0002_dmg08_cgb04c_outFF941234.gbc` | fail: printed 76871234, expected FF941234 |
| `gambatte/oamdma/oamdma_src0000_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src0000_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src0000_busypushA001_2_dmg08_cgb04c_out5576AAFF.gbc` | fail: printed 6576AAFF, expected 5576AAFF |
| `gambatte/oamdma/oamdma_src0000_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AAFF, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src0000_busypushC001_2_dmg08_out55AAFF34_cgb04c_out65AAFF55.gbc` | fail: printed 6576FF34, expected 55AAFF34 |
| `gambatte/oamdma/oamdma_src0000_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src0000_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src0000_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src0000_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65761298, expected 65AA1298 |
| `gambatte/oamdma/oamdma_src0000_busyrst0002_dmg08_cgb04c_outFF8DFA9E.gbc` | fail: printed 7687FA9E, expected FF8DFA9E |
| `gambatte/oamdma/oamdma_src0000_srambankchange_1_dmg08_cgb04c_out4.gbc` | fail: tile 0xff at column 0 is not a hex digit |
| `gambatte/oamdma/oamdma_src7F00_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src7F00_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src7F00_busypushA001_2_dmg08_cgb04c_out5576AAFF.gbc` | fail: printed 6576AAFF, expected 5576AAFF |
| `gambatte/oamdma/oamdma_src7F00_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 6576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_src7F00_busypushC001_2_dmg08_out55AAFF34_cgb04c_out65AAFF55.gbc` | fail: printed 6576FF34, expected 55AAFF34 |
| `gambatte/oamdma/oamdma_src7F00_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src7F00_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src7F00_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 65761234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_src7F00_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65761298, expected 65AA1298 |
| `gambatte/oamdma/oamdma_src8000_busypush8001_dmg08_out55761234_cgb04c_out00761234.gbc` | fail: printed 65761234, expected 55761234 |
| `gambatte/oamdma/oamdma_src8000_busypushA001_2_dmg08_out65AA12FF_cgb04c_out650012FF.gbc` | fail: printed 657612FF, expected 65AA12FF |
| `gambatte/oamdma/oamdma_src8000_busypushA001_dmg08_out65AA1255_cgb04c_out65001255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_src8000_busywrite8000_dmg08_cgb04c_out0.gbc` | fail: printed 4, expected 0 |
| `gambatte/oamdma/oamdma_src8000_srcchange0000_busyinc_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_src9F00_busypush8001_dmg08_out55761234_cgb04c_out00761234.gbc` | fail: printed 65761234, expected 55761234 |
| `gambatte/oamdma/oamdma_src9F00_busypushA001_2_dmg08_out65AA12FF_cgb04c_out650012FF.gbc` | fail: printed 657612FF, expected 65AA12FF |
| `gambatte/oamdma/oamdma_src9F00_busypushA001_dmg08_out65AA1255_cgb04c_out65001255.gbc` | fail: printed 65761255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcA000_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 4576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcA000_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65221255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcA000_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 4576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcA000_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcA000_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcA000_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcA000_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65221298, expected 65AA1298 |
| `gambatte/oamdma/oamdma_srcA000_busywrite4000_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/oamdma/oamdma_srcBF00_busypush0001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 4576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcBF00_busypush8001_dmg08_cgb04c_out65AA1255.gbc` | fail: printed 65221255, expected 65AA1255 |
| `gambatte/oamdma/oamdma_srcBF00_busypushA001_dmg08_cgb04c_out5576AA34.gbc` | fail: printed 4576AA34, expected 5576AA34 |
| `gambatte/oamdma/oamdma_srcBF00_busypushC001_dmg08_out55AA1234_cgb04c_out65AA1255.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcBF00_busypushE001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcBF00_busypushF001_dmg08_out55AA1234_cgb04c_out6576AA55.gbc` | fail: printed 45221234, expected 55AA1234 |
| `gambatte/oamdma/oamdma_srcBF00_busypushFE01_dmg08_out65AA1298_cgb04c_out6576AA98.gbc` | fail: printed 65221298, expected 65AA1298 |
| `gambatte/oamdma/oamdma_srcFE00_busyread0000_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_busyreadA000_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_busyreadC000_dmg08_out0_cgb_xoutblank.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdma_srcFE00_readFE00_dmg08_cgb04c_out0.gbc` | fail: printed 1, expected 0 |
| `gambatte/oamdma/oamdmasrc80_halt_lycirq_read8000_dmg08_cgb04c_out81.gbc` | fail: printed A0, expected 81 |
| `gambatte/oamdma/oamdmasrc80_halt_m2irq_read8000_dmg08_cgb04c_out81.gbc` | fail: printed 2B, expected 81 |
| `gambatte/scx_during_m3/scx2_scx0_during_m3_1.gbc` | fail: 14104 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_1.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_2.gbc` | fail: 14960 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_3.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_4.gbc` | fail: 14960 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_5.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0060c0/scx_during_m3_6.gbc` | fail: 14960 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_1.gbc` | fail: 17568 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_2.gbc` | fail: 18249 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_3.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_4.gbc` | fail: 14960 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_5.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0063c0/scx_during_m3_6.gbc` | fail: 14960 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_1.gbc` | fail: 16992 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_2.gbc` | fail: 17712 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_3.gbc` | fail: 18459 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_4.gbc` | fail: 22312 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_5.gbc` | fail: 21168 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0360c0/scx_during_m3_6.gbc` | fail: 21160 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_1.gbc` | fail: 13824 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_2.gbc` | fail: 14963 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_3.gbc` | fail: 14256 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_4.gbc` | fail: 15392 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_5.gbc` | fail: 14256 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0363c0/scx_during_m3_6.gbc` | fail: 15392 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_1.gbc` | fail: 18720 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_2.gbc` | fail: 19292 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_3.gbc` | fail: 19261 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_4.gbc` | fail: 15392 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_5.gbc` | fail: 14256 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0367c0/scx_during_m3_6.gbc` | fail: 15392 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_1.gbc` | fail: 20448 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_2.gbc` | fail: 20736 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_3.gbc` | fail: 21024 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_4.gbc` | fail: 21315 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_5.gbc` | fail: 21744 pixels differ from reference |
| `gambatte/scx_during_m3/scx_0761c0/scx_during_m3_6.gbc` | fail: 21736 pixels differ from reference |
| `gambatte/scx_during_m3/scx_during_m3_spx0.gbc` | fail: 15040 pixels differ from reference |
| `gambatte/scx_during_m3/scx_during_m3_spx1.gbc` | fail: 15040 pixels differ from reference |
| `gambatte/scx_during_m3/scx_during_m3_spx2.gbc` | fail: 15040 pixels differ from reference |
| `gambatte/scx_during_m3/scx_m3_extend_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/scy/scx3/scy_during_m3_1.gbc` | fail: 22603 pixels differ from reference |
| `gambatte/scy/scx3/scy_during_m3_2.gbc` | fail: 21587 pixels differ from reference |
| `gambatte/scy/scx3/scy_during_m3_3.gbc` | fail: 20736 pixels differ from reference |
| `gambatte/scy/scx3/scy_during_m3_4.gbc` | fail: 19712 pixels differ from reference |
| `gambatte/scy/scx3/scy_during_m3_5.gbc` | fail: 18432 pixels differ from reference |
| `gambatte/scy/scx3/scy_during_m3_6.gbc` | fail: 17408 pixels differ from reference |
| `gambatte/scy/scy_during_m3_1.gbc` | fail: 23032 pixels differ from reference |
| `gambatte/scy/scy_during_m3_2.gbc` | fail: 22016 pixels differ from reference |
| `gambatte/scy/scy_during_m3_3.gbc` | fail: 20736 pixels differ from reference |
| `gambatte/scy/scy_during_m3_4.gbc` | fail: 19712 pixels differ from reference |
| `gambatte/scy/scy_during_m3_5.gbc` | fail: 18432 pixels differ from reference |
| `gambatte/scy/scy_during_m3_6.gbc` | fail: 17408 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_1.gbc` | fail: 18560 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_2.gbc` | fail: 17280 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx08_3.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_1.gbc` | fail: 18560 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_2.gbc` | fail: 17392 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx09_3.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_1.gbc` | fail: 18560 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_2.gbc` | fail: 17392 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_3.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0B_1.gbc` | fail: 18560 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0B_2.gbc` | fail: 17280 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0B_3.gbc` | fail: 16256 pixels differ from reference |
| `gambatte/serial/nopx2_start_wait_read_if_1_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start_late_div_write_wait_read_if_2b_dmg08_cgb04c_outE8.gbc` | fail: printed E0, expected E8 |
| `gambatte/serial/start_late_div_write_wait_read_if_3a_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/sound/ch1_init_reset_sweep_counter_timing_nr52_1_dmg08_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/sound/ch2_late_reset_nr52_2b_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sprites/10spritesPrLine_10xposA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/sprites/late_disable_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp00_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp01_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp02_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_4_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange_sp01_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp02_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_disable_spx18_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_disable_spx19_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_disable_spx1A_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_disable_spx1B_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_enable_spx18_1_dmg08_out3.gb` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_enable_spx19_1_dmg08_out3.gb` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_enable_spx1A_1_dmg08_out3.gb` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_enable_spx1B_1_dmg08_out3.gb` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_late_disable_spx18_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_late_disable_spx19_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_late_disable_spx1A_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_late_disable_spx1B_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/tima/tc00_1stopstart_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc00_1stopstart_ff_tma_3_dmg08_cgb04c_outFE.gbc` | fail: printed 00, expected FE |
| `gambatte/tima/tc00_1stopstart_offset1_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc00_1stopstart_offset1_ff_tma_3_dmg08_cgb04c_outFE.gbc` | fail: printed 00, expected FE |
| `gambatte/tima/tc00_1stopstart_offset2_ff_tma_1_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc00_1stopstart_offset2_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FE, expected 00 |
| `gambatte/tima/tc00_1stopstart_offset2_ff_tma_3_dmg08_cgb04c_outFE.gbc` | fail: printed FF, expected FE |
| `gambatte/tima/tc00_fe_ff_2_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc00_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc00_ff_tma_3_dmg08_cgb04c_outFE.gbc` | fail: printed 00, expected FE |
| `gambatte/tima/tc00_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc00_late_stop_inc_2_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc00_late_stop_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc00_late_stop_of_2_dmg08_cgb04c_outFE.gbc` | fail: printed FF, expected FE |
| `gambatte/tima/tc00_tc01_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc00_tc01_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed 00, expected F0 |
| `gambatte/tima/tc00_tc01_late_tc00_of_2_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/tima/tc01_1stopstart_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc01_1stopstart_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed 00, expected F0 |
| `gambatte/tima/tc01_1stopstart_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_1_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FE, expected 00 |
| `gambatte/tima/tc01_1stopstart_offset1_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/tima/tc01_1stopstart_offset1_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_1_dmg08_cgb04c_outFF.gbc` | fail: printed F0, expected FF |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed F0, expected 00 |
| `gambatte/tima/tc01_1stopstart_offset2_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed F1, expected F0 |
| `gambatte/tima/tc01_1stopstart_offset2_irq_1_dmg08_cgb04c_outE0.gbc` | fail: printed E4, expected E0 |
| `gambatte/tima/tc01_1stopstart_offset3_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc01_1stopstart_offset3_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed 00, expected F0 |
| `gambatte/tima/tc01_1stopstart_offset3_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_fe_ff_2_dmg08_cgb04c_outFF.gbc` | fail: printed FE, expected FF |
| `gambatte/tima/tc01_ff_tma_2_dmg08_cgb04c_out00.gbc` | fail: printed FF, expected 00 |
| `gambatte/tima/tc01_ff_tma_3_dmg08_cgb04c_outF0.gbc` | fail: printed 00, expected F0 |
| `gambatte/tima/tc01_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_late_stop_inc_2_dmg08_cgb04c_outFE.gbc` | fail: printed FD, expected FE |
| `gambatte/tima/tc01_late_stop_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_late_stop_of_2_dmg08_cgb04c_outF0.gbc` | fail: printed FF, expected F0 |
| `gambatte/tima/tc01_late_tima_inc_2_dmg08_cgb04c_out10.gbc` | fail: printed 11, expected 10 |
| `gambatte/tima/tc01_late_tima_irq_2_dmg08_cgb04c_outE4.gbc` | fail: printed E0, expected E4 |
| `gambatte/tima/tc01_late_tima_tma_1_dmg08_cgb04c_out11.gbc` | fail: printed 12, expected 11 |
| `gambatte/tima/tc01_late_tima_tma_2_dmg08_cgb04c_outF1.gbc` | fail: printed 11, expected F1 |
| `gambatte/tima/tc01_late_tima_tma_3_dmg08_cgb04c_out11.gbc` | fail: printed F1, expected 11 |
| `gambatte/tima/tc01_late_tma_2_dmg08_cgb04c_outF1.gbc` | fail: printed 11, expected F1 |
| `gambatte/tima/tc01_tma_next_2_dmg08_cgb04c_outF1.gbc` | fail: printed F0, expected F1 |
| `gambatte/vram_m3/postread_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/vram_m3/postread_scx2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/vram_m3/postread_scx3_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/vram_m3/postread_scx5_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/vram_m3/vramw_m3start_1_dmg08_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx00_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx00_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx20_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx20_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wx_late_wy_FFto2_ly2_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_10to0_ly1_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_10to0_ly1_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_10to1_ly1_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_10to1_ly1_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_1toFF_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_2toFF_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly0_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly0_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto1_ly2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto1_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx3_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx3_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx5_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx5_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx00_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx00_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx0f_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx0f_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_early_scx03_wx0f_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx03_wx10_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx03_wx11_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx03_wx12_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_early_scx03_wx12_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_late_scx03_wx0f_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx0f_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx10_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx10_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx11_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx11_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_late_scx03_wx12_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx2_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx2_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx3_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx3_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx5_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_scx5_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_spx10_wx0f_1_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/late_disable_wx0f_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_disable_wx0f_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx2_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx3_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx3_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx5_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx5_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_wx0f_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_scx_late_disable_1_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_scx_late_disable_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_ff_07_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_ff_0f_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_scx2_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_scx3_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_scx5_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_wx03_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wx_wx0f_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_wy_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx00_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx03_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx03_scx2_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx03_scx3_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx03_scx5_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx07_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx07_scx2_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx07_scx3_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx07_scx5_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wx17_wxA5_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA5_m0irq_1_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/window/m2int_wxA5_m3stat_1_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_scx2_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/m2int_wxA6_scx3_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/m2int_wxA6_spxA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/window/m2int_wxA6_spxA7_m3stat_2_dmg08_out0_cgb04c_out3.gbc` | fail: printed 3, expected 0 |
| `gambatte/window/on_screen/weon_wx18_weoff_weon_wx80.gbc` | fail: 8380 pixels differ from reference |
| `gambatte/window/on_screen/wx17_weoff_wxA5_weon.gbc` | fail: 6784 pixels differ from reference |
| `gambatte/window/on_screen/wxA5_weoff_at_xposA5.gbc` | fail: 240 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_3.gbc` | fail: 10780 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_1.gbc` | fail: 14624 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_2.gbc` | fail: 14672 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_3.gbc` | fail: 14516 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_scx7.gbc` | fail: 10992 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_weoff_at_xposA6.gbc` | fail: 8952 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy00.gbc` | fail: 21816 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01.gbc` | fail: 21657 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_weoff_ly02.gbc` | fail: 160 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_weoff_ly02_weon_ly60.gbc` | fail: 7303 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_wxA5_ly02.gbc` | fail: 160 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_wxA7_ly02.gbc` | fail: 160 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy8F.gbc` | fail: 160 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_bgp_change.gb` | fail: 6192 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_bgp_change_sprites.gb` | fail: 4290 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_en_change.gb` | fail: 2189 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_map_change.gb` | fail: 832 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change.gb` | fail: 256 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change_variant.gb` | fail: 1444 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change.gb` | fail: 310 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change_scx.gb` | fail: 190 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_change.gb` | fail: 1020 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_win_change.gb` | fail: 1336 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple.gb` | fail: 8982 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple_wx.gb` | fail: 6020 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_map_change.gb` | fail: 754 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_obp0_change.gb` | fail: 290 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scx_high_5_bits.gb` | fail: 6426 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scx_low_3_bits.gb` | fail: 324 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scy_change.gb` | fail: 9873 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_window_timing.gb` | fail: 1215 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_window_timing_wx_0.gb` | fail: 954 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change.gb` | fail: 229 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change_sprites.gb` | fail: 10 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_5_change.gb` | fail: 638 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_6_change.gb` | fail: 13799 pixels differ from reference |

</details>
