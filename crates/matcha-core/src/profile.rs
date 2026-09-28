//! Execution profiler: what a program actually does with the machine.
//!
//! Disabled by default and zero-cost when off (one `Option` check per step).
//! Feeds the corpus analysis in `analysis/` and the debugger's heatmap.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

/// Memory regions used for access accounting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Region {
    Rom0 = 0,
    RomX = 1,
    Vram = 2,
    Sram = 3,
    Wram = 4,
    Echo = 5,
    Oam = 6,
    Io = 7,
    Hram = 8,
    Ie = 9,
}

pub const REGION_NAMES: [&str; 10] = ["rom0", "romx", "vram", "sram", "wram", "echo", "oam", "io", "hram", "ie"];

impl Region {
    #[inline]
    pub fn of(addr: u16) -> Self {
        match addr {
            0x0000..=0x3FFF => Self::Rom0,
            0x4000..=0x7FFF => Self::RomX,
            0x8000..=0x9FFF => Self::Vram,
            0xA000..=0xBFFF => Self::Sram,
            0xC000..=0xDFFF => Self::Wram,
            0xE000..=0xFDFF => Self::Echo,
            0xFE00..=0xFEFF => Self::Oam,
            0xFF00..=0xFF7F => Self::Io,
            0xFF80..=0xFFFE => Self::Hram,
            0xFFFF => Self::Ie,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Profile {
    /// Executions per unprefixed opcode.
    pub opcodes: Box<[u64; 256]>,
    /// Executions per CB-prefixed opcode.
    pub cb_opcodes: Box<[u64; 256]>,
    pub instructions: u64,
    /// M-cycles spent executing instructions (including their memory accesses).
    pub busy_cycles: u64,
    /// M-cycles spent in HALT.
    pub halted_cycles: u64,
    /// M-cycles spent in STOP or locked by an illegal opcode.
    pub stopped_cycles: u64,
    /// M-cycles spent dispatching interrupts.
    pub interrupt_cycles: u64,
    /// Dispatches per interrupt source (VBlank, STAT, Timer, Serial, Joypad).
    pub interrupts: [u64; 5],
    pub reads: [u64; 10],
    pub writes: [u64; 10],
    /// Illegal opcode that locked the CPU, if any.
    pub locked_opcode: Option<u8>,
    /// One bit per ROM byte: an instruction started there.
    coverage: Vec<u64>,
    rom_len: usize,
}

impl Profile {
    pub fn new(rom_len: usize) -> Self {
        Self {
            opcodes: Box::new([0; 256]),
            cb_opcodes: Box::new([0; 256]),
            instructions: 0,
            busy_cycles: 0,
            halted_cycles: 0,
            stopped_cycles: 0,
            interrupt_cycles: 0,
            interrupts: [0; 5],
            reads: [0; 10],
            writes: [0; 10],
            locked_opcode: None,
            coverage: vec![0; rom_len.div_ceil(64)],
            rom_len,
        }
    }

    pub fn total_cycles(&self) -> u64 {
        self.busy_cycles + self.halted_cycles + self.stopped_cycles + self.interrupt_cycles
    }

    /// Fraction of time the CPU was doing work (not halted/stopped).
    pub fn cpu_utilization(&self) -> f64 {
        let total = self.total_cycles();
        if total == 0 {
            return 0.0;
        }
        (self.busy_cycles + self.interrupt_cycles) as f64 / total as f64
    }

    #[inline]
    pub(crate) fn mark_executed(&mut self, rom_offset: usize) {
        if rom_offset < self.rom_len {
            self.coverage[rom_offset / 64] |= 1 << (rom_offset % 64);
        }
    }

    /// Number of distinct ROM addresses where an instruction started.
    pub fn covered_bytes(&self) -> u64 {
        self.coverage.iter().map(|w| u64::from(w.count_ones())).sum()
    }

    pub fn rom_len(&self) -> usize {
        self.rom_len
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.rom_len);
    }
}
