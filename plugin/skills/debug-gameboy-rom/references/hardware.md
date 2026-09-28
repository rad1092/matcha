# DMG hardware quick reference

Sources: Pan Docs (https://gbdev.io/pandocs/), matcha's conformance suites.

## Memory map

| Range | What |
|---|---|
| $0000–$3FFF | ROM bank 0 (header at $0100–$014F; entry point $0100) |
| $4000–$7FFF | Switchable ROM bank (MBC); writes here program the mapper |
| $8000–$97FF | VRAM tile data (384 tiles × 16 bytes, 2 bits per pixel) |
| $9800–$9BFF / $9C00–$9FFF | Tile maps (32×32 tile indices) |
| $A000–$BFFF | Cartridge RAM (must be enabled: write $0A to $0000–$1FFF) |
| $C000–$DFFF | Work RAM |
| $E000–$FDFF | Echo of $C000–$DDFF |
| $FE00–$FE9F | OAM: 40 objects × (Y+16, X+8, tile, attributes) |
| $FF00–$FF7F | I/O registers |
| $FF80–$FFFE | High RAM (usable during OAM DMA) |
| $FFFF | IE (interrupt enable) |

## Interrupts

IF ($FF0F) and IE ($FFFF) bits: 0 VBlank → $40, 1 STAT → $48, 2 Timer →
$50, 3 Serial → $58, 4 Joypad → $60. Lower bit = higher priority. Dispatch
takes 5 M-cycles; `ei` takes effect after the next instruction.

## Key registers

| Addr | Name | Notes |
|---|---|---|
| $FF00 | P1/JOYP | bit 5 low = read buttons, bit 4 low = read d-pad; low nibble 0 = pressed |
| $FF01/$FF02 | SB/SC | serial data / control ($81 starts an internal-clock transfer) |
| $FF04 | DIV | upper byte of the 16-bit system counter; any write resets it |
| $FF05–$FF07 | TIMA/TMA/TAC | timer counter / reload / control (bit 2 enable, bits 0–1: 4096/262144/65536/16384 Hz) |
| $FF10–$FF26 | NRxx | sound; NR52 ($FF26) bit 7 powers the APU |
| $FF30–$FF3F | Wave RAM | 32 4-bit samples for channel 3 |
| $FF40 | LCDC | 7 LCD on · 6 window map · 5 window on · 4 tile data ($8000 unsigned / $8800 signed) · 3 BG map · 2 OBJ 8×16 · 1 OBJ on · 0 BG/window on |
| $FF41 | STAT | 6 LYC int · 5 mode-2 int · 4 mode-1 int · 3 mode-0 int · 2 LY=LYC · 1–0 mode |
| $FF42/$FF43 | SCY/SCX | background scroll |
| $FF44 | LY | current line 0–153 (144–153 = VBlank) |
| $FF45 | LYC | compare value for the STAT LYC interrupt |
| $FF46 | DMA | write $XX to copy $XX00–$XX9F to OAM (160 M-cycles; run it from HRAM) |
| $FF47–$FF49 | BGP/OBP0/OBP1 | palettes: 2 bits per colour index, shade 0 = lightest |
| $FF4A/$FF4B | WY/WX | window position (WX is screen X + 7) |

## PPU timing (per line, 456 dots = 114 M-cycles)

Mode 2 (OAM scan) 80 dots → mode 3 (drawing) 172–289 dots → mode 0 (HBlank)
for the rest. Lines 144–153 are mode 1 (VBlank). VRAM is locked during mode
3 and OAM during modes 2–3: CPU writes then are silently dropped, reads return
$FF. A frame is 70,224 dots ≈ 16.74 ms.

## Cartridge header

| Addr | Field |
|---|---|
| $0134–$0143 | Title |
| $0143 | CGB flag ($80 enhanced, $C0 CGB only) |
| $0147 | Cartridge type ($00 ROM, $01–$03 MBC1, $05–$06 MBC2, $0F–$13 MBC3, $19–$1E MBC5) |
| $0148 | ROM size (32 KiB << n) |
| $0149 | RAM size (0, –, 8, 32, 128, 64 KiB) |
| $014D | Header checksum |

## Test-ROM result protocols

- Blargg: text over serial; newer tests also write status at $A000 ($80 while
  running, $00 pass, else failure code), signature DE B0 61 at $A001, then a
  zero-terminated string from $A004.
- Mooneye, dmg-acid2, Mealybug: finish by executing `ld b, b` ($40). Mooneye
  pass = B C D E H L = 3 5 8 13 21 34; fail = all $42.
