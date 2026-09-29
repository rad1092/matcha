//! The system bus: memory map, I/O registers, OAM DMA, and the per-M-cycle
//! clock that drives every peripheral.
//!
//! Every CPU access performs the access and then advances the whole machine
//! by one M-cycle ("access, then tick" — ADR-0001).

use crate::Model;
use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::CpuBus;
use crate::joypad::{Buttons, Joypad};
use crate::ppu::{Mode, Ppu};
use crate::profile::{Profile, Region};
use crate::serial::Serial;
use crate::state::{StateError, StateReader, StateWriter};
use crate::timer::Timer;
use alloc::boxed::Box;

pub mod irq {
    pub const VBLANK: u8 = 0x01;
    pub const STAT: u8 = 0x02;
    pub const TIMER: u8 = 0x04;
    pub const SERIAL: u8 = 0x08;
    pub const JOYPAD: u8 = 0x10;
}

#[derive(Clone, Debug, Default)]
struct Dma {
    /// Last value written to FF46.
    reg: u8,
    /// Source page of the running transfer.
    source: u8,
    /// Bytes copied so far (0..160) while `active`.
    index: u8,
    active: bool,
    /// M-cycles until a requested transfer starts (0 = none pending).
    start_delay: u8,
    pending_source: u8,
    /// A CPU write that collided with the transfer this M-cycle; it takes
    /// effect after the DMA's own access (see `dma_conflict`). Never outlives
    /// the M-cycle, so it is not part of save states.
    collided: Option<Collision>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Collision {
    /// Lands on the address the DMA is reading (ROM or VRAM source).
    Redirect(u16, u8),
    /// Lost; ANDs into the byte the DMA writes to OAM (RAM source).
    MaskOam(u8),
}

/// CGB VRAM DMA, in 16-byte blocks. Each byte takes two base-clock dots,
/// independently of CPU speed (Pan Docs, "CGB Registers / Transfer timings").
#[derive(Clone, Debug, Default)]
struct VramDma {
    source: u16,
    destination: u16,
    blocks: u8,
    active: bool,
    hblank: bool,
    block_pending: bool,
}

/// A memory watchpoint hit, reported after the step that caused it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatchHit {
    pub addr: u16,
    pub value: u8,
    pub write: bool,
}

/// 64K-bit set of addresses.
#[derive(Clone, Debug)]
pub(crate) struct AddrSet {
    bits: Box<[u64; 1024]>,
    count: usize,
}

impl AddrSet {
    pub(crate) fn new() -> Self {
        Self { bits: Box::new([0; 1024]), count: 0 }
    }
    #[inline]
    pub(crate) fn contains(&self, addr: u16) -> bool {
        self.bits[usize::from(addr >> 6)] & (1 << (addr & 63)) != 0
    }
    pub(crate) fn insert(&mut self, addr: u16) -> bool {
        let (w, b) = (usize::from(addr >> 6), 1u64 << (addr & 63));
        let fresh = self.bits[w] & b == 0;
        self.bits[w] |= b;
        self.count += usize::from(fresh);
        fresh
    }
    pub(crate) fn remove(&mut self, addr: u16) -> bool {
        let (w, b) = (usize::from(addr >> 6), 1u64 << (addr & 63));
        let present = self.bits[w] & b != 0;
        self.bits[w] &= !b;
        self.count -= usize::from(present);
        present
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        (0..=u16::MAX).filter(|&a| self.contains(a))
    }
}

pub struct SystemBus {
    model: Model,
    pub(crate) cart: Cartridge,
    pub(crate) ppu: Ppu,
    pub(crate) apu: Apu,
    pub(crate) timer: Timer,
    pub(crate) joypad: Joypad,
    pub(crate) serial: Serial,
    wram: Box<[u8; 0x8000]>,
    wram_bank: u8,
    double_speed: bool,
    speed_armed: bool,
    base_clock_ticks: u64,
    /// Base dots accumulated toward the APU's four-dot tick.
    apu_phase: u8,
    vram_dma: VramDma,
    infrared: u8,
    undocumented: [u8; 4],
    /// Derived from the CPU at every step; never survives a CPU transition.
    cpu_halted: bool,
    hram: [u8; 0x7F],
    ie: u8,
    if_: u8,
    /// IF bits raised after the HALT sampling point in this M-cycle.
    if_deferred: u8,
    /// Peripheral edges in the final two dots, including reasserted bits.
    /// Interrupt-entry acknowledge is at the middle of its last M-cycle.
    if_reasserted_late: u8,
    dma: Dma,
    boot_rom: Option<Box<[u8; 0x100]>>,
    boot_rom_mapped: bool,
    /// M-cycles since power-on.
    pub(crate) cycles: u64,
    /// A halted CPU may idle in one batch until `cycles` reaches this.
    pub(crate) halt_yield_at: u64,
    pub(crate) profile: Option<Box<Profile>>,
    pub(crate) read_watch: Option<AddrSet>,
    pub(crate) write_watch: Option<AddrSet>,
    pub(crate) watch_hit: Option<WatchHit>,
}

impl core::fmt::Debug for SystemBus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemBus")
            .field("cycles", &self.cycles)
            .field("ie", &self.ie)
            .field("if", &self.if_)
            .field("ppu", &self.ppu)
            .finish_non_exhaustive()
    }
}

impl SystemBus {
    pub fn new(cart: Cartridge, boot_rom: Option<Box<[u8; 0x100]>>) -> Self {
        Self::new_with_model(cart, boot_rom, Model::Dmg)
    }

