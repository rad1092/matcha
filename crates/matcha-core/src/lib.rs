//! matcha-core: a cycle-accurate Game Boy (DMG) emulator core.
//!
//! * `no_std` + `alloc`, zero dependencies, `#![forbid(unsafe_code)]`.
//! * Deterministic: the same ROM, inputs and save state always produce the
//!   same frames and audio, bit for bit.
//!
//! ```no_run
//! # let rom: Vec<u8> = Vec::new();
//! use matcha_core::{GameBoy, Buttons, RunEvent};
//! let mut gb = GameBoy::new(rom).expect("valid ROM");
//! gb.set_buttons(Buttons::START);
//! while gb.run_frame() != RunEvent::FrameComplete {}
//! let shades = gb.framebuffer(); // 160x144 values in 0..=3
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

extern crate alloc;

pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod disasm;
pub mod joypad;
pub mod ppu;
pub mod profile;
pub mod serial;
pub mod state;
pub mod timer;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bus::{AddrSet, SystemBus};
use cpu::{Cpu, PowerState, Registers, Step, StepKind};

pub use bus::WatchHit;
pub use cartridge::{Cartridge, CartridgeError, Header, MapperKind};
pub use disasm::Instruction;
pub use joypad::Buttons;
pub use ppu::{HEIGHT, WIDTH};
pub use profile::Profile;
pub use state::StateError;

/// M-cycles per video frame (70224 dots / 4).
pub const MCYCLES_PER_FRAME: u64 = 17_556;
/// The DMG's native refresh rate (≈59.7275 Hz).
pub const FRAME_RATE: f64 = 4_194_304.0 / 70_224.0;

/// Why [`GameBoy::run_frame`] (or a sibling) returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEvent {
    /// A frame finished (VBlank started); the framebuffer is complete.
    FrameComplete,
    /// Execution reached a PC breakpoint; the instruction has not run yet.
    Breakpoint { pc: u16 },
    /// An instruction at `pc` touched a watched address.
    Watchpoint { pc: u16, hit: WatchHit },
    /// The cycle budget ran out before anything else happened.
    CycleBudget,
}

/// Classic DMG palettes (0xRRGGBB, lightest first).
pub mod palettes {
    /// Neutral greys, as used by the conformance screenshots.
    pub const GREY: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];
    /// matcha's house palette: whisked-tea greens.
    pub const MATCHA: [u32; 4] = [0xE8F2C8, 0xA9C47F, 0x5E7F4A, 0x1F3326];
    /// The original pea-soup LCD.
    pub const DMG: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F];
}

/// A complete Game Boy.
pub struct GameBoy {
    cpu: Cpu,
    bus: SystemBus,
    breakpoints: Option<AddrSet>,
    /// Kept so [`GameBoy::reset`] can boot the same way again.
    boot_rom: Option<Box<[u8; 0x100]>>,
}

impl core::fmt::Debug for GameBoy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GameBoy")
            .field("title", &self.bus.cart.header().title)
            .field("pc", &self.cpu.regs.pc)
            .field("cycles", &self.bus.cycles)
            .finish_non_exhaustive()
    }
}

impl GameBoy {
    /// Creates a machine in the state the DMG boot ROM leaves it in, ready to
    /// execute the cartridge at 0x0100. No boot ROM image is needed.
    pub fn new(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        let cart = Cartridge::new(rom)?;
        Ok(Self::from_cartridge(cart, None))
    }

    /// Creates a machine that runs `boot_rom` (256 bytes) from 0x0000.
    pub fn with_boot_rom(rom: Vec<u8>, boot_rom: &[u8]) -> Result<Self, CartridgeError> {
        let cart = Cartridge::new(rom)?;
        let boot = bus::boot_rom_from(boot_rom).ok_or(CartridgeError::TooSmall(boot_rom.len()))?;
        Ok(Self::from_cartridge(cart, Some(boot)))
    }

    fn from_cartridge(cart: Cartridge, boot: Option<Box<[u8; 0x100]>>) -> Self {
        let mut cpu = Cpu::new();
        if boot.is_none() {
            let checksum_zero = cart.header().header_checksum == 0;
            cpu.regs = Registers {
                a: 0x01,
                f: if checksum_zero { 0x80 } else { 0xB0 },
                b: 0x00,
                c: 0x13,
                d: 0x00,
                e: 0xD8,
                h: 0x01,
                l: 0x4D,
                sp: 0xFFFE,
                pc: 0x0100,
            };
        }
        Self { cpu, bus: SystemBus::new(cart, boot.clone()), breakpoints: None, boot_rom: boot }
    }

