//! DIV/TIMA/TMA/TAC — the system counter and the programmable timer.
//!
//! Modelled as the hardware does it (Pan Docs, "Timer obscure behaviour"):
//! a free-running 16-bit counter whose selected bit feeds a falling-edge
//! detector. That single model produces all the famous glitches — DIV writes
//! and TAC writes that tick TIMA, and the one-cycle-late TMA reload.

use crate::Model;
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
    cgb: bool,
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
    /// CGB fast link clock, counter bit 3 (262144 Hz at normal speed).
    pub serial_fast_clock: bool,
}

/// Counter bit selected by TAC's clock-select field.
const TAC_BITS: [u16; 4] = [1 << 9, 1 << 3, 1 << 5, 1 << 7];
const DIV_APU_BIT: u16 = 1 << 12;
const SERIAL_BIT: u16 = 1 << 8;

impl Timer {
    /// Post-boot state for DMG (A/B/C): DIV = 0xAB with the internal phase
    /// the boot ROM leaves behind (mooneye `boot_div-dmgABCmgb`).
    pub fn new() -> Self {
        Self { cgb: false, counter: 0xABCC, tima: 0, tma: 0, tac: 0, reload: Reload::Idle }
    }

    pub fn new_with_model(model: Model) -> Self {
        // The CGB boot divider phase depends on the boot animation/input.
        // Use a deterministic phase; do not apply the DMG-only calibration.
        if model == Model::Cgb { Self { cgb: true, ..Self::power_on() } } else { Self::new() }
    }

    /// Power-on state (used when running a boot ROM).
    pub fn power_on() -> Self {
        Self { cgb: false, counter: 0, tima: 0, tma: 0, tac: 0, reload: Reload::Idle }
    }

    pub fn power_on_with_model(model: Model) -> Self {
        Self { cgb: model == Model::Cgb, ..Self::power_on() }
    }

    /// Counter bit feeding TIMA's edge detector (0 when the timer is off).
    #[inline]
    fn input_mask(&self) -> u16 {
        if self.tac & 0x04 != 0 { TAC_BITS[usize::from(self.tac & 3)] } else { 0 }
    }

    #[inline]
    fn input(&self, counter: u16) -> bool {
        counter & self.input_mask() != 0
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
        self.tick_with_speed(false)
    }

    pub fn tick_with_speed(&mut self, double_speed: bool) -> TimerEvents {
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
        // Counting up, a bit falls exactly when a carry ripples through it.
        let fell = old & !new;
        if fell & self.input_mask() != 0 {
            self.increment_tima();
        }
        ev.div_apu = fell & (DIV_APU_BIT << u8::from(double_speed)) != 0;
        ev.serial_clock = fell & SERIAL_BIT != 0;
        ev.serial_fast_clock = fell & (1 << 3) != 0;
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
        self.write_with_speed(addr, value, false)
    }

    pub fn write_with_speed(&mut self, addr: u16, value: u8, double_speed: bool) -> TimerEvents {
        let mut ev = TimerEvents::default();
        match addr {
            0xFF04 => {
                let old = self.counter;
                if self.input(old) {
                    self.increment_tima();
                }
                ev.div_apu = old & (DIV_APU_BIT << u8::from(double_speed)) != 0;
                ev.serial_clock = old & SERIAL_BIT != 0;
                ev.serial_fast_clock = old & (1 << 3) != 0;
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
                if was && !self.input(self.counter) && (!self.cgb || self.tac & 4 != 0) {
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
        w.bool(self.cgb);
        w.u16(self.counter);
        w.u8s(&[self.tima, self.tma, self.tac, self.reload as u8]);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        if r.bool()? != self.cgb {
            return Err(StateError::Corrupt("timer model"));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apu_divider_uses_bit_13_in_double_speed() {
        for double in [false, true] {
            let mut t = Timer::power_on_with_model(Model::Cgb);
            let mut edges = 0;
            for _ in 0..if double { 8192 } else { 4096 } {
                edges += usize::from(t.tick_with_speed(double).div_apu);
            }
            assert_eq!(edges, 2);
        }
        let mut t = Timer::power_on_with_model(Model::Cgb);
        t.counter = 0x1000;
        assert!(!t.write_with_speed(0xFF04, 0, true).div_apu);
        t.counter = 0x2000;
        assert!(t.write_with_speed(0xFF04, 0, true).div_apu);
    }

    #[test]
    fn disabling_timer_does_not_generate_the_dmg_glitch_on_cgb() {
        // Pan Docs Timer Obscure Behaviour: enable gates the CGB output,
        // after its falling-edge detector, unlike the DMG input gate.
        for model in [Model::Dmg, Model::Cgb] {
            let mut t = Timer::power_on_with_model(model);
            t.counter = 8;
            t.tac = 5;
            t.write(0xFF07, 0);
            assert_eq!(t.tima, u8::from(model == Model::Dmg));
        }
    }
}