    pub fn new_with_model(cart: Cartridge, boot_rom: Option<Box<[u8; 0x100]>>, model: Model) -> Self {
        let booting = boot_rom.is_some();
        let ppu = if booting {
            Ppu::power_on_with_model(model)
        } else {
            let mut ppu = Ppu::new_with_model(model);
            ppu.load_boot_logo(&cart.rom()[0x104..0x134]);
            ppu
        };
        Self {
            model,
            cart,
            ppu,
            apu: if booting { Apu::power_on_with_model(model) } else { Apu::new_with_model(model) },
            timer: if booting { Timer::power_on_with_model(model) } else { Timer::new_with_model(model) },
            joypad: Joypad::new(),
            serial: Serial::new_with_model(model),
            wram: Box::new([0; 0x8000]),
            wram_bank: if model == Model::Cgb { 0 } else { 1 },
            double_speed: false,
            speed_armed: false,
            base_clock_ticks: 0,
            apu_phase: 0,
            vram_dma: VramDma::default(),
            infrared: 0,
            undocumented: [0; 4],
            cpu_halted: false,
            hram: [0; 0x7F],
            ie: 0,
            if_: if booting { 0 } else { 0x01 },
            if_deferred: 0,
            if_reasserted_late: 0,
            dma: Dma { reg: if model == Model::Cgb { 0 } else { 0xFF }, ..Dma::default() },
            boot_rom_mapped: booting,
            boot_rom,
            cycles: 0,
            halt_yield_at: 0,
            profile: None,
            read_watch: None,
            write_watch: None,
            watch_hit: None,
        }
    }

