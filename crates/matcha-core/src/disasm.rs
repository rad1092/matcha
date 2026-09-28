//! SM83 disassembler (RGBDS syntax).

use alloc::format;
use alloc::string::String;

/// A decoded instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub addr: u16,
    /// Length in bytes (1–3).
    pub len: u8,
    pub bytes: [u8; 3],
    /// RGBDS-style text, e.g. `ld a, [hl+]`.
    pub text: String,
    /// Absolute branch target for jumps/calls, if statically known.
    pub target: Option<u16>,
}

const R8: [&str; 8] = ["b", "c", "d", "e", "h", "l", "[hl]", "a"];
const R16: [&str; 4] = ["bc", "de", "hl", "sp"];
const R16_STACK: [&str; 4] = ["bc", "de", "hl", "af"];
const COND: [&str; 4] = ["nz", "z", "nc", "c"];
const ALU: [&str; 8] = ["add a,", "adc a,", "sub", "sbc a,", "and", "xor", "or", "cp"];
const CB_OPS: [&str; 8] = ["rlc", "rrc", "rl", "rr", "sla", "sra", "swap", "srl"];

/// Instruction length for an opcode.
pub fn length(op: u8) -> u8 {
    match op {
        0xCB => 2,
        0x01 | 0x11 | 0x21 | 0x31 | 0x08 | 0xC2 | 0xC3 | 0xC4 | 0xCA | 0xCC | 0xCD | 0xD2 | 0xD4 | 0xDA | 0xDC
        | 0xEA | 0xFA => 3,
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xC6 | 0xCE
        | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE | 0xE0 | 0xF0 | 0xE8 | 0xF8 | 0x10 => 2,
        _ => 1,
    }
}

