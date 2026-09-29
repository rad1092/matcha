//! P1/JOYP: the button matrix.

use crate::state::{StateError, StateReader, StateWriter};

/// Button bitmask used by the public API (1 = pressed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Buttons(pub u8);

impl Buttons {
    pub const RIGHT: Self = Self(1 << 0);
    pub const LEFT: Self = Self(1 << 1);
    pub const UP: Self = Self(1 << 2);
    pub const DOWN: Self = Self(1 << 3);
    pub const A: Self = Self(1 << 4);
    pub const B: Self = Self(1 << 5);
    pub const SELECT: Self = Self(1 << 6);
    pub const START: Self = Self(1 << 7);
    pub const NONE: Self = Self(0);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Parses a button name ("a", "b", "start", "select", "up", ...).
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "right" => Self::RIGHT,
            "left" => Self::LEFT,
            "up" => Self::UP,
            "down" => Self::DOWN,
            "a" => Self::A,
            "b" => Self::B,
            "select" => Self::SELECT,
            "start" => Self::START,
            _ => return None,
        })
    }
}

impl core::ops::BitOr for Buttons {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

#[derive(Clone, Debug)]
pub struct Joypad {
    /// Bits 4–5 of P1 as last written (0 = row selected).
    select: u8,
    pressed: Buttons,
    /// Low nibble last seen by the edge detector.
    last_lines: u8,
}

impl Joypad {
    pub fn new() -> Self {
        // Post-boot P1 reads 0xCF: both rows selected, nothing pressed.
        Self { select: 0x00, pressed: Buttons::NONE, last_lines: 0x0F }
    }

    /// Current state of input lines P10–P13 (0 = low / pressed).
    fn lines(&self) -> u8 {
        let mut lines = 0x0F;
        if self.select & 0x10 == 0 {
            lines &= !(self.pressed.0 & 0x0F);
        }
        if self.select & 0x20 == 0 {
            lines &= !(self.pressed.0 >> 4);
        }
        lines
    }

    /// True if a pressed button's row is selected (a P10–P13 line is low):
    /// the condition that ends STOP mode.
    pub fn any_line_low(&self) -> bool {
        self.lines() != 0x0F
    }

    pub fn read(&self) -> u8 {
        0xC0 | self.select | self.lines()
    }

    /// Returns true if the write caused a joypad interrupt.
    pub fn write(&mut self, value: u8) -> bool {
        self.select = value & 0x30;
        self.update()
    }

    /// Returns true if the change caused a joypad interrupt.
    pub fn set_pressed(&mut self, buttons: Buttons) -> bool {
        self.pressed = buttons;
        self.update()
    }

    pub fn pressed(&self) -> Buttons {
        self.pressed
    }

    /// Interrupt on any high-to-low transition of P10–P13.
    fn update(&mut self) -> bool {
        let lines = self.lines();
        let fell = self.last_lines & !lines;
        self.last_lines = lines;
        fell != 0
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.u8s(&[self.select, self.pressed.0, self.last_lines]);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        let mut b = [0u8; 3];
        r.u8s(&mut b)?;
        self.select = b[0] & 0x30;
        self.pressed = Buttons(b[1]);
        self.last_lines = b[2] & 0x0F;
        Ok(())
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}