    /// Fills WRAM, HRAM, OAM and wave RAM with the junk a DMG-B powers up
    /// with: random bytes biased toward 1s or 0s depending on the region and
    /// address bits (SameBoy's measurements, `reset_ram`). Deterministic in
    /// `seed` (SplitMix64).
    pub(crate) fn fill_power_on_noise(&mut self, seed: u64) {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 56) as u8
        };
        let wram_len = if self.model == Model::Cgb { 0x8000 } else { 0x2000 };
        for (i, b) in self.wram[..wram_len].iter_mut().enumerate() {
            let r = next();
            *b = if i & 0x100 != 0 { r & next() } else { r | next() };
        }
        for (i, b) in self.hram.iter_mut().enumerate() {
            *b = if i & 1 != 0 { next() | next() | next() } else { next() & next() & next() };
        }
        for (i, b) in self.ppu.oam.iter_mut().enumerate() {
            *b = if i & 2 != 0 { next() & next() & next() } else { next() | next() | next() };
        }
        for (i, b) in self.apu.wave_ram_mut().iter_mut().enumerate() {
            *b = if i & 1 != 0 { next() & next() & next() } else { next() | next() | next() };
        }
    }

    // --- clock ---------------------------------------------------------------

    /// Advances every peripheral by one M-cycle.
    #[inline]
    fn tick(&mut self) {
        self.tick_with_if_write(false);
    }

    #[inline]
    fn tick_with_if_write(&mut self, cpu_if_write: bool) {
        self.tick_peripherals(cpu_if_write, true);
        self.run_vram_dma();
    }

    fn tick_peripherals(&mut self, cpu_if_write: bool, system_clock_running: bool) {
        self.cycles += 1;
        let dots = if self.double_speed { 2 } else { 4 };
        self.base_clock_ticks += u64::from(dots);
        self.if_deferred = 0;
        self.if_reasserted_late = 0;
        let t = if system_clock_running {
            self.timer.tick_with_speed(self.double_speed)
        } else {
            crate::timer::TimerEvents::default()
        };
        if t.interrupt {
            // A running CPU sees this edge immediately; DMG HALT already
            // sampled halfway through the cycle, before the reload edge.
            self.if_deferred |= irq::TIMER & !self.if_;
            self.if_ |= irq::TIMER;
            // DIV advances in complete M-cycles; overflow reload and serial
            // edges land at the final dot, after the ISR acknowledge point.
            self.if_reasserted_late |= irq::TIMER;
        }
        if t.div_apu {
            self.apu.frame_sequencer();
        }
        let serial_edge = if self.serial.fast_clock() { t.serial_fast_clock } else { t.serial_clock };
        if serial_edge && self.serial.clock() {
            self.if_deferred |= irq::SERIAL & !self.if_;
            self.if_ |= irq::SERIAL;
            self.if_reasserted_late |= irq::SERIAL;
        }
        let previous_mode = self.ppu.mode();
        let p = self.ppu.tick_dots(dots);
        self.if_reasserted_late |= p.late;
        self.if_deferred |= p.late & !self.if_;
        // IF is driven by the CPU one dot later than ordinary I/O writes
        // (SameBoy's GB_CONFLICT_WRITE_CPU). A PPU edge on that first dot
        // loses to the write; subsequent edges can set the written bit again.
        self.if_ |= if cpu_if_write { p.after_first } else { p.now | p.late };
        self.apu_phase += dots;
        if self.apu_phase >= 4 {
            self.apu_phase -= 4;
            self.apu.tick();
        }
        if system_clock_running {
            self.tick_dma();
        }
        self.cart.tick_rtc(u32::from(dots));
        if self.vram_dma.active
            && self.vram_dma.hblank
            && system_clock_running
            && !self.cpu_halted
            && previous_mode != Mode::HBlank
            && self.ppu.mode() == Mode::HBlank
            && self.ppu.position().0 < 144
        {
            self.vram_dma.block_pending = true;
        }
    }

    /// Hold the CPU while other clocks run. A block always takes 32 LCD
    /// dots: eight normal M-cycles or sixteen double-speed M-cycles.
    fn run_vram_dma(&mut self) {
        if self.cpu_halted && self.vram_dma.hblank {
            return;
        }
        if self.vram_dma.active && (!self.vram_dma.hblank || self.vram_dma.block_pending) {
            // SameBoy GB_hdma_run has a leading transfer phase (and a
            // trailing half-cycle at normal speed), totaling one CPU M-cycle.
            self.tick_peripherals(false, true);
        }
        while self.vram_dma.active && (!self.vram_dma.hblank || self.vram_dma.block_pending) {
            self.vram_dma.block_pending = false;
            let bytes_per_cycle = if self.double_speed { 1 } else { 2 };
            for _ in 0..16 / bytes_per_cycle {
                for _ in 0..bytes_per_cycle {
                    let source = self.vram_dma.source;
                    let value = match source {
                        0x0000..=0x7FFF => self.cart.read_rom(source),
                        0xA000..=0xBFFF => self.cart.read_ram(source),
                        0xC000..=0xDFFF => self.wram[self.wram_offset(source)],
                        _ => 0xFF, // VRAM and the upper address bus are not DMA sources.
                    };
                    self.ppu.write_vram(0x8000 | (self.vram_dma.destination & 0x1FFF), value);
                    self.vram_dma.source = source.wrapping_add(1);
                    self.vram_dma.destination = self.vram_dma.destination.wrapping_add(1);
                }
                self.tick_peripherals(false, true);
            }
            self.vram_dma.blocks -= 1;
            if self.vram_dma.blocks == 0 || self.vram_dma.destination == 0 {
                self.vram_dma.active = false;
                self.vram_dma.hblank = false;
            }
            if self.vram_dma.hblank {
                break;
            }
        }
    }

    #[inline]
    fn tick_dma(&mut self) {
        let collided = self.dma.collided.take();
        if self.dma.active {
            let src = u16::from(self.dma.source) << 8 | u16::from(self.dma.index);
            let mut v = self.dma_source_read(src);
            if let Some(Collision::MaskOam(mask)) = collided {
                v &= mask;
            }
            self.ppu.oam[usize::from(self.dma.index)] = v;
            self.dma.index += 1;
            if self.dma.index == 160 {
                self.dma.active = false;
            }
        }
        if let Some(Collision::Redirect(addr, value)) = collided {
            self.write_mem(addr, value);
        }
        if self.dma.start_delay > 0 {
            self.dma.start_delay -= 1;
            if self.dma.start_delay == 0 {
                self.dma.active = true;
                self.dma.index = 0;
                self.dma.source = self.dma.pending_source;
            }
        }
    }

    /// OAM DMA drives the bus it reads from — the main bus (ROM, cartridge
    /// RAM, WRAM) or the VRAM bus — so a CPU access to that bus collides with
    /// it: the CPU sees the byte the DMA just read, and a write goes astray.
    /// This is why games run their DMA routine from HRAM. Returns the address
    /// the DMA reads in this M-cycle if `addr` collides with it (DMG rules as
    /// in SameBoy, timed by Gambatte's hardware-verified oamdma tests).
    fn dma_conflict(&self, addr: u16) -> Option<u16> {
        if addr >= 0xFE00 || !self.dma.active {
            return None;
        }
        let base = u16::from(self.dma.source) << 8;
        let current = base | u16::from(self.dma.index);
        // The address the DMA reads next is served normally.
        let next = current.wrapping_add(1);
        if addr == next || (next >= 0xE000 && next & !0x2000 == addr) {
            return None;
        }
        let vram_bus = |a: u16| (0x8000..0xA000).contains(&a);
        if self.model == Model::Cgb {
            // Unlike DMG, WRAM and the cartridge have independent buses.
            let bus = |a: u16| {
                if vram_bus(a) {
                    1
                } else if a >= 0xC000 {
                    2
                } else {
                    0
                }
            };
            return (bus(addr) == bus(base)).then_some(current);
        }
        (vram_bus(addr) == vram_bus(base)).then_some(current)
    }

    /// What the DMA unit sees at `addr` (it bypasses PPU access locks).
    fn dma_source_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cart.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read_vram(addr),
            0xA000..=0xBFFF => self.cart.read_ram(addr),
            _ => self.wram[self.wram_offset(addr)],
        }
    }

    pub fn dma_active(&self) -> bool {
        self.dma.active
    }

    fn write_vram_dma(&mut self, addr: u16, value: u8) {
        let d = &mut self.vram_dma;
        match addr {
            0xFF51 => d.source = (d.source & 0x00F0) | u16::from(value) << 8,
            0xFF52 => d.source = (d.source & 0xFF00) | u16::from(value & 0xF0),
            // Only 13 address bits reach VRAM, but the internal counter is
            // 16-bit: it wraps the VRAM address range before a full overflow ends
            // the transfer (Gambatte dma_dst_wrap/dma_src_wrap).
            0xFF53 => d.destination = (d.destination & 0x00F0) | u16::from(value) << 8,
            0xFF54 => d.destination = (d.destination & 0xFF00) | u16::from(value & 0xF0),
            _ if d.active && d.hblank && value & 0x80 == 0 => {
                // The length latch is written even on cancellation. Verified
                // by SameSuite dma/hdma_lcd_off and hdma_mode0 (0 -> 0x80).
                d.blocks = (value & 0x7F) + 1;
                d.active = false;
                d.hblank = false;
                d.block_pending = false;
            }
            _ => {
                d.blocks = (value & 0x7F) + 1;
                d.hblank = value & 0x80 != 0;
                d.active = true;
                // An initial HBlank block may start immediately, including
                // LCD-off mode 0. Subsequent blocks require a new HBlank.
                d.block_pending = d.hblank && self.ppu.mode() == Mode::HBlank && !self.cpu_halted;
            }
        }
    }

    pub fn model(&self) -> Model {
        self.model
    }

    pub fn double_speed(&self) -> bool {
        self.double_speed
    }

    /// Elapsed 4 MHz clock dots, including DMA and speed-switch CPU stalls.
    pub fn base_clock_ticks(&self) -> u64 {
        self.base_clock_ticks
    }

    // --- memory map ------------------------------------------------------------

    fn wram_offset(&self, addr: u16) -> usize {
        let offset = usize::from(addr & 0x0FFF);
        if addr & 0x1000 == 0 { offset } else { usize::from(self.wram_bank.max(1)) * 0x1000 + offset }
    }

    /// A CPU read (with PPU/DMA access restrictions), no clocking.
    fn read_mem(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF if self.boot_rom_mapped => self.boot_rom.as_ref().map_or(0xFF, |b| b[usize::from(addr)]),
            0x0000..=0x7FFF => self.cart.read_rom(addr),
            0x8000..=0x9FFF => {
                if self.ppu.vram_readable() {
                    self.ppu.read_vram(addr)
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => self.cart.read_ram(addr),
            0xC000..=0xFDFF => self.wram[self.wram_offset(addr)],
            0xFE00..=0xFE9F => {
                if self.ppu.oam_readable() && !self.dma.active {
                    self.ppu.oam[usize::from(addr - 0xFE00)]
                } else {
                    0xFF
                }
            }
            0xFEA0..=0xFEFF => {
                if self.ppu.oam_readable() && !self.dma.active {
                    0x00
                } else {
                    0xFF
                }
            }
            0xFF00..=0xFF7F => self.read_io(addr),
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)],
            0xFFFF => self.ie,
        }
    }

    fn read_io(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => self.joypad.read(),
            0xFF01 | 0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.if_ | 0xE0,
            0xFF10..=0xFF3F => self.apu.read(addr),
            0xFF46 => self.dma.reg,
            0xFF40..=0xFF4B => self.ppu.read_register(addr),
            0xFF4D if self.model == Model::Cgb => 0x7E | u8::from(self.double_speed) << 7 | u8::from(self.speed_armed),
            0xFF4F | 0xFF68..=0xFF6C if self.model == Model::Cgb => self.ppu.read_register(addr),
            0xFF55 if self.model == Model::Cgb => {
                u8::from(!self.vram_dma.active) << 7 | self.vram_dma.blocks.wrapping_sub(1) & 0x7F
            }
            0xFF56 if self.model == Model::Cgb => 0x3E | self.infrared,
            0xFF70 if self.model == Model::Cgb => 0xF8 | self.wram_bank,
            0xFF72..=0xFF74 if self.model == Model::Cgb => self.undocumented[usize::from(addr - 0xFF72)],
            0xFF75 if self.model == Model::Cgb => 0x8F | self.undocumented[3],
            0xFF76 | 0xFF77 if self.model == Model::Cgb => {
                let ch = self.apu.channel_levels();
                let index = usize::from(addr - 0xFF76) * 2;
                ch[index].1 | ch[index + 1].1 << 4
            }
            _ => 0xFF,
        }
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cart.write_rom(addr, value),
            0x8000..=0x9FFF => {
                if self.ppu.vram_writable() {
                    self.ppu.write_vram(addr, value);
                }
            }
            0xA000..=0xBFFF => self.cart.write_ram(addr, value),
            0xC000..=0xFDFF => {
                let offset = self.wram_offset(addr);
                self.wram[offset] = value;
            }
            0xFE00..=0xFE9F => {
                if self.ppu.oam_writable() && !self.dma.active {
                    self.ppu.oam[usize::from(addr - 0xFE00)] = value;
                }
            }
            0xFEA0..=0xFEFF => {}
            0xFF00..=0xFF7F => self.write_io(addr, value),
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)] = value,
            0xFFFF => self.ie = value,
        }
    }

    fn write_io(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF00 => {
                let fired = self.joypad.write(value);
                if fired {
                    self.if_ |= irq::JOYPAD;
                }
            }
            0xFF01 | 0xFF02 => self.serial.write(addr, value),
            0xFF04..=0xFF07 => {
                let ev = self.timer.write_with_speed(addr, value, self.double_speed);
                if ev.div_apu {
                    self.apu.frame_sequencer();
                }
                let serial_edge = if self.serial.fast_clock() { ev.serial_fast_clock } else { ev.serial_clock };
                if serial_edge && self.serial.clock() {
                    self.if_ |= irq::SERIAL;
                }
            }
            0xFF0F => self.if_ = value & 0x1F,
            0xFF10..=0xFF3F => self.apu.write(addr, value),
            0xFF46 => {
                self.dma.reg = value;
                self.dma.pending_source = value;
                self.dma.start_delay = 2;
            }
            0xFF40..=0xFF4B => self.if_ |= self.ppu.write_register(addr, value),
            0xFF4D if self.model == Model::Cgb => self.speed_armed = value & 1 != 0,
            0xFF4F | 0xFF68..=0xFF6C if self.model == Model::Cgb => self.if_ |= self.ppu.write_register(addr, value),
            0xFF51..=0xFF55 if self.model == Model::Cgb => self.write_vram_dma(addr, value),
            0xFF56 if self.model == Model::Cgb => self.infrared = value & 0xC1,
            0xFF70 if self.model == Model::Cgb => self.wram_bank = value & 7,
            0xFF72..=0xFF74 if self.model == Model::Cgb => self.undocumented[usize::from(addr - 0xFF72)] = value,
            0xFF75 if self.model == Model::Cgb => self.undocumented[3] = value & 0x70,
            // Any non-zero write unmaps the boot ROM until reset.
            0xFF50 if value != 0 => self.boot_rom_mapped = false,
            _ => {}
        }
    }

    /// Side-effect-free read for debuggers: ignores PPU/DMA access locks.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.ppu.read_vram(addr),
            0xFE00..=0xFE9F => self.ppu.oam[usize::from(addr - 0xFE00)],
            _ => self.read_mem(addr),
        }
    }

    /// Debugger write: bypasses access locks but still goes through the
    /// mapper and registers (so writing 0x2000 switches banks, as expected).
    pub fn poke(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.ppu.write_vram(addr, value),
            0xFE00..=0xFE9F => self.ppu.oam[usize::from(addr - 0xFE00)] = value,
            _ => self.write_mem(addr, value),
        }
    }

    pub fn set_buttons(&mut self, buttons: Buttons) -> bool {
        let fired = self.joypad.set_pressed(buttons);
        if fired {
            self.if_ |= irq::JOYPAD;
        }
        fired
    }

    pub fn interrupt_flags(&self) -> (u8, u8) {
        (self.ie, self.if_)
    }

    pub fn wram(&self) -> &[u8] {
        &self.wram[..if self.model == Model::Cgb { 0x8000 } else { 0x2000 }]
    }

    pub fn hram(&self) -> &[u8; 0x7F] {
        &self.hram
    }

    /// ROM offset of an executed address, using the current bank mapping.
    #[inline]
    pub(crate) fn rom_offset(&self, addr: u16) -> Option<usize> {
        match addr {
            0x0000..=0x00FF if self.boot_rom_mapped => None,
            0x0000..=0x3FFF => Some(usize::from(addr)),
            0x4000..=0x7FFF => Some(self.cart.current_rom_bank() * 0x4000 + usize::from(addr - 0x4000)),
            _ => None,
        }
    }

    #[inline]
    fn note_read(&mut self, addr: u16, value: u8) {
        if let Some(p) = &mut self.profile {
            p.reads[Region::of(addr) as usize] += 1;
        }
        if let Some(w) = &self.read_watch {
            if w.contains(addr) && self.watch_hit.is_none() {
                self.watch_hit = Some(WatchHit { addr, value, write: false });
            }
        }
    }

    #[inline]
    fn note_write(&mut self, addr: u16, value: u8) {
        if let Some(p) = &mut self.profile {
            p.writes[Region::of(addr) as usize] += 1;
        }
        if let Some(w) = &self.write_watch {
            if w.contains(addr) && self.watch_hit.is_none() {
                self.watch_hit = Some(WatchHit { addr, value, write: true });
            }
        }
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.bool(self.model == Model::Cgb);
        self.cart.save(w);
        self.ppu.save(w);
        self.apu.save(w);
        self.timer.save(w);
        self.joypad.save(w);
        self.serial.save(w);
        w.u8s(&self.wram[..]);
        w.u8s(&self.hram);
        w.u8s(&[self.ie, self.if_, self.if_deferred, self.if_reasserted_late]);
        let d = &self.dma;
        w.u8s(&[d.reg, d.source, d.index, u8::from(d.active), d.start_delay, d.pending_source]);
        w.bool(self.boot_rom_mapped);
        w.u64(self.cycles);
        w.u8(self.wram_bank);
        w.bool(self.double_speed);
        w.bool(self.speed_armed);
        w.u64(self.base_clock_ticks);
        w.u8(self.apu_phase);
        w.u8(self.infrared);
        w.u8s(&self.undocumented);
        let v = &self.vram_dma;
        w.u16(v.source);
        w.u16(v.destination);
        w.u8(v.blocks);
        w.bool(v.active);
        w.bool(v.hblank);
        w.bool(v.block_pending);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        if r.bool()? != (self.model == Model::Cgb) {
            return Err(StateError::Corrupt("bus model"));
        }
        self.cart.load(r)?;
        self.ppu.load(r)?;
        self.apu.load(r)?;
        self.timer.load(r)?;
        self.joypad.load(r)?;
        self.serial.load(r)?;
        r.u8s(&mut self.wram[..])?;
        r.u8s(&mut self.hram)?;
        let mut b = [0u8; 4];
        r.u8s(&mut b)?;
        [self.ie, self.if_, self.if_deferred, self.if_reasserted_late] = [b[0], b[1] & 0x1F, b[2] & 0x1F, b[3] & 0x1F];
        let mut d = [0u8; 6];
        r.u8s(&mut d)?;
        if d[2] > 160 || d[4] > 2 || (d[3] != 0 && d[2] == 160) {
            return Err(StateError::Corrupt("dma"));
        }
        self.dma = Dma {
            reg: d[0],
            source: d[1],
            index: d[2],
            active: d[3] != 0,
            start_delay: d[4],
            pending_source: d[5],
            collided: None,
        };
        self.boot_rom_mapped = r.bool()? && self.boot_rom.is_some();
        self.cycles = r.u64()?;
        if self.cycles >= 1 << 62 {
            // 139,000 years of emulation: only a crafted state gets here, and
            // counters that close to wrapping would overflow later.
            return Err(StateError::Corrupt("cycle counter"));
        }
        self.wram_bank = r.u8()?;
        self.double_speed = r.bool()?;
        self.speed_armed = r.bool()?;
        self.base_clock_ticks = r.u64()?;
        self.apu_phase = r.u8()?;
        self.infrared = r.u8()?;
        r.u8s(&mut self.undocumented)?;
        self.vram_dma = VramDma {
            source: r.u16()?,
            destination: r.u16()?,
            blocks: r.u8()?,
            active: r.bool()?,
            hblank: r.bool()?,
            block_pending: r.bool()?,
        };
        let v = &self.vram_dma;
        if self.wram_bank > 7
            || self.apu_phase > 2
            || self.apu_phase & 1 != 0
            || self.base_clock_ticks >= 1 << 62
            || self.infrared & !0xC1 != 0
            || self.undocumented[3] & !0x70 != 0
            || v.destination & 15 != 0
            || v.source & 15 != 0
            || v.blocks > 128
            || v.active && v.blocks == 0
            || v.hblank && !v.active
            || v.block_pending && (!v.active || !v.hblank)
            || self.model == Model::Dmg && (self.wram_bank != 1 || self.double_speed || self.speed_armed || v.active)
        {
            return Err(StateError::Corrupt("CGB bus state"));
        }
        self.cpu_halted = false;
        Ok(())
    }
}

