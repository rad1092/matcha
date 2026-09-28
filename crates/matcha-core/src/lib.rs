//! matcha-core: a cycle-accurate Game Boy (DMG) emulator core.
//!
//! * `no_std` + `alloc`, zero dependencies, `#![forbid(unsafe_code)]`.
//! * Deterministic: the same ROM, inputs and save state always produce the
//!   same frames and audio, bit for bit.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod cpu;
pub mod state;
