//! The system bus: memory map, I/O registers, OAM DMA, and the per-M-cycle
//! clock that drives every peripheral.
//!
//! Every CPU access performs the access and then advances the whole machine
//! by one M-cycle ("access, then tick" — ADR-0001).

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::CpuBus;
use crate::joypad::{Buttons, Joypad};
use crate::ppu::Ppu;
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
    pub(crate) cart: Cartridge,
    pub(crate) ppu: Ppu,
    pub(crate) apu: Apu,
    pub(crate) timer: Timer,
    pub(crate) joypad: Joypad,
    pub(crate) serial: Serial,
    wram: Box<[u8; 0x2000]>,
    hram: [u8; 0x7F],
    ie: u8,
    if_: u8,
    /// IF bits raised too late in the current M-cycle to be dispatched yet.
    if_deferred: u8,
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
        let booting = boot_rom.is_some();
        let ppu = if booting {
            Ppu::power_on()
        } else {
            let mut ppu = Ppu::new();
            ppu.load_boot_logo(&cart.rom()[0x104..0x134]);
            ppu
        };
        Self {
            cart,
            ppu,
            apu: if booting { Apu::power_on() } else { Apu::new() },
            timer: if booting { Timer::power_on() } else { Timer::new() },
            joypad: Joypad::new(),
            serial: Serial::new(),
            wram: Box::new([0; 0x2000]),
            hram: [0; 0x7F],
            ie: 0,
            if_: if booting { 0 } else { 0x01 },
            if_deferred: 0,
            dma: Dma { reg: 0xFF, ..Dma::default() },
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

    // --- clock ---------------------------------------------------------------

    /// Advances every peripheral by one M-cycle.
    #[inline]
    fn tick(&mut self) {
        self.cycles += 1;
        self.if_deferred = 0;
        let t = self.timer.tick();
        if t.interrupt {
            self.if_ |= irq::TIMER;
        }
        if t.div_apu {
            self.apu.frame_sequencer();
        }
        if t.serial_clock && self.serial.clock() {
            self.if_ |= irq::SERIAL;
        }
        let p = self.ppu.tick();
        self.if_deferred |= p.late & !self.if_;
        self.if_ |= p.now | p.late;
        self.apu.tick();
        self.tick_dma();
        self.cart.tick_rtc(4);
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
        (vram_bus(addr) == vram_bus(base)).then_some(current)
    }

    /// What the DMA unit sees at `addr` (it bypasses PPU access locks).
    fn dma_source_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cart.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.vram[usize::from(addr - 0x8000)],
            0xA000..=0xBFFF => self.cart.read_ram(addr),
            _ => self.wram[usize::from(addr & 0x1FFF)],
        }
    }

    pub fn dma_active(&self) -> bool {
        self.dma.active
    }

    // --- memory map ------------------------------------------------------------

    /// A CPU read (with PPU/DMA access restrictions), no clocking.
    fn read_mem(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF if self.boot_rom_mapped => self.boot_rom.as_ref().map_or(0xFF, |b| b[usize::from(addr)]),
            0x0000..=0x7FFF => self.cart.read_rom(addr),
            0x8000..=0x9FFF => {
                if self.ppu.vram_readable() {
                    self.ppu.vram[usize::from(addr - 0x8000)]
                } else {
                    0xFF
                }
            }
            0xA000..=0xBFFF => self.cart.read_ram(addr),
            0xC000..=0xFDFF => self.wram[usize::from(addr & 0x1FFF)],
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
            _ => 0xFF,
        }
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cart.write_rom(addr, value),
            0x8000..=0x9FFF => {
                if self.ppu.vram_writable() {
                    self.ppu.vram[usize::from(addr - 0x8000)] = value;
                }
            }
            0xA000..=0xBFFF => self.cart.write_ram(addr, value),
            0xC000..=0xFDFF => self.wram[usize::from(addr & 0x1FFF)] = value,
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
                let ev = self.timer.write(addr, value);
                if ev.div_apu {
                    self.apu.frame_sequencer();
                }
                if ev.serial_clock && self.serial.clock() {
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
            // Any non-zero write unmaps the boot ROM until reset.
            0xFF50 if value != 0 => self.boot_rom_mapped = false,
            _ => {}
        }
    }

    /// Side-effect-free read for debuggers: ignores PPU/DMA access locks.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.ppu.vram[usize::from(addr - 0x8000)],
            0xFE00..=0xFE9F => self.ppu.oam[usize::from(addr - 0xFE00)],
            _ => self.read_mem(addr),
        }
    }

    /// Debugger write: bypasses access locks but still goes through the
    /// mapper and registers (so writing 0x2000 switches banks, as expected).
    pub fn poke(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.ppu.vram[usize::from(addr - 0x8000)] = value,
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

    pub fn wram(&self) -> &[u8; 0x2000] {
        &self.wram
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
        self.cart.save(w);
        self.ppu.save(w);
        self.apu.save(w);
        self.timer.save(w);
        self.joypad.save(w);
        self.serial.save(w);
        w.u8s(&self.wram[..]);
        w.u8s(&self.hram);
        w.u8s(&[self.ie, self.if_, self.if_deferred]);
        let d = &self.dma;
        w.u8s(&[d.reg, d.source, d.index, u8::from(d.active), d.start_delay, d.pending_source]);
        w.bool(self.boot_rom_mapped);
        w.u64(self.cycles);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        self.cart.load(r)?;
        self.ppu.load(r)?;
        self.apu.load(r)?;
        self.timer.load(r)?;
        self.joypad.load(r)?;
        self.serial.load(r)?;
        r.u8s(&mut self.wram[..])?;
        r.u8s(&mut self.hram)?;
        let mut b = [0u8; 3];
        r.u8s(&mut b)?;
        [self.ie, self.if_, self.if_deferred] = [b[0], b[1] & 0x1F, b[2] & 0x1F];
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
        Ok(())
    }
}

impl CpuBus for SystemBus {
    #[inline]
    fn read(&mut self, addr: u16) -> u8 {
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
        match self.dma_conflict(addr) {
            // The write lands on the DMA's address instead (on ROM that is a
            // mapper register) ...
            Some(dma_addr) if dma_addr < 0xA000 => self.dma.collided = Some(Collision::Redirect(dma_addr, value)),
            // ... or, from cartridge RAM/WRAM, is lost and ANDs into the OAM
            // byte the DMA writes.
            Some(_) => self.dma.collided = Some(Collision::MaskOam(value)),
            None => self.write_mem(addr, value),
        }
        self.tick();
    }

    #[inline]
    fn idle(&mut self) {
        self.tick();
    }

    #[inline]
    fn pending_interrupts(&self) -> u8 {
        self.ie & self.if_ & !self.if_deferred & 0x1F
    }

    #[inline]
    fn acknowledge_interrupt(&mut self, mask: u8) {
        self.if_ &= !mask;
    }

    fn stop(&mut self) -> bool {
        if self.joypad.any_line_low() {
            return false;
        }
        self.timer.reset_div();
        true
    }

    fn idle_stopped(&mut self) {
        self.cycles += 1;
        self.ppu.tick_stopped();
        self.apu.tick_stopped();
        self.cart.tick_rtc(4); // the cartridge clock has its own crystal
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
