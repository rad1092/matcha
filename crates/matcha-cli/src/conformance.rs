//! Conformance scoreboard: runs the community test-ROM suites (as packaged by
//! c-sp/game-boy-test-roms) and judges each ROM the way its author intended.
//!
//! * Blargg: result signature in cartridge RAM (0xA001 = DE B0 61, status at
//!   0xA000), "Passed"/"Failed" on the serial port, or a reference screenshot.
//! * Mooneye / dmg-acid2 / Mealybug: run to the `LD B,B` software breakpoint,
//!   then check the Fibonacci registers or compare the screen.

use crate::image;
use matcha_core::{GameBoy, RunEvent};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Frames per emulated second (rounded up).
const FPS: u32 = 60;

#[derive(Clone, Debug)]
pub enum Judge {
    /// Blargg-style: run up to `seconds`; RAM signature / serial / screenshot.
    Blargg { seconds: u32, screenshot: Option<PathBuf> },
    /// Run to `LD B,B`; pass if B..L hold 3,5,8,13,21,34.
    Fibonacci,
    /// Run to `LD B,B`; pass if the screen matches the reference image.
    Screenshot(PathBuf),
}

#[derive(Clone, Debug)]
pub struct Case {
    pub suite: &'static str,
    pub name: String,
    pub path: PathBuf,
    pub judge: Judge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail(String),
    /// The ROM could not be run at all (e.g. unsupported mapper).
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub case: Case,
    pub verdict: Verdict,
    pub frames: u64,
    pub millis: u128,
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

fn gb_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(gb_files(&p));
        } else if p.extension().is_some_and(|x| x == "gb") {
            out.push(p);
        }
    }
    out.sort();
    out
}

/// Mooneye naming: a trailing `-XYZ` lists the models a test is valid on.
/// Keep tests with no model suffix, or whose suffix includes DMG (A/B/C) or `G`.
fn mooneye_runs_on_dmg(stem: &str) -> bool {
    let Some((_, suffix)) = stem.rsplit_once('-') else { return true };
    match suffix {
        "dmgABC" | "dmgABCmgb" | "GS" | "dmg" => true,
        s if s.contains("dmg0") || s == "mgb" || s == "S" || s == "sgb" || s == "sgb2" || s == "C" || s == "A" => {
            false
        }
        s if s.chars().all(|c| c.is_ascii_uppercase()) => s.contains('G'),
        _ => true,
    }
}

/// Discovers all DMG-relevant cases under the unpacked release directory.
pub fn discover(root: &Path) -> Vec<Case> {
    let mut cases = Vec::new();
    let b = root.join("blargg");
    let blargg_main: [(&str, u32, Option<&str>); 7] = [
        ("cpu_instrs/cpu_instrs.gb", 60, Some("cpu_instrs/cpu_instrs-dmg-cgb.png")),
        ("instr_timing/instr_timing.gb", 3, Some("instr_timing/instr_timing-dmg-cgb.png")),
        ("mem_timing/mem_timing.gb", 5, Some("mem_timing/mem_timing-dmg-cgb.png")),
        ("mem_timing-2/mem_timing.gb", 6, Some("mem_timing-2/mem_timing-dmg-cgb.png")),
        ("halt_bug.gb", 4, Some("halt_bug-dmg-cgb.png")),
        ("dmg_sound/dmg_sound.gb", 40, Some("dmg_sound/dmg_sound-dmg.png")),
        ("oam_bug/oam_bug.gb", 25, Some("oam_bug/oam_bug-dmg.png")),
    ];
    for (file, seconds, png) in blargg_main {
        let path = b.join(file);
        if path.exists() {
            cases.push(Case {
                suite: "blargg",
                name: rel(root, &path),
                judge: Judge::Blargg { seconds, screenshot: png.map(|p| b.join(p)) },
                path,
            });
        }
    }
    for sub in ["cpu_instrs/individual", "mem_timing/individual", "mem_timing-2/rom_singles", "dmg_sound/rom_singles", "oam_bug/rom_singles"] {
        for path in gb_files(&b.join(sub)) {
            cases.push(Case {
                suite: "blargg",
                name: rel(root, &path),
                judge: Judge::Blargg { seconds: 30, screenshot: None },
                path,
            });
        }
    }

    let m = root.join("mooneye-test-suite");
    for sub in ["acceptance", "emulator-only"] {
        for path in gb_files(&m.join(sub)) {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            if mooneye_runs_on_dmg(&stem) {
                cases.push(Case { suite: "mooneye", name: rel(root, &path), judge: Judge::Fibonacci, path });
            }
        }
    }

    let acid = root.join("dmg-acid2/dmg-acid2.gb");
    if acid.exists() {
        cases.push(Case {
            suite: "dmg-acid2",
            name: rel(root, &acid),
            judge: Judge::Screenshot(root.join("dmg-acid2/dmg-acid2-dmg.png")),
            path: acid,
        });
    }

    for path in gb_files(&root.join("mealybug-tearoom-tests/ppu")) {
        let reference = path.with_file_name(format!("{}_dmg_blob.png", path.file_stem().unwrap().to_string_lossy()));
        if reference.exists() {
            cases.push(Case { suite: "mealybug", name: rel(root, &path), judge: Judge::Screenshot(reference), path });
        }
    }
    cases
}