/// Decodes the instruction at `addr` whose bytes are `bytes` (pad with 0).
pub fn decode(addr: u16, bytes: [u8; 3]) -> Instruction {
    let op = bytes[0];
    let len = length(op);
    let n8 = bytes[1];
    let n16 = u16::from_le_bytes([bytes[1], bytes[2]]);
    let rel = addr.wrapping_add(2).wrapping_add_signed(i16::from(n8 as i8));
    let mut target = None;
    let text = match op {
        0x00 => "nop".into(),
        0x10 => "stop".into(),
        0x76 => "halt".into(),
        0xF3 => "di".into(),
        0xFB => "ei".into(),
        0x07 => "rlca".into(),
        0x0F => "rrca".into(),
        0x17 => "rla".into(),
        0x1F => "rra".into(),
        0x27 => "daa".into(),
        0x2F => "cpl".into(),
        0x37 => "scf".into(),
        0x3F => "ccf".into(),
        0x40..=0x7F => format!("ld {}, {}", R8[usize::from((op >> 3) & 7)], R8[usize::from(op & 7)]),
        0x80..=0xBF => format!("{} {}", ALU[usize::from((op >> 3) & 7)], R8[usize::from(op & 7)]),
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            format!("ld {}, ${n8:02x}", R8[usize::from(op >> 3)])
        }
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => format!("inc {}", R8[usize::from(op >> 3)]),
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => format!("dec {}", R8[usize::from(op >> 3)]),
        0x01 | 0x11 | 0x21 | 0x31 => format!("ld {}, ${n16:04x}", R16[usize::from(op >> 4)]),
        0x03 | 0x13 | 0x23 | 0x33 => format!("inc {}", R16[usize::from(op >> 4)]),
        0x0B | 0x1B | 0x2B | 0x3B => format!("dec {}", R16[usize::from(op >> 4)]),
        0x09 | 0x19 | 0x29 | 0x39 => format!("add hl, {}", R16[usize::from(op >> 4)]),
        0x02 => "ld [bc], a".into(),
        0x12 => "ld [de], a".into(),
        0x22 => "ld [hl+], a".into(),
        0x32 => "ld [hl-], a".into(),
        0x0A => "ld a, [bc]".into(),
        0x1A => "ld a, [de]".into(),
        0x2A => "ld a, [hl+]".into(),
        0x3A => "ld a, [hl-]".into(),
        0x08 => format!("ld [${n16:04x}], sp"),
        0x18 => {
            target = Some(rel);
            format!("jr ${rel:04x}")
        }
        0x20 | 0x28 | 0x30 | 0x38 => {
            target = Some(rel);
            format!("jr {}, ${rel:04x}", COND[usize::from((op >> 3) & 3)])
        }
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
            format!("{} ${n8:02x}", ALU[usize::from((op >> 3) & 7)])
        }
        0xC0 | 0xC8 | 0xD0 | 0xD8 => format!("ret {}", COND[usize::from((op >> 3) & 3)]),
        0xC9 => "ret".into(),
        0xD9 => "reti".into(),
        0xC1 | 0xD1 | 0xE1 | 0xF1 => format!("pop {}", R16_STACK[usize::from((op >> 4) & 3)]),
        0xC5 | 0xD5 | 0xE5 | 0xF5 => format!("push {}", R16_STACK[usize::from((op >> 4) & 3)]),
        0xC2 | 0xCA | 0xD2 | 0xDA => {
            target = Some(n16);
            format!("jp {}, ${n16:04x}", COND[usize::from((op >> 3) & 3)])
        }
        0xC3 => {
            target = Some(n16);
            format!("jp ${n16:04x}")
        }
        0xE9 => "jp hl".into(),
        0xC4 | 0xCC | 0xD4 | 0xDC => {
            target = Some(n16);
            format!("call {}, ${n16:04x}", COND[usize::from((op >> 3) & 3)])
        }
        0xCD => {
            target = Some(n16);
            format!("call ${n16:04x}")
        }
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            target = Some(u16::from(op & 0x38));
            format!("rst ${:02x}", op & 0x38)
        }
        0xE0 => format!("ldh [${:04x}], a", 0xFF00 | u16::from(n8)),
        0xF0 => format!("ldh a, [${:04x}]", 0xFF00 | u16::from(n8)),
        0xE2 => "ldh [c], a".into(),
        0xF2 => "ldh a, [c]".into(),
        0xEA => format!("ld [${n16:04x}], a"),
        0xFA => format!("ld a, [${n16:04x}]"),
        0xE8 => format!("add sp, {}", n8 as i8),
        0xF8 => format!("ld hl, sp{:+}", n8 as i8),
        0xF9 => "ld sp, hl".into(),
        0xCB => {
            let cb = n8;
            let reg = R8[usize::from(cb & 7)];
            let bit = (cb >> 3) & 7;
            match cb >> 6 {
                0 => format!("{} {reg}", CB_OPS[usize::from(bit)]),
                1 => format!("bit {bit}, {reg}"),
                2 => format!("res {bit}, {reg}"),
                _ => format!("set {bit}, {reg}"),
            }
        }
        _ => format!("db ${op:02x} ; illegal"),
    };
    Instruction { addr, len, bytes, text, target }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_common_instructions() {
        assert_eq!(decode(0x100, [0x00, 0, 0]).text, "nop");
        assert_eq!(decode(0x100, [0xC3, 0x50, 0x01]).text, "jp $0150");
        assert_eq!(decode(0x150, [0x20, 0xFE, 0]).text, "jr nz, $0150");
        assert_eq!(decode(0x100, [0xE0, 0x44, 0]).text, "ldh [$ff44], a");
        assert_eq!(decode(0x100, [0xCB, 0x7C, 0]).text, "bit 7, h");
        assert_eq!(decode(0x100, [0x36, 0x12, 0]).text, "ld [hl], $12");
        assert_eq!(decode(0x100, [0xF8, 0xFE, 0]).text, "ld hl, sp-2");
        assert_eq!(decode(0x100, [0xD3, 0, 0]).text, "db $d3 ; illegal");
    }

    #[test]
    fn lengths_match_operand_usage() {
        for op in 0..=255u8 {
            let l = length(op);
            assert!((1..=3).contains(&l), "opcode {op:#04x}");
        }
        assert_eq!(length(0xCD), 3);
        assert_eq!(length(0xCB), 2);
        assert_eq!(length(0xE0), 2);
    }
}
