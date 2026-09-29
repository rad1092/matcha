//! Cartridge: header parsing and memory bank controllers (MBC1/1M/2/3/5).

use crate::state::{StateError, StateReader, StateWriter};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

/// Mapper chip family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapperKind {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

impl MapperKind {
    /// 16 KiB banks the mapper can select (its bank register width).
    fn max_rom_banks(self) -> usize {
        match self {
            Self::RomOnly => 2,
            Self::Mbc1 | Self::Mbc3 => 128,
            Self::Mbc2 => 16,
            Self::Mbc5 => 512,
        }
    }
}

/// Everything the header at 0x0100–0x014F says about the cartridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub title: String,
    /// 0x0143: 0x80 = CGB-enhanced, 0xC0 = CGB-only.
    pub cgb_flag: u8,
    /// 0x0146: 0x03 = SGB functions.
    pub sgb_flag: u8,
    /// 0x0147 raw cartridge type.
    pub cart_type: u8,
    pub rom_size: usize,
    pub ram_size: usize,
    pub old_licensee: u8,
    pub version: u8,
    pub header_checksum: u8,
    pub header_checksum_ok: bool,
    pub global_checksum: u16,
}

impl Header {
    pub fn cgb_only(&self) -> bool {
        self.cgb_flag == 0xC0
    }
    pub fn cgb_enhanced(&self) -> bool {
        self.cgb_flag & 0x80 != 0
    }
    /// Human-readable name for the cartridge type byte.
    pub fn cart_type_name(&self) -> &'static str {
        cart_type_name(self.cart_type)
    }
}

pub fn cart_type_name(t: u8) -> &'static str {
    match t {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+BATTERY",
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        0x1C => "MBC5+RUMBLE",
        0x1D => "MBC5+RUMBLE+RAM",
        0x1E => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xFC => "POCKET CAMERA",
        0xFD => "BANDAI TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+BATTERY",
        _ => "UNKNOWN",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CartridgeError {
    /// Smaller than the 0x150-byte header.
    TooSmall(usize),
    /// A mapper this core does not implement (yet).
    UnsupportedMapper { cart_type: u8, name: &'static str },
}

impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall(n) => write!(f, "ROM is only {n} bytes; a Game Boy ROM is at least 336"),
            Self::UnsupportedMapper { cart_type, name } => {
                write!(f, "unsupported cartridge type {cart_type:#04x} ({name})")
            }
        }
    }
}

/// Parses the header without validating the mapper.
pub fn parse_header(rom: &[u8]) -> Result<Header, CartridgeError> {
    if rom.len() < 0x150 {
        return Err(CartridgeError::TooSmall(rom.len()));
    }
    let cgb_flag = rom[0x143];
    // Newer carts use 0x13F..0x143 for the manufacturer code / CGB flag.
    let title_end = if cgb_flag & 0x80 != 0 { 0x143 } else { 0x144 };
    let title: String = rom[0x134..title_end]
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| if (0x20..0x7F).contains(&b) { b as char } else { '?' })
        .collect();
    let rom_size = match rom[0x148] {
        n @ 0x00..=0x08 => 0x8000usize << n,
        0x52 => 72 * 0x4000,
        0x53 => 80 * 0x4000,
        0x54 => 96 * 0x4000,
        _ => rom.len(),
    };
    let ram_size = match rom[0x149] {
        0x01 => 0x800,
        0x02 => 0x2000,
        0x03 => 0x8000,
        0x04 => 0x20000,
        0x05 => 0x10000,
        _ => 0,
    };
    let computed = rom[0x134..=0x14C].iter().fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));
    Ok(Header {
        title,
        cgb_flag,
        sgb_flag: rom[0x146],
        cart_type: rom[0x147],
        rom_size,
        ram_size,
        old_licensee: rom[0x14B],
        version: rom[0x14C],
        header_checksum: rom[0x14D],
        header_checksum_ok: computed == rom[0x14D],
        global_checksum: u16::from_be_bytes([rom[0x14E], rom[0x14F]]),
    })
}

