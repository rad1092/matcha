//! `matcha profile`: run ROMs headless with the execution profiler on and
//! emit one JSON record per ROM (the input to `analysis/`).

use matcha_core::profile::REGION_NAMES;
use matcha_core::{Buttons, GameBoy, RunEvent};
use serde_json::{Value, json};
use std::time::Instant;

/// Frames between samples for `distinct_frames_sampled`.
const SAMPLE_EVERY: u64 = 15;
/// Identical consecutive frames that make a "static screen".
const STATIC_RUN: u32 = 30;
/// At most this many static screens are recorded per ROM.
const MAX_STATIC: usize = 64;

/// 64-bit FNV-1a; seeds the monkey input and fingerprints frames. The
/// SameBoy reference runner (analysis/reference/) uses the same function.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3))
}

/// Deterministic xorshift PRNG so "monkey" input is reproducible per ROM.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// No buttons pressed at all.
    None,
    /// Press Start periodically, then random buttons (a "monkey tester").
    Monkey,
}

impl InputMode {
    pub fn parse(name: Option<&str>, default: Self) -> Result<Self, String> {
        match name {
            None => Ok(default),
            Some("none") => Ok(Self::None),
            Some("monkey") => Ok(Self::Monkey),
            Some(other) => Err(format!("unknown --input '{other}' (none|monkey)")),
        }
    }
}

/// Button state for frame `frame` under `mode`, or `None` to keep the
/// previous buttons.
fn input_for(mode: InputMode, frame: u64, rng: &mut XorShift) -> Option<Buttons> {
    match mode {
        InputMode::None => Some(Buttons::NONE),
        InputMode::Monkey => {
            // First 10 s: tap Start, then A, every 2 s to get past title
            // screens and dialogue.
            if frame < 600 {
                return Some(match frame % 120 {
                    60..=65 => Buttons::START,
                    90..=95 => Buttons::A,
                    _ => Buttons::NONE,
                });
            }
            // Then: a new random chord every 8 frames. At most one direction,
            // A/B at random, Start rarely (it pauses most games), never Select.
            if frame % 8 != 0 {
                return None;
            }
            let r = rng.next();
            let dirs = [Buttons::NONE, Buttons::RIGHT, Buttons::LEFT, Buttons::UP, Buttons::DOWN];
            let mut b = dirs[(r % 5) as usize];
            if r & 0x100 != 0 {
                b = b | Buttons::A;
            }
            if r & 0x200 != 0 {
                b = b | Buttons::B;
            }
            if r >> 16 & 0x1F == 0 {
                b = b | Buttons::START;
            }
            Some(b)
        }
    }
}

/// Feeds a machine the scripted input: call [`InputDriver::drive`] before
/// running; it applies each frame's buttons once, when that frame begins.
pub struct InputDriver {
    mode: InputMode,
    rng: XorShift,
    held: Buttons,
    frame: Option<u64>,
}

impl InputDriver {
    /// `rom` seeds the random input, so every ROM gets its own sequence.
    pub fn new(mode: InputMode, rom: &[u8]) -> Self {
        Self { mode, rng: XorShift(fnv1a(rom) | 1), held: Buttons::NONE, frame: None }
    }

    pub fn drive(&mut self, gb: &mut GameBoy) {
        let frame = gb.frame_count();
        if self.frame == Some(frame) {
            return;
        }
        self.frame = Some(frame);
        if let Some(b) = input_for(self.mode, frame, &mut self.rng) {
            if b != self.held {
                self.held = b;
                gb.set_buttons(b);
            }
        }
    }
}

