//! SB/SC: the link port, with no cable attached.
//!
//! Internal-clock transfers complete after 8 serial clocks (8192 Hz, derived
//! from the system counter) and shift in 1s, as with nothing plugged in.
//! Every byte sent is also captured, which is how Blargg's test ROMs report.

use crate::Model;
use crate::state::{StateError, StateReader, StateWriter};
use alloc::vec::Vec;

/// Captured output is capped so a chatty ROM cannot grow memory forever.
const MAX_CAPTURE: usize = 64 * 1024;

#[derive(Clone, Debug, Default)]
pub struct Serial {
    cgb: bool,
    sb: u8,
    sc: u8,
    bits_left: u8,
    output: Vec<u8>,
}

impl Serial {
    pub fn new() -> Self {
        Self { cgb: false, sb: 0, sc: 0, bits_left: 0, output: Vec::new() }
    }

    pub fn new_with_model(model: Model) -> Self {
        // Pan Docs post-boot hardware state: CGB SC reads 0x7F.
        Self { cgb: model == Model::Cgb, sc: if model == Model::Cgb { 3 } else { 0 }, ..Self::new() }
    }

    pub fn fast_clock(&self) -> bool {
        self.cgb && self.sc & 2 != 0
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr == 0xFF01 { self.sb } else { self.sc | if self.cgb { 0x7C } else { 0x7E } }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if addr == 0xFF01 {
            self.sb = value;
            return;
        }
        self.sc = value & if self.cgb { 0x83 } else { 0x81 };
        if value & 0x81 == 0x81 {
            self.bits_left = 8;
            if self.output.len() < MAX_CAPTURE {
                self.output.push(self.sb);
            }
        } else {
            self.bits_left = 0;
        }
    }

    /// Called on each falling edge of the serial clock. Returns true when a
    /// transfer completes (serial interrupt).
    #[inline]
    pub fn clock(&mut self) -> bool {
        if self.bits_left == 0 {
            return false;
        }
        self.sb = (self.sb << 1) | 1;
        self.bits_left -= 1;
        if self.bits_left == 0 {
            self.sc &= 0x7F;
            return true;
        }
        false
    }

    pub fn output(&self) -> &[u8] {
        &self.output
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.bool(self.cgb);
        w.u8s(&[self.sb, self.sc, self.bits_left]);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        if r.bool()? != self.cgb {
            return Err(StateError::Corrupt("serial model"));
        }
        let mut b = [0u8; 3];
        r.u8s(&mut b)?;
        if b[2] > 8 || b[1] & !(if self.cgb { 0x83 } else { 0x81 }) != 0 {
            return Err(StateError::Corrupt("serial transfer"));
        }
        [self.sb, self.sc, self.bits_left] = b;
        Ok(())
    }
}