    /// Power-cycles the machine. Cartridge RAM (the save file), breakpoints,
    /// watchpoints and profiling survive.
    pub fn reset(&mut self) {
        let mut cart = self.bus.cart.clone();
        cart.reset();
        let mut fresh = Self::from_cartridge(cart, self.boot_rom.take());
        fresh.breakpoints = self.breakpoints.take();
        fresh.bus.profile = self.bus.profile.take();
        fresh.bus.read_watch = self.bus.read_watch.take();
        fresh.bus.write_watch = self.bus.write_watch.take();
        *self = fresh;
    }

    // --- running -----------------------------------------------------------------

    /// Executes one instruction (or one interrupt dispatch / halted cycle).
    pub fn step(&mut self) -> Step {
        if self.bus.profile.is_none() {
            return self.cpu.step(&mut self.bus);
        }
        let before = self.bus.cycles;
        let rom_offset = self.bus.rom_offset(self.cpu.regs.pc);
        let step = self.cpu.step(&mut self.bus);
        let cycles = self.bus.cycles - before;
        if let Some(p) = &mut self.bus.profile {
            match step.kind {
                StepKind::Instruction { opcode, prefixed } => {
                    p.instructions += 1;
                    p.busy_cycles += cycles;
                    if prefixed {
                        p.opcodes[0xCB] += 1;
                        p.cb_opcodes[usize::from(opcode)] += 1;
                    } else {
                        p.opcodes[usize::from(opcode)] += 1;
                    }
                    if let Some(off) = rom_offset {
                        p.mark_executed(off);
                    }
                }
                StepKind::Interrupt { vector } => {
                    p.interrupt_cycles += cycles;
                    if (0x40..=0x60).contains(&vector) {
                        p.interrupts[usize::from((vector - 0x40) / 8)] += 1;
                    }
                }
                StepKind::Halted => p.halted_cycles += cycles,
                StepKind::Stopped => p.stopped_cycles += cycles,
                StepKind::Locked { opcode } => {
                    p.stopped_cycles += cycles;
                    p.locked_opcode = Some(opcode);
                }
            }
        }
        step
    }

    /// Runs until the next frame completes, a breakpoint/watchpoint triggers,
    /// or two frames' worth of cycles pass (only possible with the CPU stuck
    /// in STOP while the LCD is off). A breakpoint at the current PC is
    /// stepped over, so calling this again after a breakpoint continues.
    pub fn run_frame(&mut self) -> RunEvent {
        self.run_cycles(MCYCLES_PER_FRAME * 2, true)
    }

    /// Runs for at most `budget` M-cycles. With `stop_at_frame`, also returns
    /// as soon as a frame completes.
    pub fn run_cycles(&mut self, budget: u64, stop_at_frame: bool) -> RunEvent {
        let start = self.bus.cycles;
        let mut first = true;
        loop {
            if !first {
                if let Some(bp) = &self.breakpoints {
                    if bp.contains(self.cpu.regs.pc) && self.cpu.power_state() == PowerState::Running {
                        return RunEvent::Breakpoint { pc: self.cpu.regs.pc };
                    }
                }
            }
            first = false;
            let step = self.step();
            if let Some(hit) = self.bus.watch_hit.take() {
                return RunEvent::Watchpoint { pc: step.pc, hit };
            }
            if self.bus.ppu.take_frame_ready() && stop_at_frame {
                return RunEvent::FrameComplete;
            }
            if self.bus.cycles - start >= budget {
                return RunEvent::CycleBudget;
            }
        }
    }

    // --- input / output --------------------------------------------------------------

    /// Sets the full button state (1 = held).
    pub fn set_buttons(&mut self, buttons: Buttons) {
        let newly_pressed = buttons.0 & !self.bus.joypad.pressed().0 != 0;
        self.bus.set_buttons(buttons);
        if newly_pressed {
            self.cpu.wake_from_stop();
        }
    }

    pub fn buttons(&self) -> Buttons {
        self.bus.joypad.pressed()
    }

    /// 160x144 shade indices (0 = lightest .. 3 = darkest), row-major.
    pub fn framebuffer(&self) -> &[u8; WIDTH * HEIGHT] {
        self.bus.ppu.framebuffer()
    }

