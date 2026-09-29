| Suite | Passed | Total | |
|---|---:|---:|---|
| blargg | 43 | 43 | CPU, timing, sound and OAM-bug tests by Shay Green |
| mooneye | 94 | 94 | Mooneye Test Suite: acceptance + emulator-only (DMG-applicable) |
| dmg-acid2 | 1 | 1 | PPU rendering torture test |
| gambatte | 1595 | 1783 | Gambatte hardware-verified tests: DMG hex-result and screenshot cases |
| mealybug | 9 | 24 | Mealybug Tearoom: mid-scanline register, fetch and window effects |
| **all** | **1742** | **1945** | |

Left out because they cannot pass on hardware either:

- `blargg/oam_bug/rom_singles/7-timing_effect.gb`: its cartridge-RAM log of 19 OAM dumps outgrows the 8 KiB RAM and overwrites the test's own code in WRAM (SameBoy crashes the same way); the same test passes inside `oam_bug.gb`

<details><summary>Not passing</summary>

| ROM | Result |
|---|---|
| `gambatte/bgtiledata/bgtiledata_spx08_1.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_2.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_3.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx08_4.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_1.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_2.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_3.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtiledata/bgtiledata_spx0A_4.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_1.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_2.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_3.gbc` | fail: 128 pixels differ from reference |
| `gambatte/bgtilemap/bgtilemap_spx0A_4.gbc` | fail: 128 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_3.gb` | fail: 1 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_4.gb` | fail: 144 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_5.gb` | fail: 144 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/dmgpalette_during_m3_scx1_4.gb` | fail: 144 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_3.gb` | fail: 143 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/lycint_dmgpalette_during_m3_4.gb` | fail: 143 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_4.gb` | fail: 1 pixels differ from reference |
| `gambatte/dmgpalette_during_m3/scx3/dmgpalette_during_m3_5.gb` | fail: 144 pixels differ from reference |
| `gambatte/enable_display/enable_display_ly0_sprites_m0stat_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/enable_display/ly0_late_scx7_m3stat_scx0_2_dmg08_out87_cgb04c_out84.gbc` | fail: printed 84, expected 87 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx2_halt_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/irq_precedence/late_m0irq_vs_tima_scx3_halt_1_dmg08_cgb04c_out4.gbc` | fail: printed 2, expected 4 |
| `gambatte/ly0/lycint152_lyc153irq_late_retrigger_2_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
| `gambatte/lyc153int_m2irq/lyc153int_m2irq_ifw_1_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/lycEnable/ff41_disable_2_dmg08_out0_cgb04c_out2.gbc` | fail: printed 2, expected 0 |
| `gambatte/lycEnable/lcdoff_lycirqen_1_dmg08_cgb04c_outE2.gbc` | fail: printed E0, expected E2 |
| `gambatte/lycEnable/lcdoff_lycirqen_4_dmg08_outE2_cgb04c_outE0.gbc` | fail: printed E0, expected E2 |
| `gambatte/m0enable/lycdisable_ff45_scx3_3_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/m1/lyc143_late_m2enable_lycdisable_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2disable_lycdisable_3_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_1_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m1irq_m2enable_lyc_2_dmg08_out1_cgb04c_out3.gbc` | fail: printed 3, expected 1 |
| `gambatte/m1/m2m1irq_ifw_2_dmg08_cgb04c_out1.gbc` | fail: printed 3, expected 1 |
| `gambatte/miscmstatirq/lycstatwirq_trigger_m0_late_ly44_lyc44_08_40_4_dmg08_cgb04c_outE0.gbc` | fail: printed E2, expected E0 |
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
| `gambatte/oamdma/oamdmasrc80_halt_m2irq_read8000_dmg08_cgb04c_out81.gbc` | fail: printed 2A, expected 81 |
| `gambatte/scy/scy_during_m3_spx08_2.gbc` | fail: 112 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_1.gbc` | fail: 16 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_2.gbc` | fail: 240 pixels differ from reference |
| `gambatte/scy/scy_during_m3_spx0A_3.gbc` | fail: 16 pixels differ from reference |
| `gambatte/serial/nopx2_start_wait_read_if_1_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/serial/start_late_div_write_wait_read_if_2b_dmg08_cgb04c_outE8.gbc` | fail: printed E0, expected E8 |
| `gambatte/serial/start_late_div_write_wait_read_if_3a_dmg08_cgb04c_outE0.gbc` | fail: printed E8, expected E0 |
| `gambatte/sound/ch1_init_reset_sweep_counter_timing_nr52_1_dmg08_cgb04c_out1.gbc` | fail: printed 0, expected 1 |
| `gambatte/sound/ch2_late_reset_nr52_2b_dmg08_cgb04c_out0.gbc` | fail: printed 2, expected 0 |
| `gambatte/sprites/10spritesPrLine_10xposA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/sprites/late_sizechange2_sp00_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp01_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange2_sp02_2_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp00_4_dmg08_cgb04c_out0.gbc` | fail: printed 3, expected 0 |
| `gambatte/sprites/late_sizechange_sp01_3_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/late_sizechange_sp02_2_dmg08_cgb04c_out3.gbc` | fail: printed 0, expected 3 |
| `gambatte/sprites/sprite_late_disable_spx1A_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_disable_spx1B_1_dmg08_out0.gb` | fail: printed 3, expected 0 |
| `gambatte/sprites/sprite_late_enable_spx1A_1_dmg08_out3.gb` | fail: printed 0, expected 3 |
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
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx00_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_scx_late_wy_FFto4_ly4_wx20_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_10to1_ly1_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto0_ly0_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_scx5_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx00_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/arg/late_wy_FFto2_ly2_wx0f_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx2_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/late_reenable_scx5_2_dmg08_out3_cgb04c_out0.gbc` | fail: printed 0, expected 3 |
| `gambatte/window/m2int_wxA6_spxA7_m0irq_2_dmg08_cgb04c_out2.gbc` | fail: printed 0, expected 2 |
| `gambatte/window/on_screen/wxA6_3.gbc` | fail: 328 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_1.gbc` | fail: 6548 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_2.gbc` | fail: 6548 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_late_we_reenable_3.gbc` | fail: 14516 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_scx7.gbc` | fail: 282 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_weoff_at_xposA6.gbc` | fail: 8832 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy00.gbc` | fail: 272 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01.gbc` | fail: 424 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_weoff_ly02.gbc` | fail: 160 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_weoff_ly02_weon_ly60.gbc` | fail: 436 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_wxA5_ly02.gbc` | fail: 162 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy01_wxA7_ly02.gbc` | fail: 160 pixels differ from reference |
| `gambatte/window/on_screen/wxA6_wy8F.gbc` | fail: 160 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_en_change.gb` | fail: 403 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_bg_map_change.gb` | fail: 192 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change.gb` | fail: 60 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_en_change_variant.gb` | fail: 92 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change.gb` | fail: 15 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_obj_size_change_scx.gb` | fail: 30 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_change.gb` | fail: 192 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_tile_sel_win_change.gb` | fail: 178 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_en_change_multiple_wx.gb` | fail: 334 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_lcdc_win_map_change.gb` | fail: 122 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_scy_change.gb` | fail: 627 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_window_timing_wx_0.gb` | fail: 126 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change.gb` | fail: 229 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_4_change_sprites.gb` | fail: 10 pixels differ from reference |
| `mealybug-tearoom-tests/ppu/m3_wx_5_change.gb` | fail: 638 pixels differ from reference |

</details>