impl CpuBus for SystemBus {
    #[inline]
    fn read(&mut self, addr: u16) -> u8 {
        if self.model == Model::Dmg && addr & 0xFF00 == 0xFE00 {
            self.ppu.oam_bug_read(addr, self.dma.active);
        }
        let v = match self.dma_conflict(addr) {
            Some(dma_addr) => self.dma_source_read(dma_addr),
            None => self.read_mem(addr),
        };
        if self.profile.is_some() || self.read_watch.is_some() {
            self.note_read(addr, v);
        }
        self.tick();
        v
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        if self.profile.is_some() || self.write_watch.is_some() {
            self.note_write(addr, value);
        }
        if self.model == Model::Dmg && addr & 0xFF00 == 0xFE00 && !self.ppu.oam_writable() {
            self.ppu.oam_bug_write();
        }
        let conflict = self.dma_conflict(addr);
        match conflict {
            // The write lands on the DMA's address instead (on ROM that is a
            // mapper register) ...
            Some(_) if self.model == Model::Cgb => {}
            Some(dma_addr) if dma_addr < 0xA000 => self.dma.collided = Some(Collision::Redirect(dma_addr, value)),
            // ... or, from cartridge RAM/WRAM, is lost and ANDs into the OAM
            // byte the DMA writes.
            Some(_) => self.dma.collided = Some(Collision::MaskOam(value)),
            None => self.write_mem(addr, value),
        }
        self.tick_with_if_write(addr == 0xFF0F && conflict.is_none());
    }