/// Blargg's result protocol in cartridge RAM: Some(status) once finished.
fn blargg_ram_status(gb: &GameBoy) -> Option<u8> {
    let sig = [gb.peek(0xA001), gb.peek(0xA002), gb.peek(0xA003)];
    let status = gb.peek(0xA000);
    (sig == [0xDE, 0xB0, 0x61] && status != 0x80).then_some(status)
}

fn blargg_ram_text(gb: &GameBoy) -> String {
    let mut s = String::new();
    for addr in 0xA004..0xA800u16 {
        match gb.peek(addr) {
            0 => break,
            c => s.push(c as char),
        }
    }
    s
}

fn serial_text(gb: &GameBoy) -> String {
    String::from_utf8_lossy(gb.serial_output()).into_owned()
}

fn summarize(text: &str) -> String {
    let t: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() > 160 { format!("{}…", t.chars().take(160).collect::<String>()) } else { t }
}

fn run_blargg(gb: &mut GameBoy, seconds: u32, screenshot: Option<&Path>) -> Verdict {
    let max_frames = u64::from(seconds * FPS);
    let mut finished_at = None;
    while gb.frame_count() < max_frames {
        gb.run_frame();
        if finished_at.is_none() && gb.frame_count() % 30 == 0 {
            let serial = serial_text(gb);
            if blargg_ram_status(gb).is_some() || serial.contains("Passed") || serial.contains("Failed") {
                finished_at = Some(gb.frame_count());
            }
        }
        // Let the result text reach the screen before judging.
        if finished_at.is_some_and(|f| gb.frame_count() >= f + 20) {
            break;
        }
    }
    if let Some(status) = blargg_ram_status(gb) {
        return if status == 0 {
            Verdict::Pass
        } else {
            Verdict::Fail(format!("status {status:#04x}: {}", summarize(&blargg_ram_text(gb))))
        };
    }
    let serial = serial_text(gb);
    if serial.contains("Passed") && !serial.contains("Failed") {
        return Verdict::Pass;
    }
    if serial.contains("Failed") {
        return Verdict::Fail(summarize(&serial));
    }
    match screenshot {
        Some(png) => match image::diff_against(gb, png) {
            Ok(0) => Verdict::Pass,
            Ok(n) => Verdict::Fail(format!("{n} pixels differ from reference")),
            Err(e) => Verdict::Error(e),
        },
        None => Verdict::Fail(format!("no result after {seconds}s; serial: {}", summarize(&serial))),
    }
}

/// Runs until the `LD B,B` (0x40) software breakpoint executes.
fn run_to_ld_b_b(gb: &mut GameBoy, max_seconds: u32) -> bool {
    let max_frames = u64::from(max_seconds * FPS);
    while gb.frame_count() < max_frames {
        let pc = gb.registers().pc;
        let is_ld_b_b = gb.peek(pc) == 0x40 && gb.power_state() == matcha_core::cpu::PowerState::Running;
        gb.step();
        if is_ld_b_b {
            return true;
        }
    }
    false
}

