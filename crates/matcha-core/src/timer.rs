//! DIV/TIMA/TMA/TAC — the system counter and the programmable timer.
//!
//! Modelled as the hardware does it (Pan Docs, "Timer obscure behaviour"):
//! a free-running 16-bit counter whose selected bit feeds a falling-edge
//! detector. That single model produces all the famous glitches — DIV writes
//! and TAC writes that tick TIMA, and the one-cycle-late TMA reload.

use crate::state::{StateError, StateReader, StateWriter};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
enum Reload {
    #[default]
    Idle = 0,
    /// TIMA overflowed during the previous M-cycle and reads as 0 ("cycle A").
    /// A CPU write to TIMA now cancels the reload and the interrupt.
    Pending = 1,
    /// TMA was just copied into TIMA ("cycle B"): TIMA writes are ignored and
    /// TMA writes also land in TIMA.
    Reloaded = 2,
}

#[derive(Clone, Debug)]
pub struct Timer {
    /// System counter in T-cycles; DIV is the upper 8 bits.
    counter: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    reload: Reload,
}

/// Events the timer produced during one M-cycle.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimerEvents {
    pub interrupt: bool,
    /// Falling edge of DIV bit 4: clocks the APU frame sequencer (512 Hz).
    pub div_apu: bool,
    /// Falling edge of counter bit 8: shifts one serial bit (8192 Hz).
    pub serial_clock: bool,
}

/// Counter bit selected by TAC's clock-select field.
const TAC_BITS: [u16; 4] = [1 << 9, 1 << 3, 1 << 5, 1 << 7];
const DIV_APU_BIT: u16 = 1 << 12;
const SERIAL_BIT: u16 = 1 << 8;

impl Timer {
    /// Post-boot state for DMG (A/B/C): DIV = 0xAB with the internal phase
    /// the boot ROM leaves behind (mooneye `boot_div-dmgABCmgb`).
    pub fn new() -> Self {
        Self { counter: 0xABCC, tima: 0, tma: 0, tac: 0, reload: Reload::Idle }
    }

    /// Power-on state (used when running a boot ROM).
    pub fn power_on() -> Self {
        Self { counter: 0, tima: 0, tma: 0, tac: 0, reload: Reload::Idle }
    }

    #[inline]
    fn input(&self, counter: u16) -> bool {
        self.tac & 0x04 != 0 && counter & TAC_BITS[usize::from(self.tac & 3)] != 0
    }

    #[inline]
    fn increment_tima(&mut self) {
        let (v, overflow) = self.tima.overflowing_add(1);
        self.tima = v;
        if overflow {
            self.reload = Reload::Pending;
        }
    }

    /// Advances one M-cycle (4 T-cycles).
    #[inline]
    pub fn tick(&mut self) -> TimerEvents {
        let mut ev = TimerEvents::default();
        match self.reload {
            Reload::Idle => {}
            Reload::Reloaded => self.reload = Reload::Idle,
            Reload::Pending => {
                self.tima = self.tma;
                self.reload = Reload::Reloaded;
                ev.interrupt = true;
            }
        }
        let old = self.counter;
        let new = old.wrapping_add(4);
        self.counter = new;
        if self.input(old) && !self.input(new) {
            self.increment_tima();
        }
        let fell = old & !new;
        ev.div_apu = fell & DIV_APU_BIT != 0;
        ev.serial_clock = fell & SERIAL_BIT != 0;
        ev
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.counter >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            _ => self.tac | 0xF8,
        }
    }

    /// Returns events caused by the write itself (DIV reset glitches).
    pub fn write(&mut self, addr: u16, value: u8) -> TimerEvents {
        let mut ev = TimerEvents::default();
        match addr {
            0xFF04 => {
                let old = self.counter;
                if self.input(old) {
                    self.increment_tima();
                }
                ev.div_apu = old & DIV_APU_BIT != 0;
                ev.serial_clock = old & SERIAL_BIT != 0;
                self.counter = 0;
            }
            0xFF05 => match self.reload {
                Reload::Pending => {
                    self.tima = value;
                    self.reload = Reload::Idle;
                }
                Reload::Reloaded => {}
                Reload::Idle => self.tima = value,
            },
            0xFF06 => {
                self.tma = value;
                if self.reload == Reload::Reloaded {
                    self.tima = value;
                }
            }
            _ => {
                let was = self.input(self.counter);
                self.tac = value & 0x07;
                // DMG: a 1 -> 0 transition of the multiplexed input ticks TIMA.
                if was && !self.input(self.counter) {
                    self.increment_tima();
                }
            }
        }
        ev
    }

    /// STOP resets the divider.
    pub fn reset_div(&mut self) {
        self.counter = 0;
    }

    /// The raw 16-bit system counter (for debuggers).
    pub fn system_counter(&self) -> u16 {
        self.counter
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.u16(self.counter);
        w.u8s(&[self.tima, self.tma, self.tac, self.reload as u8]);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        self.counter = r.u16()?;
        let mut b = [0u8; 4];
        r.u8s(&mut b)?;
        [self.tima, self.tma, self.tac] = [b[0], b[1], b[2] & 0x07];
        self.reload = match b[3] {
            0 => Reload::Idle,
            1 => Reload::Pending,
            2 => Reload::Reloaded,
            _ => return Err(StateError::Corrupt("timer reload state")),
        };
        Ok(())
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
