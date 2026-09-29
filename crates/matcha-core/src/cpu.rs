//! Sharp SM83 CPU (the Game Boy's LR35902 core).
//!
//! The CPU owns no memory. Every bus access goes through [`CpuBus`], and each
//! call to `read`, `write` or `idle` represents exactly one M-cycle (4 T-cycles).
//! The bus implementation advances the rest of the system during that call, so
//! the timing of every memory access inside an instruction is observable by the
//! PPU, timer and APU exactly as on hardware (see ADR-0001).

/// The memory/peripheral side of the machine, as seen by the CPU.
pub trait CpuBus {
    /// Reads a byte. Consumes one M-cycle.
    fn read(&mut self, addr: u16) -> u8;
    /// Writes a byte. Consumes one M-cycle.
    fn write(&mut self, addr: u16, value: u8);
    /// An internal CPU cycle with no memory access. Consumes one M-cycle.
    fn idle(&mut self);
    /// Interrupts that are both requested and enabled (`IE & IF & 0x1F`).
    fn pending_interrupts(&self) -> u8;
    /// Clears the given bit in `IF` (interrupt acknowledged).
    fn acknowledge_interrupt(&mut self, mask: u8);
    /// Called when the `STOP` instruction executes (resets DIV on hardware).
    fn stop(&mut self) {}
    /// Lets a halted CPU idle through many M-cycles in one step until this
    /// returns true (the system uses it to stop at frame boundaries).
    /// The default never batches: one halted M-cycle per step.
    fn halt_should_yield(&self) -> bool {
        true
    }
}

/// Flag bits in the `F` register.
pub mod flags {
    pub const Z: u8 = 0x80;
    pub const N: u8 = 0x40;
    pub const H: u8 = 0x20;
    pub const C: u8 = 0x10;
}

use flags::{C, H, N, Z};

/// The architectural register file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub a: u8,
    /// Only the upper nibble is meaningful; the low nibble always reads as 0.
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    #[inline]
    pub fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f])
    }
    #[inline]
    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }
    #[inline]
    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }
    #[inline]
    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }
    #[inline]
    pub fn set_af(&mut self, v: u16) {
        let [a, f] = v.to_be_bytes();
        self.a = a;
        self.f = f & 0xF0;
    }
    #[inline]
    pub fn set_bc(&mut self, v: u16) {
        [self.b, self.c] = v.to_be_bytes();
    }
    #[inline]
    pub fn set_de(&mut self, v: u16) {
        [self.d, self.e] = v.to_be_bytes();
    }
    #[inline]
    pub fn set_hl(&mut self, v: u16) {
        [self.h, self.l] = v.to_be_bytes();
    }
    #[inline]
    fn flag(&self, mask: u8) -> bool {
        self.f & mask != 0
    }
}

/// What a single call to [`Cpu::step`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    /// Executed an instruction. `prefixed` is true for `CB xx` opcodes.
    Instruction { opcode: u8, prefixed: bool },
    /// Dispatched an interrupt to `vector` (`0x0000` if the dispatch was
    /// cancelled by the IE-push quirk).
    Interrupt { vector: u16 },
    /// Spent one M-cycle halted.
    Halted,
    /// Spent one M-cycle in STOP mode.
    Stopped,
    /// The CPU executed an illegal opcode and is hung until reset.
    Locked { opcode: u8 },
}

/// Result of [`Cpu::step`]: what happened, and at which program counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub pc: u16,
    pub kind: StepKind,
}

/// Low-power state of the CPU.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PowerState {
    #[default]
    Running = 0,
    Halted = 1,
    Stopped = 2,
    /// Illegal opcode executed; only a reset recovers.
    Locked = 3,
}

#[derive(Clone, Debug, Default)]
pub struct Cpu {
    pub regs: Registers,
    /// Interrupt master enable.
    pub ime: bool,
    /// `EI` was executed; IME becomes 1 after the next instruction.
    ei_delay: bool,
    power: PowerState,
    /// The next opcode fetch does not increment PC (HALT bug).
    halt_bug: bool,
    locked_opcode: u8,
}