    /// Converts the framebuffer to RGBA8 using `palette` (0xRRGGBB, lightest first).
    pub fn render_rgba(&self, palette: &[u32; 4], out: &mut [u8]) {
        for (px, &shade) in out.chunks_exact_mut(4).zip(self.framebuffer().iter()) {
            let c = palette[usize::from(shade & 3)];
            px.copy_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 0xFF]);
        }
    }

    /// Completed frames since power-on.
    pub fn frame_count(&self) -> u64 {
        self.bus.ppu.frame_count()
    }

    /// M-cycles since power-on.
    pub fn cycles(&self) -> u64 {
        self.bus.cycles
    }

    /// Interleaved stereo f32 samples produced since the last [`Self::clear_audio`].
    pub fn audio_samples(&self) -> &[f32] {
        self.bus.apu.samples()
    }

    pub fn clear_audio(&mut self) {
        self.bus.apu.clear_samples();
    }

    pub fn set_sample_rate(&mut self, hz: u32) {
        self.bus.apu.set_sample_rate(hz);
    }

    /// Mutes channels for listening/visualising: bit n = channel n+1 audible.
    pub fn set_audio_channel_mask(&mut self, mask: u8) {
        self.bus.apu.channel_mask = mask & 0x0F;
    }

    /// Per-channel (enabled, digital level 0..15) for visualisers.
    pub fn audio_channel_levels(&self) -> [(bool, u8); 4] {
        self.bus.apu.channel_levels()
    }

    /// Bytes sent over the link port (Blargg's test ROMs print here).
    pub fn serial_output(&self) -> &[u8] {
        self.bus.serial.output()
    }

    pub fn clear_serial_output(&mut self) {
        self.bus.serial.clear_output();
    }

    // --- cartridge ---------------------------------------------------------------------

    pub fn header(&self) -> &Header {
        self.bus.cart.header()
    }

    pub fn cartridge(&self) -> &Cartridge {
        &self.bus.cart
    }

    /// Battery-backed cartridge RAM (the `.sav` file), if any.
    pub fn battery_ram(&self) -> Option<&[u8]> {
        self.bus.cart.battery_ram()
    }

    pub fn load_battery_ram(&mut self, data: &[u8]) {
        self.bus.cart.load_battery_ram(data);
    }

    /// True if battery RAM changed since the last call.
    pub fn take_battery_dirty(&mut self) -> bool {
        self.bus.cart.take_ram_dirty()
    }

    /// Advances the MBC3 clock by real seconds (e.g. time the game was closed).
    pub fn rtc_advance_seconds(&mut self, seconds: u64) {
        self.bus.cart.rtc_advance_seconds(seconds);
    }

    // --- save states -------------------------------------------------------------------

    pub fn save_state(&self) -> Vec<u8> {
        let mut w = state::StateWriter::new();
        w.header(self.bus.cart.rom_id());
        self.cpu.save(&mut w);
        self.bus.save(&mut w);
        w.finish()
    }

    /// Restores a state produced by [`Self::save_state`]. On error the
    /// machine is left exactly as it was.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let backup = self.save_state();
        match self.load_state_unchecked(data) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.load_state_unchecked(&backup).expect("restoring a state we just saved cannot fail");
                Err(e)
            }
        }
    }

    fn load_state_unchecked(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = state::StateReader::new(data);
        r.header(self.bus.cart.rom_id())?;
        self.cpu.load(&mut r)?;
        self.bus.load(&mut r)?;
        if !r.is_empty() {
            return Err(StateError::Corrupt("trailing data"));
        }
        self.bus.watch_hit = None;
        Ok(())
    }

    // --- debugging ---------------------------------------------------------------------

    pub fn registers(&self) -> Registers {
        self.cpu.regs
    }

    pub fn set_registers(&mut self, regs: Registers) {
        self.cpu.regs = Registers { f: regs.f & 0xF0, ..regs };
    }

    pub fn ime(&self) -> bool {
        self.cpu.ime
    }

    pub fn power_state(&self) -> PowerState {
        self.cpu.power_state()
    }

    /// (IE, IF).
    pub fn interrupt_registers(&self) -> (u8, u8) {
        self.bus.interrupt_flags()
    }

    /// Reads memory without side effects or access restrictions.
    pub fn peek(&self, addr: u16) -> u8 {
        self.bus.peek(addr)
    }

    /// Writes memory as the CPU would, minus access restrictions.
    pub fn poke(&mut self, addr: u16, value: u8) {
        self.bus.poke(addr, value);
    }

    pub fn disassemble(&self, addr: u16) -> Instruction {
        let b = [self.peek(addr), self.peek(addr.wrapping_add(1)), self.peek(addr.wrapping_add(2))];
        disasm::decode(addr, b)
    }

    /// PPU registers: LCDC, STAT, SCY, SCX, LY, LYC, BGP, OBP0, OBP1, WY, WX.
    pub fn ppu_registers(&self) -> [u8; 11] {
        self.bus.ppu.registers()
    }

    /// Current (line, dot) of the PPU.
    pub fn ppu_position(&self) -> (u8, u16) {
        self.bus.ppu.position()
    }

    /// Decodes VRAM tile `index` (0..384) into 64 colour indices.
    pub fn decode_tile(&self, index: usize, out: &mut [u8; 64]) {
        self.bus.ppu.decode_tile(index, out);
    }

    pub fn add_breakpoint(&mut self, addr: u16) -> bool {
        self.breakpoints.get_or_insert_with(AddrSet::new).insert(addr)
    }

    pub fn remove_breakpoint(&mut self, addr: u16) -> bool {
        let removed = self.breakpoints.as_mut().is_some_and(|b| b.remove(addr));
        if self.breakpoints.as_ref().is_some_and(AddrSet::is_empty) {
            self.breakpoints = None;
        }
        removed
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints = None;
    }

    pub fn breakpoints(&self) -> Vec<u16> {
        self.breakpoints.as_ref().map(|b| b.iter().collect()).unwrap_or_default()
    }

    /// Watches reads (`write == false`) or writes of `addr`.
    pub fn add_watchpoint(&mut self, addr: u16, write: bool) -> bool {
        let set = if write { &mut self.bus.write_watch } else { &mut self.bus.read_watch };
        set.get_or_insert_with(AddrSet::new).insert(addr)
    }

    pub fn clear_watchpoints(&mut self) {
        self.bus.read_watch = None;
        self.bus.write_watch = None;
        self.bus.watch_hit = None;
    }

    pub fn watchpoints(&self) -> Vec<(u16, bool)> {
        let mut v: Vec<(u16, bool)> = Vec::new();
        if let Some(w) = &self.bus.read_watch {
            v.extend(w.iter().map(|a| (a, false)));
        }
        if let Some(w) = &self.bus.write_watch {
            v.extend(w.iter().map(|a| (a, true)));
        }
        v
    }

    /// Starts (or restarts) execution profiling.
    pub fn enable_profiling(&mut self) {
        self.bus.profile = Some(Box::new(Profile::new(self.bus.cart.rom().len())));
    }

    pub fn disable_profiling(&mut self) -> Option<Box<Profile>> {
        self.bus.profile.take()
    }

    pub fn profile(&self) -> Option<&Profile> {
        self.bus.profile.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// A tiny ROM-only cartridge that scrolls the screen, pokes VRAM and
    /// plays a tone, waiting for VBlank with HALT each frame.
    pub(crate) fn test_rom(title: &[u8]) -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000];
        rom[0x40] = 0xD9; // VBlank handler: reti
        rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]); // nop; jp $0150
        rom[0x134..0x134 + title.len()].copy_from_slice(title);
        #[rustfmt::skip]
        let program: [u8; 55] = [
            0xF3,                   // di
            0x31, 0xFE, 0xFF,       // ld sp,$FFFE
            0x3E, 0x80, 0xE0, 0x26, // NR52 = $80
            0x3E, 0x77, 0xE0, 0x24, // NR50 = $77
            0x3E, 0xFF, 0xE0, 0x25, // NR51 = $FF
            0x3E, 0xF0, 0xE0, 0x12, // NR12 = $F0
            0x3E, 0x80, 0xE0, 0x11, // NR11 = $80
            0x3E, 0x00, 0xE0, 0x13, // NR13 = $00
            0x3E, 0x87, 0xE0, 0x14, // NR14 = $87 (trigger)
            // loop:
            0x21, 0x00, 0xC0,       // ld hl,$C000
            0x34,                   // inc (hl)
            0x7E,                   // ld a,(hl)
            0xE0, 0x42,             // ldh (SCY),a
            0x21, 0x10, 0x80,       // ld hl,$8010
            0x77,                   // ld (hl),a
            0x21, 0x00, 0x98,       // ld hl,$9800
            0x36, 0x01,             // ld (hl),$01
            0x3E, 0x01, 0xE0, 0xFF, // IE = VBlank
            0xFB,                   // ei
            0x76,                   // halt
            0xF3,                   // di
        ];
        rom[0x150..0x150 + program.len()].copy_from_slice(&program);
        let jr_at = 0x150 + program.len();
        rom[jr_at] = 0x18; // jr loop
        rom[jr_at + 1] = (0x170i32 - (jr_at as i32 + 2)) as i8 as u8;
        let checksum = rom[0x134..=0x14C].iter().fold(0u8, |a, &b| a.wrapping_sub(b).wrapping_sub(1));
        rom[0x14D] = checksum;
        rom
    }

    fn snapshot(gb: &GameBoy) -> (Vec<u8>, Registers, Vec<u32>, u64) {
        let audio = gb.audio_samples().iter().map(|s| s.to_bits()).collect();
        (gb.framebuffer().to_vec(), gb.registers(), audio, gb.cycles())
    }

    fn run(gb: &mut GameBoy, frames: u32) {
        for _ in 0..frames {
            while gb.run_frame() != RunEvent::FrameComplete {}
        }
    }

    #[test]
    fn test_rom_header_is_valid() {
        let gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        assert!(gb.header().header_checksum_ok);
        assert_eq!(gb.header().title, "MATCHATEST");
    }

    #[test]
    fn test_rom_actually_does_work() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        run(&mut gb, 20);
        assert!(gb.peek(0xC000) >= 18, "frame counter in WRAM advanced");
        assert!(!gb.audio_samples().is_empty(), "APU produced audio");
        assert!(gb.audio_samples().iter().any(|&s| s != 0.0), "audio is not silent");
    }

    #[test]
    fn two_machines_are_bit_identical() {
        let mut a = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        let mut b = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        for f in 0..40 {
            let buttons = Buttons(if f % 7 == 0 { 0x81 } else { 0 });
            a.set_buttons(buttons);
            b.set_buttons(buttons);
            run(&mut a, 1);
            run(&mut b, 1);
        }
        assert_eq!(snapshot(&a), snapshot(&b));
    }

    #[test]
    fn save_state_round_trip_replays_exactly() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        run(&mut gb, 25);
        let state = gb.save_state();
        gb.clear_audio();
        run(&mut gb, 30);
        let expected = snapshot(&gb);

        gb.load_state(&state).unwrap();
        gb.clear_audio();
        run(&mut gb, 30);
        assert_eq!(snapshot(&gb), expected, "reloading replays identically");

        let mut other = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        other.load_state(&state).unwrap();
        run(&mut other, 30);
        assert_eq!(snapshot(&other), expected, "a fresh machine resumes identically");
    }

    #[test]
    fn bad_states_are_rejected_without_side_effects() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        run(&mut gb, 10);
        let good = gb.save_state();
        let before = gb.save_state();

        assert_eq!(gb.load_state(b"not a state"), Err(StateError::BadMagic));
        assert_eq!(gb.load_state(&good[..good.len() / 2]), Err(StateError::Truncated));
        let mut trailing = good.clone();
        trailing.push(0);
        assert_eq!(gb.load_state(&trailing), Err(StateError::Corrupt("trailing data")));
        let mut other_rom = GameBoy::new(test_rom(b"OTHERGAME")).unwrap();
        assert_eq!(other_rom.load_state(&good), Err(StateError::WrongRom));
        let mut future = good.clone();
        future[6] = 0xFF;
        assert!(matches!(gb.load_state(&future), Err(StateError::Version { .. })));

        assert_eq!(gb.save_state(), before, "failed loads leave the machine untouched");
    }

    #[test]
    fn breakpoints_stop_before_execution_and_resume() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        gb.add_breakpoint(0x0170);
        assert_eq!(gb.run_frame(), RunEvent::Breakpoint { pc: 0x0170 });
        assert_eq!(gb.registers().pc, 0x0170);
        // Continuing steps over the breakpoint and stops there again next loop.
        let mut hits = 0;
        for _ in 0..10 {
            if let RunEvent::Breakpoint { pc } = gb.run_frame() {
                assert_eq!(pc, 0x0170);
                hits += 1;
            }
        }
        assert!(hits >= 5);
        assert!(gb.remove_breakpoint(0x0170));
        assert!(gb.breakpoints().is_empty());
    }

    #[test]
    fn watchpoints_report_the_writer() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        gb.add_watchpoint(0xC000, true);
        match gb.run_frame() {
            RunEvent::Watchpoint { pc, hit } => {
                assert_eq!(pc, 0x0173, "inc (hl) writes the counter");
                assert_eq!(hit, WatchHit { addr: 0xC000, value: 1, write: true });
            }
            other => panic!("expected watchpoint, got {other:?}"),
        }
    }

    #[test]
    fn profiler_accounts_for_every_cycle() {
        let mut gb = GameBoy::new(test_rom(b"MATCHATEST")).unwrap();
        gb.enable_profiling();
        let start = gb.cycles();
        run(&mut gb, 30);
        let p = gb.profile().unwrap();
        assert_eq!(p.total_cycles(), gb.cycles() - start);
        assert!(p.halted_cycles > p.busy_cycles, "the test ROM idles in HALT most of the frame");
        assert!(p.interrupts[0] >= 29, "one VBlank interrupt per frame");
        assert!(p.covered_bytes() > 20);
    }
}
