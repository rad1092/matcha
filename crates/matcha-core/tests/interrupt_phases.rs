//! DMG interrupt-entry bus phases, independent of the PPU implementation.
//!
//! SameBoy's interrupt path in Core/sm83_cpu.c writes the PC high/low bytes
//! at dots 12/16 of its 20-dot entry sequence. A pending interrupt observed
//! during the HALT opcode fetch returns to that HALT after service.

use matcha_core::cpu::{Cpu, CpuBus, StepKind};

struct Bus {
    memory: Vec<u8>,
    cycles: u64,
    request: u8,
    request_at: Option<u64>,
    writes: Vec<(u64, u16, u8)>,
    sample_at_start: bool,
}

impl Bus {
    fn new() -> Self {
        Self {
            memory: vec![0; 65536],
            cycles: 0,
            request: 0,
            request_at: None,
            writes: Vec::new(),
            sample_at_start: false,
        }
    }

    fn advance(&mut self) {
        self.cycles += 1;
        if self.request_at == Some(self.cycles) {
            self.request = 1;
        }
    }
}

impl CpuBus for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        let value = self.memory[usize::from(addr)];
        self.advance();
        value
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.writes.push((self.cycles * 4, addr, value));
        self.memory[usize::from(addr)] = value;
        self.advance();
    }

    fn idle(&mut self) {
        self.advance();
    }

    fn pending_interrupts(&self) -> u8 {
        self.request
    }

    fn halt_samples_at_start(&self) -> bool {
        self.sample_at_start
    }

    fn acknowledge_interrupt(&mut self, mask: u8) {
        self.request &= !mask;
    }
}

#[test]
fn interrupt_stack_writes_land_at_dots_12_and_16() {
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x1234;
    cpu.regs.sp = 0xFFFE;
    cpu.ime = true;
    let mut bus = Bus::new();
    bus.request = 1;

    assert_eq!(cpu.step(&mut bus).kind, StepKind::Interrupt { vector: 0x40 });
    assert_eq!(bus.writes, [(12, 0xFFFD, 0x12), (16, 0xFFFC, 0x34)]);
    assert_eq!(bus.cycles * 4, 20);
}

#[test]
fn interrupt_arriving_during_halt_fetch_returns_to_halt() {
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x0100;
    cpu.regs.sp = 0xFFFE;
    cpu.ime = true;
    let mut bus = Bus::new();
    bus.memory[0x0100] = 0x76; // HALT
    bus.memory[0x0040] = 0xD9; // RETI
    bus.request_at = Some(1);

    cpu.step(&mut bus); // request arrives during the opcode fetch
    assert_eq!(cpu.step(&mut bus).kind, StepKind::Interrupt { vector: 0x40 });
    cpu.step(&mut bus); // RETI
    assert_eq!(cpu.regs.pc, 0x0100);
    cpu.step(&mut bus);
    assert!(cpu.is_halted(), "the reexecuted HALT now waits without a pending request");
}

#[test]
fn cgb_halt_samples_before_the_idle_cycle() {
    for cgb in [false, true] {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        bus.sample_at_start = cgb;
        bus.memory[0] = 0x76;
        cpu.step(&mut bus); // HALT fetch
        cpu.step(&mut bus); // initial idle cycle, shared by both models
        bus.request_at = Some(3);
        cpu.step(&mut bus); // edge occurs during the idle cycle
        assert_eq!(cpu.is_halted(), cgb);
        if cgb {
            cpu.step(&mut bus);
            assert!(!cpu.is_halted());
            assert_eq!(bus.cycles, 4);
        } else {
            assert_eq!(bus.cycles, 3);
        }
    }
}