    #[inline]
    fn idle(&mut self) {
        self.tick();
    }

    #[inline]
    fn idle_at(&mut self, addr: u16) {
        if self.model == Model::Dmg && addr & 0xFF00 == 0xFE00 {
            self.ppu.oam_bug_write();
        }
        self.tick();
    }

    #[inline]
    fn pending_interrupts(&self) -> u8 {
        self.ie & self.if_ & 0x1F
    }

    fn halted_pending_interrupts(&self) -> u8 {
        // SameBoy sm83_cpu.c: a halted CPU samples IE/IF halfway through
        // its idle cycle. A running CPU samples after flushing that cycle.
        // Applying the early sample to both delays normal mode-2 IRQs by
        // one M-cycle (Mealybug m3_bgp_change's line-0 compensation).
        self.pending_interrupts() & !self.if_deferred
    }

    fn halt_samples_at_start(&self) -> bool {
        self.model == Model::Cgb
    }

    #[inline]
    fn acknowledge_interrupt(&mut self, mask: u8) {
        self.if_ = (self.if_ & !mask) | (mask & self.if_reasserted_late);
    }

    fn stop(&mut self) -> bool {
        if self.joypad.any_line_low() {
            return false;
        }
        self.timer.reset_div();
        true
    }

    fn speed_switch(&mut self) -> bool {
        if self.model != Model::Cgb || !self.speed_armed || self.joypad.any_line_low() {
            return false;
        }
        let interrupt_pending = self.pending_interrupts() != 0;
        // Pan Docs KEY1: DIV is held reset during the 2050 M-cycle pause.
        // Video/audio/RTC keep their base-clock rates, independently of CPU.
        let ev = self.timer.write_with_speed(0xFF04, 0, self.double_speed);
        if ev.div_apu {
            self.apu.frame_sequencer();
        }
        self.double_speed = !self.double_speed;
        self.speed_armed = false;
        if !interrupt_pending {
            for _ in 0..2050 {
                self.tick_peripherals(false, false);
            }
        }
        true
    }