/// Profiles one ROM; returns a JSON record (never fails: errors are recorded).
pub fn profile_rom(path: &str, seconds: f64, mode: InputMode) -> Value {
    let started = Instant::now();
    let rom = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => return json!({ "rom": path, "error": e.to_string() }),
    };
    let rom_len = rom.len();
    let mut input = InputDriver::new(mode, &rom);
    let mut gb = match GameBoy::new(rom) {
        Ok(gb) => gb,
        Err(e) => {
            return json!({
                "rom": path,
                "rom_bytes": rom_len,
                "error": e.to_string(),
            });
        }
    };
    let header = gb.header().clone();
    gb.set_audio_output(false);
    gb.enable_profiling();
    let frames = (seconds * matcha_core::FRAME_RATE).round() as u64;
    let mut lcd_on_frames = 0u64;
    let mut nonblank_frames = 0u64;
    let mut distinct_frames = std::collections::HashSet::new();
    // Screens that stay identical for STATIC_RUN frames (titles, menus, text):
    // comparable across emulators regardless of small timing offsets.
    let mut static_screens: Vec<u64> = Vec::new();
    let (mut last_hash, mut run) = (0u64, 0u32);
    while gb.frame_count() < frames {
        let frame = gb.frame_count();
        input.drive(&mut gb);
        match gb.run_frame() {
            RunEvent::FrameComplete | RunEvent::CycleBudget => {}
            RunEvent::Breakpoint { .. } | RunEvent::Watchpoint { .. } => unreachable!("none are set"),
        }
        if gb.frame_count() == frame {
            continue; // no frame completed (CPU in STOP with the LCD off)
        }
        let fb = gb.framebuffer();
        if gb.ppu_registers()[0] & 0x80 != 0 {
            lcd_on_frames += 1;
        }
        if fb.iter().any(|&p| p != fb[0]) {
            nonblank_frames += 1;
        }
        let h = fnv1a(fb);
        if gb.frame_count() % SAMPLE_EVERY == 0 {
            distinct_frames.insert(h);
        }
        if h == last_hash {
            run += 1;
            if run == STATIC_RUN && static_screens.len() < MAX_STATIC && !static_screens.contains(&h) {
                static_screens.push(h);
            }
        } else {
            (last_hash, run) = (h, 1);
        }
    }
    let p = gb.profile().expect("profiling enabled").clone();
    let opcodes: Vec<u64> = p.opcodes.to_vec();
    let cb: Vec<u64> = p.cb_opcodes.to_vec();
    let reads: serde_json::Map<String, Value> =
        REGION_NAMES.iter().zip(p.reads.iter()).map(|(n, v)| ((*n).to_string(), json!(v))).collect();
    let writes: serde_json::Map<String, Value> =
        REGION_NAMES.iter().zip(p.writes.iter()).map(|(n, v)| ((*n).to_string(), json!(v))).collect();
    json!({
        "rom": path,
        "rom_bytes": rom_len,
        "title": header.title,
        "cart_type": header.cart_type,
        "cart_type_name": header.cart_type_name(),
        "cgb_flag": header.cgb_flag,
        "sgb_flag": header.sgb_flag,
        "header_checksum_ok": header.header_checksum_ok,
        "input": if mode == InputMode::Monkey { "monkey" } else { "none" },
        "frames": gb.frame_count(),
        "total_cycles": p.total_cycles(),
        "busy_cycles": p.busy_cycles,
        "halted_cycles": p.halted_cycles,
        "stopped_cycles": p.stopped_cycles,
        "locked_cycles": p.locked_cycles,
        "interrupt_cycles": p.interrupt_cycles,
        "cpu_utilization": p.cpu_utilization(),
        "instructions": p.instructions,
        "interrupts": { "vblank": p.interrupts[0], "stat": p.interrupts[1], "timer": p.interrupts[2], "serial": p.interrupts[3], "joypad": p.interrupts[4] },
        "reads": reads,
        "writes": writes,
        "opcodes": opcodes,
        "cb_opcodes": cb,
        "covered_rom_bytes": p.covered_bytes(),
        "locked_opcode": p.locked_opcode,
        "lcd_on_frames": lcd_on_frames,
        "nonblank_frames": nonblank_frames,
        "distinct_frames_sampled": distinct_frames.len(),
        "static_screens": static_screens.iter().map(|h| format!("{h:016x}")).collect::<Vec<_>>(),
        "wall_ms": started.elapsed().as_millis() as u64,
    })
}

pub fn cmd_profile(
    roms: &[String],
    json_out: Option<&str>,
    input: Option<&str>,
    seconds: Result<f64, String>,
) -> Result<bool, String> {
    let seconds = seconds?;
    if roms.is_empty() {
        return Err("profile: give at least one ROM".into());
    }
    let mode = InputMode::parse(input, InputMode::Monkey)?;
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results = std::sync::Mutex::new(vec![Value::Null; roms.len()]);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(rom) = roms.get(i) else { break };
                    let rec = profile_rom(rom, seconds, mode);
                    results.lock().unwrap()[i] = rec;
                }
            });
        }
    });
    let results = results.into_inner().unwrap();
    for r in &results {
        if let Some(e) = r.get("error") {
            eprintln!("{}: {}", r["rom"].as_str().unwrap_or("?"), e);
        } else {
            eprintln!(
                "{:<48} util {:>5.1}%  {:>9} instr  {:>4} frames",
                r["title"].as_str().unwrap_or(""),
                r["cpu_utilization"].as_f64().unwrap_or(0.0) * 100.0,
                r["instructions"],
                r["frames"]
            );
        }
    }
    let out = serde_json::to_string(&results).unwrap();
    match json_out {
        Some(path) => std::fs::write(path, out).map_err(|e| format!("{path}: {e}"))?,
        None => println!("{out}"),
    }
    Ok(true)
}