fn mapper_for(cart_type: u8) -> Result<(MapperKind, bool, bool), CartridgeError> {
    // (kind, has_battery, has_rtc)
    Ok(match cart_type {
        0x00 | 0x08 => (MapperKind::RomOnly, false, false),
        0x09 => (MapperKind::RomOnly, true, false),
        0x01 | 0x02 => (MapperKind::Mbc1, false, false),
        0x03 => (MapperKind::Mbc1, true, false),
        0x05 => (MapperKind::Mbc2, false, false),
        0x06 => (MapperKind::Mbc2, true, false),
        0x0F | 0x10 => (MapperKind::Mbc3, true, true),
        0x11 | 0x12 => (MapperKind::Mbc3, false, false),
        0x13 => (MapperKind::Mbc3, true, false),
        0x19 | 0x1A | 0x1C | 0x1D => (MapperKind::Mbc5, false, false),
        0x1B | 0x1E => (MapperKind::Mbc5, true, false),
        t => return Err(CartridgeError::UnsupportedMapper { cart_type: t, name: cart_type_name(t) }),
    })
}

/// MBC3 real-time clock. Advances with *emulated* time so emulation stays
/// deterministic; hosts may call [`Cartridge::rtc_advance_seconds`] to catch
/// up with wall-clock time (e.g. after loading a save).
#[derive(Clone, Debug, Default)]
struct Rtc {
    /// Live registers: seconds, minutes, hours, day low, day high/flags.
    regs: [u8; 5],
    latched: [u8; 5],
    /// Sub-second progress in T-cycles.
    subsecond: u32,
    latch_armed: bool,
}

const RTC_HZ: u32 = 4_194_304;

impl Rtc {
    fn halted(&self) -> bool {
        self.regs[4] & 0x40 != 0
    }

    fn tick(&mut self, t_cycles: u32) {
        if self.halted() {
            return;
        }
        self.subsecond += t_cycles;
        while self.subsecond >= RTC_HZ {
            self.subsecond -= RTC_HZ;
            self.increment_second();
        }
    }

    /// Counters are 6/6/5/9 bits wide; values written out of range count up
    /// to the register width and wrap to 0 *without* carrying (rtc3test).
    fn increment_second(&mut self) {
        let r = &mut self.regs;
        r[0] = (r[0] + 1) & 0x3F;
        if r[0] != 60 {
            return;
        }
        r[0] = 0;
        r[1] = (r[1] + 1) & 0x3F;
        if r[1] != 60 {
            return;
        }
        r[1] = 0;
        r[2] = (r[2] + 1) & 0x1F;
        if r[2] != 24 {
            return;
        }
        r[2] = 0;
        let day = (u16::from(r[4] & 1) << 8 | u16::from(r[3])) + 1;
        r[3] = day as u8;
        r[4] = (r[4] & 0xFE) | ((day >> 8) as u8 & 1);
        if day > 0x1FF {
            r[3] = 0;
            r[4] = (r[4] & 0xFE) | 0x80; // day counter carry
        }
    }

    fn write(&mut self, reg: u8, value: u8) {
        match reg {
            0x08 => {
                self.regs[0] = value & 0x3F;
                self.subsecond = 0;
            }
            0x09 => self.regs[1] = value & 0x3F,
            0x0A => self.regs[2] = value & 0x1F,
            0x0B => self.regs[3] = value,
            0x0C => self.regs[4] = value & 0xC1,
            _ => {}
        }
    }

    fn read(&self, reg: u8) -> u8 {
        match reg {
            0x08 => self.latched[0] | 0xC0,
            0x09 => self.latched[1] | 0xC0,
            0x0A => self.latched[2] | 0xE0,
            0x0B => self.latched[3],
            0x0C => self.latched[4] | 0x3E,
            _ => 0xFF,
        }
    }
}