    fn set_halted(&mut self, halted: bool) {
        // If HALT ended during HBlank, its postponed block may now proceed.
        if self.cpu_halted
            && !halted
            && self.vram_dma.active
            && self.vram_dma.hblank
            && self.ppu.mode() == Mode::HBlank
            && self.ppu.position().0 < 144
        {
            self.vram_dma.block_pending = true;
        }
        self.cpu_halted = halted;
    }

    fn idle_stopped(&mut self) {
        self.cycles += 1;
        let dots = if self.double_speed { 2 } else { 4 };
        self.base_clock_ticks += u64::from(dots);
        self.apu_phase += dots;
        if self.apu_phase >= 4 {
            self.apu_phase -= 4;
            self.ppu.tick_stopped();
            self.apu.tick_stopped();
        }
        self.cart.tick_rtc(u32::from(dots)); // the cartridge clock has its own crystal
    }

    /// Batch halted cycles, but hand control back at frame boundaries and
    /// when the caller's cycle budget runs out.
    #[inline]
    fn halt_should_yield(&self) -> bool {
        self.cycles >= self.halt_yield_at || self.ppu.frame_ready_pending()
    }
}

/// Build a zeroed boot ROM buffer from a slice (must be 256 bytes).
pub(crate) fn boot_rom_from(data: &[u8]) -> Option<Box<[u8; 0x100]>> {
    let arr: [u8; 0x100] = data.try_into().ok()?;
    Some(Box::new(arr))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bus(model: Model) -> SystemBus {
        let cart = Cartridge::new(crate::tests::test_rom(b"CGB BUS")).unwrap();
        let mut bus = SystemBus::new_with_model(cart, None, model);
        bus.poke(0xFF40, 0); // DMA and bank tests do not depend on LCD timing.
        bus
    }

    fn configure_dma(bus: &mut SystemBus, source: u16, destination: u16, length: u8) {
        bus.poke(0xFF51, (source >> 8) as u8);
        bus.poke(0xFF52, source as u8);
        bus.poke(0xFF53, (destination >> 8) as u8);
        bus.poke(0xFF54, destination as u8);
        bus.poke(0xFF55, length);
    }

    #[test]
    fn cgb_wram_banks_and_echo_alias_the_selected_bank() {
        let mut b = bus(Model::Cgb);
        assert_eq!(b.peek(0xFF70), 0xF8);
        assert_eq!(b.peek(0xFF02), 0x7F);
        b.poke(0xC123, 0x42);
        for bank in 1..=7 {
            b.poke(0xFF70, bank);
            b.poke(0xD123, bank);
            assert_eq!(b.peek(0xF123), bank);
            assert_eq!(b.peek(0xE123), 0x42);
        }
        b.poke(0xFF70, 0);
        assert_eq!(b.peek(0xFF70), 0xF8);
        assert_eq!(b.peek(0xD123), 1);
        b.poke(0xF123, 0xAB);
        b.poke(0xFF70, 1);
        assert_eq!(b.peek(0xD123), 0xAB);
        let mut dmg = bus(Model::Dmg);
        dmg.poke(0xFF70, 7);
        dmg.poke(0xD123, 0x55);
        assert_eq!(dmg.peek(0xFF70), 0xFF);
        assert_eq!(dmg.peek(0xF123), 0x55);
    }

    #[test]
    fn gdma_stalls_cpu_and_targets_the_selected_vram_bank_at_both_speeds() {
        for double in [false, true] {
            let mut b = bus(Model::Cgb);
            b.double_speed = double;
            for i in 0..32 {
                b.poke(0xC000 + i, i as u8 + 1);
            }
            b.poke(0xFF4F, 1);
            configure_dma(&mut b, 0xC00F, 0x8A0F, 1); // low nibbles ignored.
            let start = b.cycles;
            let dots = b.base_clock_ticks;
            b.idle();
            assert_eq!(b.cycles - start, 2 + if double { 32 } else { 16 });
            assert_eq!(b.base_clock_ticks - dots, 64 + if double { 4 } else { 8 });
            assert_eq!(b.peek(0xFF55), 0xFF);
            for i in 0..32 {
                assert_eq!(b.peek(0x8A00 + i), i as u8 + 1);
            }
            b.poke(0xFF4F, 0);
            assert_eq!(b.peek(0x8A00), 0);
        }
    }

    #[test]
    fn hblank_dma_pauses_in_halt_and_can_be_cancelled() {
        let mut b = bus(Model::Cgb);
        for i in 0..32 {
            b.poke(0xC000 + i, i as u8 + 1);
        }
        b.set_halted(true);
        configure_dma(&mut b, 0xC000, 0x8000, 0x81);
        for _ in 0..10 {
            b.idle();
        }
        assert_eq!(b.peek(0x8000), 0);
        assert_eq!(b.peek(0xFF55), 1);
        b.set_halted(false);
        b.idle();
        assert_eq!(b.peek(0x8000), 1);
        assert_eq!(b.peek(0x8010), 0);
        assert_eq!(b.peek(0xFF55), 0);
        b.poke(0xFF55, 0);
        assert_eq!(b.peek(0xFF55), 0x80);
        b.poke(0xFF40, 0x91);
        for _ in 0..250 {
            b.idle();
        }
        assert_eq!(b.peek(0x8010), 0);
    }

    #[test]
    fn vram_dma_wraps_the_vram_address_without_ending_the_counter() {
        let mut b = bus(Model::Cgb);
        for i in 0..32 {
            b.poke(0xC000 + i, i as u8 + 1);
        }
        configure_dma(&mut b, 0xC000, 0x9FF0, 1);
        b.idle();
        assert_eq!(b.peek(0x9FF0), 1);
        assert_eq!(b.peek(0x8000), 17);
        assert_eq!(b.peek(0xFF55), 0xFF);
        assert_eq!(b.vram_dma.destination, 0xA010);
    }

    #[test]
    fn hblank_dma_transfers_one_block_per_visible_scanline() {
        let mut b = bus(Model::Cgb);
        for i in 0..32 {
            b.poke(0xC000 + i, i as u8 + 1);
        }
        b.poke(0xFF40, 0x91);
        while b.ppu.mode() != Mode::Drawing {
            b.idle();
        }
        configure_dma(&mut b, 0xC000, 0x8000, 0x81);
        for _ in 0..150 {
            b.idle();
            if b.vram_dma.blocks == 1 {
                break;
            }
        }
        assert_eq!(b.peek(0x8000), 1);
        assert_eq!(b.peek(0x8010), 0);
        assert_eq!(b.ppu.mode(), Mode::HBlank);
        let first_line = b.ppu.position().0;
        for _ in 0..150 {
            b.idle();
            if !b.vram_dma.active {
                break;
            }
        }
        assert_eq!(b.peek(0x8010), 17);
        assert_eq!(b.ppu.position().0, first_line + 1);
        assert_eq!(b.peek(0xFF55), 0xFF);
    }

    #[test]
    fn speed_switch_preserves_real_time_and_holds_div_during_the_pause() {
        let mut b = bus(Model::Cgb);
        b.poke(0xFF00, 0x30);
        b.poke(0xFF4D, 1);
        let start = b.cycles;
        assert!(b.speed_switch());
        assert_eq!(b.cycles - start, 2050);
        assert_eq!(b.base_clock_ticks, 4100);
        assert_eq!(b.timer.system_counter(), 0);
        assert_eq!(b.peek(0xFF4D), 0xFE);
        b.apu.clear_samples();
        let start = b.base_clock_ticks;
        for _ in 0..2048 {
            b.idle();
        }
        assert_eq!(b.base_clock_ticks - start, 4096);
        assert_eq!(b.timer.system_counter(), 8192);
        // 4096 base dots produce ~47 stereo samples, independently of CPU speed.
        assert!((92..=96).contains(&b.apu.samples().len()));
        b.poke(0xFF4D, 1);
        assert!(b.speed_switch());
        assert!(!b.double_speed());
        assert_eq!(b.peek(0xFF4D), 0x7E);
    }

    #[test]
    fn cgb_fast_serial_finishes_after_128_cpu_clocks() {
        let mut b = bus(Model::Cgb);
        b.poke(0xFF04, 0);
        b.poke(0xFF01, 0x5A);
        b.poke(0xFF02, 0x83);
        b.poke(0xFF0F, 0);
        for _ in 0..31 {
            b.idle();
        }
        assert_eq!(b.peek(0xFF0F) & irq::SERIAL, 0);
        b.idle();
        assert_ne!(b.peek(0xFF0F) & irq::SERIAL, 0);
        assert_eq!(b.peek(0xFF01), 0xFF);
        assert_eq!(b.peek(0xFF02), 0x7F);
    }

    #[test]
    fn pending_interrupt_skips_the_speed_switch_pause() {
        let mut b = bus(Model::Cgb);
        b.poke(0xFF00, 0x30);
        b.poke(0xFFFF, irq::TIMER);
        b.poke(0xFF0F, irq::TIMER);
        b.poke(0xFF4D, 1);
        let cycles = b.cycles;
        assert!(b.speed_switch());
        assert_eq!(b.cycles, cycles);
        assert!(b.double_speed());
        assert_eq!(b.peek(0xFF4D), 0xFE);
    }

    #[test]
    fn hdma_cancel_also_writes_the_length_latch() {
        let mut b = bus(Model::Cgb);
        configure_dma(&mut b, 0xC000, 0x8000, 0x83);
        b.idle();
        assert_eq!(b.peek(0xFF55), 2);
        b.poke(0xFF55, 0);
        assert_eq!(b.peek(0xFF55), 0x80);
    }

    #[test]
    fn cgb_oam_dma_leaves_the_other_memory_bus_accessible() {
        let mut b = bus(Model::Cgb);
        b.poke(0xC000, 0xAA);
        b.poke(0xFF46, 0xC0);
        b.idle();
        b.idle();
        assert!(b.dma_active());
        assert_eq!(b.dma_conflict(0x0100), None);
        assert!(b.dma_conflict(0xC080).is_some());
        assert_eq!(b.dma_conflict(0x8000), None);
    }

    #[test]
    fn timer_reload_interrupt_is_later_than_the_dmg_halt_sample() {
        let mut b = bus(Model::Dmg);
        b.poke(0xFF04, 0);
        b.poke(0xFF05, 0xFF);
        b.poke(0xFF07, 5);
        b.poke(0xFF0F, 0);
        b.poke(0xFFFF, irq::TIMER);
        for _ in 0..4 {
            b.idle();
        }
        assert_eq!(b.pending_interrupts(), 0);
        b.idle(); // TIMA reloads at the end of this M-cycle.
        assert_eq!(b.pending_interrupts(), irq::TIMER);
        assert_eq!(b.halted_pending_interrupts(), 0);
        b.idle();
        assert_eq!(b.halted_pending_interrupts(), irq::TIMER);
    }

    #[test]
    fn cgb_bus_state_restores_banks_dma_and_clock_phase() {
        let mut b = bus(Model::Cgb);
        b.double_speed = true;
        b.poke(0xFF70, 7);
        b.poke(0xD123, 0xA5);
        b.poke(0xC000, 0x77);
        b.idle(); // half of an APU tick.
        b.set_halted(true);
        configure_dma(&mut b, 0xC000, 0x8000, 0x81);
        let mut w = StateWriter::new();
        b.save(&mut w);
        let bytes = w.finish();
        let mut resumed = bus(Model::Cgb);
        resumed.load(&mut StateReader::new(&bytes)).unwrap();
        assert_eq!(resumed.peek(0xD123), 0xA5);
        assert_eq!(resumed.base_clock_ticks, 2);
        assert_eq!(resumed.apu_phase, 2);
        assert!(resumed.double_speed);
        for machine in [&mut b, &mut resumed] {
            machine.set_halted(true);
            machine.set_halted(false);
            machine.idle();
            assert_eq!(machine.peek(0x8000), 0x77);
            assert_eq!(machine.peek(0xFF55), 0);
        }
        assert!(bus(Model::Dmg).load(&mut StateReader::new(&bytes)).is_err());
    }

    #[test]
    fn invalid_cgb_bus_states_are_rejected() {
        for bad in 0..5 {
            let mut b = bus(Model::Cgb);
            match bad {
                0 => b.wram_bank = 8,
                1 => b.apu_phase = 3,
                2 => b.vram_dma.active = true, // Zero remaining blocks.
                3 => b.vram_dma.destination = 1,
                _ => b.vram_dma.source = 1, // Transfer snapshots are block-aligned.
            }
            let mut w = StateWriter::new();
            b.save(&mut w);
            assert!(bus(Model::Cgb).load(&mut StateReader::new(&w.finish())).is_err());
        }
    }
}