/// Interrupt vectors, indexed by IF bit.
const VECTORS: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn power_state(&self) -> PowerState {
        self.power
    }

    pub fn is_halted(&self) -> bool {
        self.power == PowerState::Halted
    }

    /// Whether an `EI` is waiting to take effect.
    pub fn ei_pending(&self) -> bool {
        self.ei_delay
    }

    /// Leaves STOP mode (called by the system when a button is pressed).
    pub fn wake_from_stop(&mut self) {
        if self.power == PowerState::Stopped {
            self.power = PowerState::Running;
        }
    }

    /// Executes one instruction, one interrupt dispatch, or one halted cycle.
    pub fn step<B: CpuBus>(&mut self, bus: &mut B) -> Step {
        let pc = self.regs.pc;
        match self.power {
            PowerState::Running => {}
            PowerState::Halted => {
                // Idle until an interrupt is pending (or the bus asks us to
                // yield); each iteration is exactly one M-cycle.
                loop {
                    bus.idle();
                    if bus.pending_interrupts() != 0 {
                        // Wake-up: the interrupt (if IME) is dispatched next step.
                        self.power = PowerState::Running;
                        break;
                    }
                    if bus.halt_should_yield() {
                        break;
                    }
                }
                return Step { pc, kind: StepKind::Halted };
            }
            PowerState::Stopped => {
                bus.idle();
                return Step { pc, kind: StepKind::Stopped };
            }
            PowerState::Locked => {
                bus.idle();
                return Step { pc, kind: StepKind::Locked { opcode: self.locked_opcode } };
            }
        }

        if self.ime && bus.pending_interrupts() != 0 {
            let vector = self.dispatch_interrupt(bus);
            return Step { pc, kind: StepKind::Interrupt { vector } };
        }

        // EI takes effect after the instruction that follows it. The interrupt
        // check for this step already happened above with IME still clear.
        let ime_just_enabled = core::mem::take(&mut self.ei_delay);
        if ime_just_enabled {
            self.ime = true;
        }

        let opcode = self.fetch_opcode(bus);
        if opcode == 0xCB {
            let op = self.fetch(bus);
            self.execute_cb(bus, op);
            return Step { pc, kind: StepKind::Instruction { opcode: op, prefixed: true } };
        }
        self.execute(bus, opcode, ime_just_enabled);
        let kind = if self.power == PowerState::Locked {
            StepKind::Locked { opcode }
        } else {
            StepKind::Instruction { opcode, prefixed: false }
        };
        Step { pc, kind }
    }

    /// Interrupt dispatch: 5 M-cycles. The vector is chosen *after* the high
    /// byte of PC has been pushed, so a push that overwrites IE (SP wrapping
    /// onto 0xFFFF) can redirect or cancel the dispatch (mooneye `ie_push`).
    fn dispatch_interrupt<B: CpuBus>(&mut self, bus: &mut B) -> u16 {
        self.ime = false;
        bus.idle();
        bus.idle();
        let [hi, lo] = self.regs.pc.to_be_bytes();
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, hi);
        let pending = bus.pending_interrupts();
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, lo);
        let vector = if pending == 0 {
            0x0000
        } else {
            let bit = pending.trailing_zeros() as usize;
            bus.acknowledge_interrupt(1 << bit);
            VECTORS[bit]
        };
        self.regs.pc = vector;
        bus.idle();
        vector
    }

    #[inline]
    fn fetch_opcode<B: CpuBus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.regs.pc);
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.regs.pc = self.regs.pc.wrapping_add(1);
        }
        v
    }

    #[inline]
    fn fetch<B: CpuBus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        v
    }

    #[inline]
    fn fetch16<B: CpuBus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
        u16::from_le_bytes([lo, hi])
    }

    #[inline]
    fn push<B: CpuBus>(&mut self, bus: &mut B, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, hi);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, lo);
    }

    #[inline]
    fn pop<B: CpuBus>(&mut self, bus: &mut B) -> u16 {
        let lo = bus.read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let hi = bus.read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        u16::from_le_bytes([lo, hi])
    }

    /// Reads an 8-bit operand by its 3-bit register code (6 = `(HL)`).
    #[inline]
    fn read_r8<B: CpuBus>(&mut self, bus: &mut B, code: u8) -> u8 {
        match code & 7 {
            0 => self.regs.b,
            1 => self.regs.c,
            2 => self.regs.d,
            3 => self.regs.e,
            4 => self.regs.h,
            5 => self.regs.l,
            6 => bus.read(self.regs.hl()),
            _ => self.regs.a,
        }
    }

    #[inline]
    fn write_r8<B: CpuBus>(&mut self, bus: &mut B, code: u8, value: u8) {
        match code & 7 {
            0 => self.regs.b = value,
            1 => self.regs.c = value,
            2 => self.regs.d = value,
            3 => self.regs.e = value,
            4 => self.regs.h = value,
            5 => self.regs.l = value,
            6 => bus.write(self.regs.hl(), value),
            _ => self.regs.a = value,
        }
    }

    /// 16-bit register pair by code, group 1 (BC, DE, HL, SP).
    #[inline]
    fn read_r16(&self, code: u8) -> u16 {
        match code & 3 {
            0 => self.regs.bc(),
            1 => self.regs.de(),
            2 => self.regs.hl(),
            _ => self.regs.sp,
        }
    }

    #[inline]
    fn write_r16(&mut self, code: u8, value: u16) {
        match code & 3 {
            0 => self.regs.set_bc(value),
            1 => self.regs.set_de(value),
            2 => self.regs.set_hl(value),
            _ => self.regs.sp = value,
        }
    }

    /// Condition codes NZ, Z, NC, C.
    #[inline]
    fn condition(&self, code: u8) -> bool {
        match code & 3 {
            0 => !self.regs.flag(Z),
            1 => self.regs.flag(Z),
            2 => !self.regs.flag(C),
            _ => self.regs.flag(C),
        }
    }

    fn execute<B: CpuBus>(&mut self, bus: &mut B, op: u8, ime_just_enabled: bool) {
        match op {
            // --- 8-bit loads -------------------------------------------------
            0x40..=0x7F => {
                if op == 0x76 {
                    self.halt(bus, ime_just_enabled);
                } else {
                    let v = self.read_r8(bus, op);
                    self.write_r8(bus, op >> 3, v);
                }
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let v = self.fetch(bus);
                self.write_r8(bus, op >> 3, v);
            }
            0x02 => bus.write(self.regs.bc(), self.regs.a),
            0x12 => bus.write(self.regs.de(), self.regs.a),
            0x22 => {
                let hl = self.regs.hl();
                bus.write(hl, self.regs.a);
                self.regs.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.regs.hl();
                bus.write(hl, self.regs.a);
                self.regs.set_hl(hl.wrapping_sub(1));
            }
            0x0A => self.regs.a = bus.read(self.regs.bc()),
            0x1A => self.regs.a = bus.read(self.regs.de()),
            0x2A => {
                let hl = self.regs.hl();
                self.regs.a = bus.read(hl);
                self.regs.set_hl(hl.wrapping_add(1));
            }
            0x3A => {
                let hl = self.regs.hl();
                self.regs.a = bus.read(hl);
                self.regs.set_hl(hl.wrapping_sub(1));
            }
            0xE0 => {
                let n = self.fetch(bus);
                bus.write(0xFF00 | u16::from(n), self.regs.a);
            }
            0xF0 => {
                let n = self.fetch(bus);
                self.regs.a = bus.read(0xFF00 | u16::from(n));
            }
            0xE2 => bus.write(0xFF00 | u16::from(self.regs.c), self.regs.a),
            0xF2 => self.regs.a = bus.read(0xFF00 | u16::from(self.regs.c)),
            0xEA => {
                let addr = self.fetch16(bus);
                bus.write(addr, self.regs.a);
            }
            0xFA => {
                let addr = self.fetch16(bus);
                self.regs.a = bus.read(addr);
            }

            // --- 16-bit loads ------------------------------------------------
            0x01 | 0x11 | 0x21 | 0x31 => {
                let v = self.fetch16(bus);
                self.write_r16(op >> 4, v);
            }
            0x08 => {
                let addr = self.fetch16(bus);
                let [hi, lo] = self.regs.sp.to_be_bytes();
                bus.write(addr, lo);
                bus.write(addr.wrapping_add(1), hi);
            }
            0xF9 => {
                self.regs.sp = self.regs.hl();
                bus.idle();
            }
            0xF8 => {
                let e = self.fetch(bus);
                let v = self.add_sp_e(e);
                self.regs.set_hl(v);
                bus.idle();
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let v = match op {
                    0xC5 => self.regs.bc(),
                    0xD5 => self.regs.de(),
                    0xE5 => self.regs.hl(),
                    _ => self.regs.af(),
                };
                bus.idle();
                self.push(bus, v);
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let v = self.pop(bus);
                match op {
                    0xC1 => self.regs.set_bc(v),
                    0xD1 => self.regs.set_de(v),
                    0xE1 => self.regs.set_hl(v),
                    _ => self.regs.set_af(v),
                }
            }

            // --- 8-bit arithmetic / logic ------------------------------------
            0x80..=0xBF => {
                let v = self.read_r8(bus, op);
                self.alu(op >> 3, v);
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                let v = self.fetch(bus);
                self.alu(op >> 3, v);
            }
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let v = self.read_r8(bus, op >> 3);
                let r = v.wrapping_add(1);
                self.regs.f = (self.regs.f & C) | if r == 0 { Z } else { 0 } | if v & 0x0F == 0x0F { H } else { 0 };
                self.write_r8(bus, op >> 3, r);
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let v = self.read_r8(bus, op >> 3);
                let r = v.wrapping_sub(1);
                self.regs.f = (self.regs.f & C) | N | if r == 0 { Z } else { 0 } | if v & 0x0F == 0 { H } else { 0 };
                self.write_r8(bus, op >> 3, r);
            }
            0x27 => self.daa(),
            0x2F => {
                self.regs.a = !self.regs.a;
                self.regs.f |= N | H;
            }
            0x37 => self.regs.f = (self.regs.f & Z) | C,
            0x3F => self.regs.f = (self.regs.f & (Z | C)) ^ C,

            // --- 16-bit arithmetic -------------------------------------------
            0x03 | 0x13 | 0x23 | 0x33 => {
                let v = self.read_r16(op >> 4).wrapping_add(1);
                self.write_r16(op >> 4, v);
                bus.idle();
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                let v = self.read_r16(op >> 4).wrapping_sub(1);
                self.write_r16(op >> 4, v);
                bus.idle();
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let hl = self.regs.hl();
                let rr = self.read_r16(op >> 4);
                let (r, carry) = hl.overflowing_add(rr);
                self.regs.f = (self.regs.f & Z)
                    | if (hl & 0x0FFF) + (rr & 0x0FFF) > 0x0FFF { H } else { 0 }
                    | if carry { C } else { 0 };
                self.regs.set_hl(r);
                bus.idle();
            }
            0xE8 => {
                let e = self.fetch(bus);
                self.regs.sp = self.add_sp_e(e);
                bus.idle();
                bus.idle();
            }

            // --- rotates on A ------------------------------------------------
            0x07 => {
                let r = self.rlc(self.regs.a);
                self.regs.a = r;
                self.regs.f &= !Z;
            }
            0x0F => {
                let r = self.rrc(self.regs.a);
                self.regs.a = r;
                self.regs.f &= !Z;
            }
            0x17 => {
                let r = self.rl(self.regs.a);
                self.regs.a = r;
                self.regs.f &= !Z;
            }
            0x1F => {
                let r = self.rr(self.regs.a);
                self.regs.a = r;
                self.regs.f &= !Z;
            }

            // --- jumps, calls, returns ---------------------------------------
            0xC3 => {
                let addr = self.fetch16(bus);
                bus.idle();
                self.regs.pc = addr;
            }
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                let addr = self.fetch16(bus);
                if self.condition(op >> 3) {
                    bus.idle();
                    self.regs.pc = addr;
                }
            }
            0xE9 => self.regs.pc = self.regs.hl(),
            0x18 => {
                let e = self.fetch(bus) as i8;
                bus.idle();
                self.regs.pc = self.regs.pc.wrapping_add_signed(i16::from(e));
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                let e = self.fetch(bus) as i8;
                if self.condition(op >> 3) {
                    bus.idle();
                    self.regs.pc = self.regs.pc.wrapping_add_signed(i16::from(e));
                }
            }
            0xCD => {
                let addr = self.fetch16(bus);
                bus.idle();
                self.push(bus, self.regs.pc);
                self.regs.pc = addr;
            }
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let addr = self.fetch16(bus);
                if self.condition(op >> 3) {
                    bus.idle();
                    self.push(bus, self.regs.pc);
                    self.regs.pc = addr;
                }
            }
            0xC9 => {
                self.regs.pc = self.pop(bus);
                bus.idle();
            }
            0xD9 => {
                self.regs.pc = self.pop(bus);
                bus.idle();
                self.ime = true;
            }
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                bus.idle();
                if self.condition(op >> 3) {
                    self.regs.pc = self.pop(bus);
                    bus.idle();
                }
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                bus.idle();
                self.push(bus, self.regs.pc);
                self.regs.pc = u16::from(op & 0x38);
            }

            // --- control -----------------------------------------------------
            0x00 => {}
            0x10 => {
                // STOP. On DMG this enters a very-low-power state until a button
                // is pressed; the byte after it is conventionally 0x00 and skipped.
                // DIV is reset.
                bus.stop();
                self.power = PowerState::Stopped;
            }
            0xF3 => {
                self.ime = false;
                self.ei_delay = false;
            }
            0xFB => self.ei_delay = true,

            // Illegal opcodes hang the CPU.
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                self.power = PowerState::Locked;
                self.locked_opcode = op;
                self.regs.pc = self.regs.pc.wrapping_sub(1);
            }
            0xCB => unreachable!("CB prefix is handled in step()"),
        }
    }

    fn halt<B: CpuBus>(&mut self, bus: &mut B, ime_just_enabled: bool) {
        let pending = bus.pending_interrupts() != 0;
        if !pending {
            self.power = PowerState::Halted;
        } else if ime_just_enabled {
            // `EI; HALT` with an interrupt already pending: IME was still 0 when
            // HALT executed on hardware, so the halt bug fires, the interrupt is
            // serviced, and the handler returns to the HALT itself.
            self.regs.pc = self.regs.pc.wrapping_sub(1);
        } else if !self.ime {
            // HALT bug: the next opcode fetch does not increment PC.
            self.halt_bug = true;
        }
        // IME=1 with a pending interrupt: HALT exits immediately and the
        // interrupt is dispatched on the next step.
    }

    fn execute_cb<B: CpuBus>(&mut self, bus: &mut B, op: u8) {
        let reg = op & 7;
        let bit = (op >> 3) & 7;
        let v = self.read_r8(bus, reg);
        match op >> 6 {
            0 => {
                let r = match bit {
                    0 => self.rlc(v),
                    1 => self.rrc(v),
                    2 => self.rl(v),
                    3 => self.rr(v),
                    4 => self.sla(v),
                    5 => self.sra(v),
                    6 => self.swap(v),
                    _ => self.srl(v),
                };
                self.write_r8(bus, reg, r);
            }
            1 => {
                // BIT: no write-back, so (HL) costs only the read.
                self.regs.f = (self.regs.f & C) | H | if v & (1 << bit) == 0 { Z } else { 0 };
            }
            2 => self.write_r8(bus, reg, v & !(1 << bit)),
            _ => self.write_r8(bus, reg, v | (1 << bit)),
        }
    }

    // --- ALU helpers --------------------------------------------------------

    /// The eight accumulator operations selected by bits 3..5 of the opcode.
    #[inline]
    fn alu(&mut self, which: u8, v: u8) {
        let a = self.regs.a;
        match which & 7 {
            0 => self.regs.a = self.add8(a, v, false),
            1 => {
                let carry = self.regs.flag(C);
                self.regs.a = self.add8(a, v, carry);
            }
            2 => self.regs.a = self.sub8(a, v, false),
            3 => {
                let carry = self.regs.flag(C);
                self.regs.a = self.sub8(a, v, carry);
            }
            4 => {
                self.regs.a = a & v;
                self.regs.f = H | if self.regs.a == 0 { Z } else { 0 };
            }
            5 => {
                self.regs.a = a ^ v;
                self.regs.f = if self.regs.a == 0 { Z } else { 0 };
            }
            6 => {
                self.regs.a = a | v;
                self.regs.f = if self.regs.a == 0 { Z } else { 0 };
            }
            _ => {
                self.sub8(a, v, false);
            }
        }
    }

    #[inline]
    fn add8(&mut self, a: u8, b: u8, carry: bool) -> u8 {
        let c = u16::from(carry);
        let sum = u16::from(a) + u16::from(b) + c;
        let r = sum as u8;
        self.regs.f = if r == 0 { Z } else { 0 }
            | if (a & 0x0F) + (b & 0x0F) + c as u8 > 0x0F { H } else { 0 }
            | if sum > 0xFF { C } else { 0 };
        r
    }

    #[inline]
    fn sub8(&mut self, a: u8, b: u8, carry: bool) -> u8 {
        let c = i16::from(carry);
        let diff = i16::from(a) - i16::from(b) - c;
        let r = diff as u8;
        self.regs.f = N
            | if r == 0 { Z } else { 0 }
            | if i16::from(a & 0x0F) - i16::from(b & 0x0F) - c < 0 { H } else { 0 }
            | if diff < 0 { C } else { 0 };
        r
    }

    /// SP + signed 8-bit offset, with the flags set from the unsigned
    /// low-byte addition (shared by `ADD SP,e` and `LD HL,SP+e`).
    #[inline]
    fn add_sp_e(&mut self, e: u8) -> u16 {
        let sp = self.regs.sp;
        let low = sp & 0xFF;
        let e16 = u16::from(e);
        self.regs.f = if (low & 0x0F) + (e16 & 0x0F) > 0x0F { H } else { 0 } | if low + e16 > 0xFF { C } else { 0 };
        sp.wrapping_add_signed(i16::from(e as i8))
    }

    fn daa(&mut self) {
        let mut a = self.regs.a;
        let mut carry = self.regs.flag(C);
        if self.regs.flag(N) {
            if self.regs.flag(H) {
                a = a.wrapping_sub(0x06);
            }
            if carry {
                a = a.wrapping_sub(0x60);
            }
        } else {
            if carry || a > 0x99 {
                a = a.wrapping_add(0x60);
                carry = true;
            }
            if self.regs.flag(H) || a & 0x0F > 0x09 {
                a = a.wrapping_add(0x06);
            }
        }
        self.regs.a = a;
        self.regs.f = (self.regs.f & N) | if a == 0 { Z } else { 0 } | if carry { C } else { 0 };
    }

    #[inline]
    fn shift_flags(&mut self, r: u8, carry: bool) {
        self.regs.f = if r == 0 { Z } else { 0 } | if carry { C } else { 0 };
    }

    fn rlc(&mut self, v: u8) -> u8 {
        let r = v.rotate_left(1);
        self.shift_flags(r, v & 0x80 != 0);
        r
    }
    fn rrc(&mut self, v: u8) -> u8 {
        let r = v.rotate_right(1);
        self.shift_flags(r, v & 0x01 != 0);
        r
    }
    fn rl(&mut self, v: u8) -> u8 {
        let r = (v << 1) | u8::from(self.regs.flag(C));
        self.shift_flags(r, v & 0x80 != 0);
        r
    }
    fn rr(&mut self, v: u8) -> u8 {
        let r = (v >> 1) | (u8::from(self.regs.flag(C)) << 7);
        self.shift_flags(r, v & 0x01 != 0);
        r
    }
    fn sla(&mut self, v: u8) -> u8 {
        let r = v << 1;
        self.shift_flags(r, v & 0x80 != 0);
        r
    }
    fn sra(&mut self, v: u8) -> u8 {
        let r = (v >> 1) | (v & 0x80);
        self.shift_flags(r, v & 0x01 != 0);
        r
    }
    fn swap(&mut self, v: u8) -> u8 {
        let r = v.rotate_left(4);
        self.shift_flags(r, false);
        r
    }
    fn srl(&mut self, v: u8) -> u8 {
        let r = v >> 1;
        self.shift_flags(r, v & 0x01 != 0);
        r
    }

    // --- save state support ---------------------------------------------------

    pub(crate) fn save(&self, w: &mut crate::state::StateWriter) {
        let r = &self.regs;
        w.u8s(&[r.a, r.f, r.b, r.c, r.d, r.e, r.h, r.l]);
        w.u16(r.sp);
        w.u16(r.pc);
        w.bool(self.ime);
        w.bool(self.ei_delay);
        w.u8(self.power as u8);
        w.bool(self.halt_bug);
        w.u8(self.locked_opcode);
    }

    pub(crate) fn load(&mut self, r: &mut crate::state::StateReader) -> Result<(), crate::state::StateError> {
        let mut b = [0u8; 8];
        r.u8s(&mut b)?;
        let [a, f, bb, c, d, e, h, l] = b;
        self.regs = Registers { a, f: f & 0xF0, b: bb, c, d, e, h, l, sp: r.u16()?, pc: r.u16()? };
        self.ime = r.bool()?;
        self.ei_delay = r.bool()?;
        self.power = match r.u8()? {
            0 => PowerState::Running,
            1 => PowerState::Halted,
            2 => PowerState::Stopped,
            3 => PowerState::Locked,
            _ => return Err(crate::state::StateError::Corrupt("cpu power state")),
        };
        self.halt_bug = r.bool()?;
        self.locked_opcode = r.u8()?;
        Ok(())
    }
}