/// A loaded cartridge: ROM, external RAM, and mapper state.
#[derive(Clone, Debug)]
pub struct Cartridge {
    header: Header,
    kind: MapperKind,
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_battery: bool,
    rom_bank_mask: usize,
    ram_bank_count: usize,
    /// MBC1 wired as a 1 MiB multi-game compilation ("MBC1M").
    multicart: bool,
    // Mapper registers (meaning depends on `kind`).
    ram_enabled: bool,
    rom_bank: u16,
    bank2: u8,
    mode: bool,
    rtc: Option<Rtc>,
    /// Set whenever battery-backed RAM is written; hosts clear it after saving.
    ram_dirty: bool,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        let header = parse_header(&rom)?;
        let (kind, has_battery, has_rtc) = mapper_for(header.cart_type)?;
        let mut rom = rom;
        // Bytes past the mapper's highest bank can never be read; dropping
        // them bounds memory for oversized files (8 MiB at most, for MBC5).
        rom.truncate(kind.max_rom_banks() * 0x4000);
        // Pad to a power-of-two number of 16 KiB banks so bank masking is exact.
        let banks = rom.len().div_ceil(0x4000).max(2).next_power_of_two();
        rom.resize(banks * 0x4000, 0xFF);
        let ram_len = match kind {
            MapperKind::Mbc2 => 512,
            _ => header.ram_size,
        };
        let multicart = kind == MapperKind::Mbc1 && detect_mbc1m(&rom);
        Ok(Self {
            kind,
            ram: vec![0; ram_len],
            has_battery,
            rom_bank_mask: banks - 1,
            ram_bank_count: (ram_len / 0x2000).max(1),
            multicart,
            ram_enabled: false,
            rom_bank: 1,
            bank2: 0,
            mode: false,
            rtc: has_rtc.then(Rtc::default),
            ram_dirty: false,
            header,
            rom,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn mapper(&self) -> MapperKind {
        self.kind
    }
    pub fn rom(&self) -> &[u8] {
        &self.rom
    }
    pub fn has_battery(&self) -> bool {
        self.has_battery
    }
    pub fn has_rtc(&self) -> bool {
        self.rtc.is_some()
    }
    pub fn is_multicart(&self) -> bool {
        self.multicart
    }

    /// Stable identity used to bind save states to this ROM.
    pub fn rom_id(&self) -> u32 {
        // FNV-1a over the header region plus global checksum: cheap and stable.
        let mut h: u32 = 0x811C_9DC5;
        for &b in &self.rom[0x100..0x150] {
            h ^= u32::from(b);
            h = h.wrapping_mul(0x0100_0193);
        }
        h
    }

    pub fn reset(&mut self) {
        self.ram_enabled = false;
        self.rom_bank = 1;
        self.bank2 = 0;
        self.mode = false;
        if let Some(rtc) = &mut self.rtc {
            rtc.latch_armed = false;
        }
    }

    /// Battery-backed RAM contents, if the cartridge has a battery.
    pub fn battery_ram(&self) -> Option<&[u8]> {
        (self.has_battery && !self.ram.is_empty()).then_some(&self.ram[..])
    }

    pub fn load_battery_ram(&mut self, data: &[u8]) {
        let n = data.len().min(self.ram.len());
        self.ram[..n].copy_from_slice(&data[..n]);
        self.ram_dirty = false;
    }

    /// True when battery RAM changed since the last call (clears the flag).
    pub fn take_ram_dirty(&mut self) -> bool {
        core::mem::take(&mut self.ram_dirty)
    }

    /// Advances the RTC by `t_cycles` of emulated time.
    #[inline]
    pub(crate) fn tick_rtc(&mut self, t_cycles: u32) {
        if let Some(rtc) = &mut self.rtc {
            rtc.tick(t_cycles);
        }
    }

    /// Moves the RTC forward by wall-clock seconds (e.g. time spent powered off).
    pub fn rtc_advance_seconds(&mut self, seconds: u64) {
        if let Some(rtc) = &mut self.rtc {
            if rtc.halted() {
                return;
            }
            // Cap to one full 512-day cycle; anything longer only sets carry.
            let capped = seconds.min(512 * 86_400 + 1);
            for _ in 0..capped {
                rtc.increment_second();
            }
        }
    }

    #[inline]
    fn rom_byte(&self, bank: usize, addr: u16) -> u8 {
        self.rom[((bank & self.rom_bank_mask) << 14) | (addr as usize & 0x3FFF)]
    }

    #[inline]
    pub fn read_rom(&self, addr: u16) -> u8 {
        match self.kind {
            MapperKind::RomOnly => self.rom_byte(usize::from(addr >> 14), addr),
            MapperKind::Mbc1 => {
                if addr < 0x4000 {
                    let bank = if self.mode { self.mbc1_upper_bits() } else { 0 };
                    self.rom_byte(bank, addr)
                } else {
                    let low = usize::from(self.rom_bank & 0x1F).max(1);
                    let low = if self.multicart { low & 0x0F } else { low };
                    self.rom_byte(self.mbc1_upper_bits() | low, addr)
                }
            }
            MapperKind::Mbc2 | MapperKind::Mbc3 | MapperKind::Mbc5 => {
                if addr < 0x4000 {
                    self.rom_byte(0, addr)
                } else {
                    self.rom_byte(usize::from(self.rom_bank), addr)
                }
            }
        }
    }

    /// MBC1: the 2-bit register shifted into ROM bank bits 5–6 (4–5 on MBC1M).
    #[inline]
    fn mbc1_upper_bits(&self) -> usize {
        usize::from(self.bank2) << if self.multicart { 4 } else { 5 }
    }

    pub fn write_rom(&mut self, addr: u16, value: u8) {
        match self.kind {
            MapperKind::RomOnly => {}
            MapperKind::Mbc1 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
                0x2000..=0x3FFF => self.rom_bank = u16::from(value & 0x1F),
                0x4000..=0x5FFF => self.bank2 = value & 0x03,
                _ => self.mode = value & 1 != 0,
            },
            MapperKind::Mbc2 => {
                if addr < 0x4000 {
                    if addr & 0x0100 == 0 {
                        self.ram_enabled = value & 0x0F == 0x0A;
                    } else {
                        self.rom_bank = u16::from(value & 0x0F).max(1);
                    }
                }
            }
            MapperKind::Mbc3 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
                0x2000..=0x3FFF => self.rom_bank = u16::from(value & 0x7F).max(1),
                0x4000..=0x5FFF => self.bank2 = value & 0x0F,
                _ => {
                    if let Some(rtc) = &mut self.rtc {
                        if value == 0x01 && rtc.latch_armed {
                            rtc.latched = rtc.regs;
                        }
                        rtc.latch_armed = value == 0x00;
                    }
                }
            },
            MapperKind::Mbc5 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = value & 0x0F == 0x0A,
                0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | u16::from(value),
                0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | (u16::from(value & 1) << 8),
                0x4000..=0x5FFF => self.bank2 = value & 0x0F,
                _ => {}
            },
        }
    }

    /// Offset into `ram` for an A000–BFFF access, or None if unmapped.
    #[inline]
    fn ram_offset(&self, addr: u16) -> Option<usize> {
        if !self.ram_enabled && self.kind != MapperKind::RomOnly {
            return None;
        }
        if self.ram.is_empty() {
            return None;
        }
        let bank = match self.kind {
            MapperKind::RomOnly => 0,
            MapperKind::Mbc1 => {
                if self.mode {
                    usize::from(self.bank2)
                } else {
                    0
                }
            }
            MapperKind::Mbc2 => return Some(usize::from(addr & 0x01FF)),
            MapperKind::Mbc3 => usize::from(self.bank2 & 0x07),
            MapperKind::Mbc5 => usize::from(self.bank2 & 0x0F),
        };
        let bank = bank % self.ram_bank_count;
        Some(((bank << 13) | (addr as usize & 0x1FFF)) % self.ram.len())
    }

    #[inline]
    fn rtc_selected(&self) -> Option<u8> {
        match (&self.rtc, self.kind) {
            (Some(_), MapperKind::Mbc3) if self.ram_enabled && (0x08..=0x0C).contains(&self.bank2) => Some(self.bank2),
            _ => None,
        }
    }

    #[inline]
    pub fn read_ram(&self, addr: u16) -> u8 {
        if let Some(reg) = self.rtc_selected() {
            return self.rtc.as_ref().map_or(0xFF, |rtc| rtc.read(reg));
        }
        match self.ram_offset(addr) {
            Some(off) if self.kind == MapperKind::Mbc2 => self.ram[off] | 0xF0,
            Some(off) => self.ram[off],
            None => 0xFF,
        }
    }

    #[inline]
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        if let Some(reg) = self.rtc_selected() {
            if let Some(rtc) = &mut self.rtc {
                rtc.write(reg, value);
            }
            return;
        }
        if let Some(off) = self.ram_offset(addr) {
            let value = if self.kind == MapperKind::Mbc2 { value & 0x0F } else { value };
            if self.ram[off] != value {
                self.ram[off] = value;
                self.ram_dirty |= self.has_battery;
            }
        }
    }

    /// Current ROM bank mapped at 0x4000 (for debuggers/disassembly).
    pub fn current_rom_bank(&self) -> usize {
        match self.kind {
            MapperKind::RomOnly => 1,
            MapperKind::Mbc1 => {
                let low = usize::from(self.rom_bank & 0x1F).max(1);
                let low = if self.multicart { low & 0x0F } else { low };
                (self.mbc1_upper_bits() | low) & self.rom_bank_mask
            }
            _ => usize::from(self.rom_bank) & self.rom_bank_mask,
        }
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.bytes(&self.ram);
        w.bool(self.ram_enabled);
        w.u16(self.rom_bank);
        w.u8(self.bank2);
        w.bool(self.mode);
        w.bool(self.rtc.is_some());
        if let Some(rtc) = &self.rtc {
            w.u8s(&rtc.regs);
            w.u8s(&rtc.latched);
            w.u32(rtc.subsecond);
            w.bool(rtc.latch_armed);
        }
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.bytes_exact(&mut self.ram)?;
        self.ram_enabled = r.bool()?;
        self.rom_bank = r.u16()?;
        self.bank2 = r.u8()?;
        self.mode = r.bool()?;
        let has_rtc = r.bool()?;
        if has_rtc != self.rtc.is_some() {
            return Err(StateError::Corrupt("rtc presence"));
        }
        if let Some(rtc) = &mut self.rtc {
            r.u8s(&mut rtc.regs)?;
            r.u8s(&mut rtc.latched)?;
            rtc.subsecond = r.u32()?;
            rtc.latch_armed = r.bool()?;
            // The registers can only hold what `Rtc::write` lets through.
            let [s, m, h, _, flags] = rtc.regs;
            if s > 0x3F || m > 0x3F || h > 0x1F || flags & !0xC1 != 0 || rtc.subsecond >= RTC_HZ {
                return Err(StateError::Corrupt("rtc"));
            }
        }
        Ok(())
    }
}

/// MBC1M carts are 1 MiB and repeat the Nintendo logo at the start of bank 0x10.
fn detect_mbc1m(rom: &[u8]) -> bool {
    const LOGO_START: usize = 0x104;
    if rom.len() != 0x10_0000 {
        return false;
    }
    let logo = &rom[LOGO_START..LOGO_START + 0x30];
    let second = 0x10 * 0x4000 + LOGO_START;
    rom.get(second..second + 0x30) == Some(logo)
}