fn run_case(case: &Case) -> Outcome {
    let start = Instant::now();
    let rom = match std::fs::read(&case.path) {
        Ok(r) => r,
        Err(e) => {
            return Outcome { case: case.clone(), verdict: Verdict::Error(e.to_string()), frames: 0, millis: 0 };
        }
    };
    let mut gb = match GameBoy::new(rom) {
        Ok(mut gb) => {
            gb.set_audio_output(false);
            gb
        }
        Err(e) => {
            return Outcome { case: case.clone(), verdict: Verdict::Error(e.to_string()), frames: 0, millis: 0 };
        }
    };
    let verdict = match &case.judge {
        Judge::Blargg { seconds, screenshot } => run_blargg(&mut gb, *seconds, screenshot.as_deref()),
        Judge::Fibonacci => {
            if !run_to_ld_b_b(&mut gb, 30) {
                Verdict::Fail("timed out before LD B,B".into())
            } else {
                let r = gb.registers();
                let got = [r.b, r.c, r.d, r.e, r.h, r.l];
                if got == [3, 5, 8, 13, 21, 34] {
                    Verdict::Pass
                } else if got == [0x42; 6] {
                    Verdict::Fail("test reported failure (registers = 0x42)".into())
                } else {
                    Verdict::Fail(format!("registers B-L = {got:02x?}"))
                }
            }
        }
        Judge::Screenshot(reference) => {
            if !run_to_ld_b_b(&mut gb, 10) {
                Verdict::Fail("timed out before LD B,B".into())
            } else {
                // Finish the frame being drawn so the framebuffer is complete.
                while gb.run_frame() != RunEvent::FrameComplete {}
                match image::diff_against(&gb, reference) {
                    Ok(0) => Verdict::Pass,
                    Ok(n) => Verdict::Fail(format!("{n} pixels differ from reference")),
                    Err(e) => Verdict::Error(e),
                }
            }
        }
    };
    Outcome { case: case.clone(), verdict, frames: gb.frame_count(), millis: start.elapsed().as_millis() }
}

/// Runs all cases on `threads` worker threads, preserving input order.
pub fn run_all(cases: &[Case], threads: usize) -> Vec<Outcome> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Outcome>>> = Mutex::new(vec![None; cases.len()]);
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(case) = cases.get(i) else { break };
                    let outcome = run_case(case);
                    results.lock().unwrap()[i] = Some(outcome);
                }
            });
        }
    });
    results.into_inner().unwrap().into_iter().map(|o| o.expect("every case ran")).collect()
}

/// Suites in scoreboard order, with a one-line description.
const SUITES: [(&str, &str); 4] = [
    ("blargg", "CPU, timing, sound and OAM-bug tests by Shay Green"),
    ("mooneye", "Mooneye Test Suite: acceptance + emulator-only (DMG-applicable)"),
    ("dmg-acid2", "PPU rendering torture test"),
    ("mealybug", "Mealybug Tearoom: mid-scanline PPU effects (needs a pixel FIFO)"),
];

pub fn scoreboard_markdown(outcomes: &[Outcome]) -> String {
    let mut md = String::from("| Suite | Passed | Total | |\n|---|---:|---:|---|\n");
    for (suite, desc) in SUITES {
        let of: Vec<_> = outcomes.iter().filter(|o| o.case.suite == suite).collect();
        if of.is_empty() {
            continue;
        }
        let pass = of.iter().filter(|o| o.verdict == Verdict::Pass).count();
        md += &format!("| {suite} | {pass} | {} | {desc} |\n", of.len());
    }
    let total = outcomes.len();
    let pass = outcomes.iter().filter(|o| o.verdict == Verdict::Pass).count();
    md += &format!("| **all** | **{pass}** | **{total}** | |\n");
    let failures: Vec<_> = outcomes.iter().filter(|o| o.verdict != Verdict::Pass).collect();
    if !failures.is_empty() {
        md += "\n<details><summary>Not passing</summary>\n\n| ROM | Result |\n|---|---|\n";
        for o in failures {
            let why = match &o.verdict {
                Verdict::Fail(m) => format!("fail: {m}"),
                Verdict::Error(m) => format!("error: {m}"),
                Verdict::Pass => unreachable!(),
            };
            md += &format!("| `{}` | {} |\n", o.case.name, why.replace('|', "\\|"));
        }
        md += "\n</details>\n";
    }
    md
}

pub fn scoreboard_json(outcomes: &[Outcome]) -> serde_json::Value {
    let rows: Vec<_> = outcomes
        .iter()
        .map(|o| {
            let (status, detail) = match &o.verdict {
                Verdict::Pass => ("pass", String::new()),
                Verdict::Fail(m) => ("fail", m.clone()),
                Verdict::Error(m) => ("error", m.clone()),
            };
            json!({
                "suite": o.case.suite,
                "rom": o.case.name,
                "status": status,
                "detail": detail,
                "frames": o.frames,
                "millis": o.millis,
            })
        })
        .collect();
    json!({ "emulator": "matcha", "version": env!("CARGO_PKG_VERSION"), "results": rows })
}
