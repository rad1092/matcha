//! `matcha profile`: run ROMs headless with the execution profiler on and
//! emit one JSON record per ROM (the input to `analysis/`).

use matcha_core::profile::REGION_NAMES;
use matcha_core::{Buttons, GameBoy, RunEvent};
use serde_json::{Value, json};
use std::time::Instant;

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

/// Button state for frame `frame` under `mode`.
fn input_for(mode: InputMode, frame: u64, rng: &mut XorShift) -> Buttons {
    match mode {
        InputMode::None => Buttons::NONE,
        InputMode::Monkey => {
            // Tap Start/A every ~2s for the first 10s to get past title screens,
            // then hold a random combination for 8-frame stretches.
            if frame < 600 {
                return match frame % 120 {
                    60..=65 => Buttons::START,
                    90..=95 => Buttons::A,
                    _ => Buttons::NONE,
                };
            }
            if frame % 8 == 0 {
                let r = rng.next();
                // Avoid Select+Start+A+B soft-reset combos and never press opposite directions.
                let mut b = Buttons((r & 0x3F) as u8);
                if b.contains(Buttons::LEFT | Buttons::RIGHT) {
                    b.0 &= !Buttons::LEFT.0;
                }
                if b.contains(Buttons::UP | Buttons::DOWN) {
                    b.0 &= !Buttons::UP.0;
                }
                if r & 0x100 != 0 {
                    b = b | Buttons::START;
                }
                return b;
            }
            Buttons(u8::MAX) // sentinel: keep previous
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
    let seed = rom.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3));
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
    let mut rng = XorShift(seed | 1);
    let mut held = Buttons::NONE;
    let mut lcd_on_frames = 0u64;
    let mut distinct_frames = std::collections::HashSet::new();
    let mut nonblank_frames = 0u64;
    while gb.frame_count() < frames {
        let b = input_for(mode, gb.frame_count(), &mut rng);
        if b != Buttons(u8::MAX) && b != held {
            held = b;
            gb.set_buttons(held);
        }
        if let RunEvent::Breakpoint { .. } = gb.run_frame() {
            unreachable!("no breakpoints are set");
        }
        let fb = gb.framebuffer();
        if gb.ppu_registers()[0] & 0x80 != 0 {
            lcd_on_frames += 1;
        }
        let first = fb[0];
        if fb.iter().any(|&p| p != first) {
            nonblank_frames += 1;
        }
        if gb.frame_count() % 15 == 0 {
            let h = fb.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3));
            distinct_frames.insert(h);
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
    let mode = match input.unwrap_or("monkey") {
        "none" => InputMode::None,
        "monkey" => InputMode::Monkey,
        other => return Err(format!("unknown --input '{other}' (none|monkey)")),
    };
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
