//! Save-state serialization.
//!
//! A deliberately boring, hand-rolled binary format (ADR-0006): little-endian
//! scalars written in a fixed order by each component's `save`/`load` pair,
//! framed by a header that pins the format version and the ROM identity.
//! No serde, no reflection: the order in `save` *is* the schema, and the
//! round-trip tests in `lib.rs` keep `save` and `load` in lock-step.

use alloc::vec::Vec;
use core::fmt;

/// File magic: "MTCHST" + format version.
const MAGIC: &[u8; 6] = b"MTCHST";
/// Bump whenever any component's save layout changes.
pub const STATE_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateError {
    /// Not a matcha save state.
    BadMagic,
    /// Written by an incompatible version of the emulator.
    Version { found: u16, expected: u16 },
    /// The state belongs to a different ROM.
    WrongRom,
    /// The data ended early.
    Truncated,
    /// A field held an impossible value.
    Corrupt(&'static str),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => f.write_str("not a matcha save state"),
            Self::Version { found, expected } => {
                write!(f, "save state version {found} is not supported (expected {expected})")
            }
            Self::WrongRom => f.write_str("save state was made with a different ROM"),
            Self::Truncated => f.write_str("save state is truncated"),
            Self::Corrupt(what) => write!(f, "save state is corrupt ({what})"),
        }
    }
}

#[derive(Default)]
pub struct StateWriter {
    buf: Vec<u8>,
}

impl StateWriter {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(64 * 1024) }
    }

    pub(crate) fn header(&mut self, rom_id: u32) {
        self.buf.extend_from_slice(MAGIC);
        self.u16(STATE_VERSION);
        self.u32(rom_id);
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    pub fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }
    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    /// Fixed-size byte block (length is implied by the reader).
    pub fn u8s(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }
    /// Variable-size byte block, length-prefixed.
    pub fn bytes(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.buf.extend_from_slice(v);
    }
}

pub struct StateReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> StateReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub(crate) fn header(&mut self, rom_id: u32) -> Result<(), StateError> {
        let mut magic = [0u8; 6];
        self.u8s(&mut magic).map_err(|_| StateError::BadMagic)?;
        if &magic != MAGIC {
            return Err(StateError::BadMagic);
        }
        let version = self.u16()?;
        if version != STATE_VERSION {
            return Err(StateError::Version { found: version, expected: STATE_VERSION });
        }
        if self.u32()? != rom_id {
            return Err(StateError::WrongRom);
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.pos == self.data.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], StateError> {
        let end = self.pos.checked_add(n).ok_or(StateError::Truncated)?;
        let s = self.data.get(self.pos..end).ok_or(StateError::Truncated)?;
        self.pos = end;
        Ok(s)
    }

    pub fn u8(&mut self) -> Result<u8, StateError> {
        Ok(self.take(1)?[0])
    }
    pub fn bool(&mut self) -> Result<bool, StateError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(StateError::Corrupt("bool")),
        }
    }
    pub fn u16(&mut self) -> Result<u16, StateError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn u32(&mut self) -> Result<u32, StateError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn u64(&mut self) -> Result<u64, StateError> {
        let b = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(u64::from_le_bytes(a))
    }
    pub fn i32(&mut self) -> Result<i32, StateError> {
        Ok(self.u32()? as i32)
    }
    pub fn f32(&mut self) -> Result<f32, StateError> {
        Ok(f32::from_bits(self.u32()?))
    }
    pub fn u8s(&mut self, out: &mut [u8]) -> Result<(), StateError> {
        let s = self.take(out.len())?;
        out.copy_from_slice(s);
        Ok(())
    }
    /// Reads a length-prefixed block whose length must equal `out.len()`.
    pub fn bytes_exact(&mut self, out: &mut [u8]) -> Result<(), StateError> {
        let len = self.u32()? as usize;
        if len != out.len() {
            return Err(StateError::Corrupt("block length"));
        }
        self.u8s(out)
    }
    /// Reads a length-prefixed block of any length (bounded by `max`).
    pub fn bytes_vec(&mut self, max: usize) -> Result<Vec<u8>, StateError> {
        let len = self.u32()? as usize;
        if len > max {
            return Err(StateError::Corrupt("block too large"));
        }
        Ok(self.take(len)?.to_vec())
    }
}
