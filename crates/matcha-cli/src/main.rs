//! `matcha` — headless runner, conformance scoreboard and profiler.

mod conformance;
mod image;
mod profile;

use matcha_core::cpu::StepKind;
use matcha_core::{Buttons, GameBoy, Model, Options, PowerOnRam, palettes};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
matcha — a Game Boy / Game Boy Color emulator you can see inside

USAGE:
  matcha info <rom>
  matcha run <rom> [--seconds N | --frames N] [--hold BUTTONS | --input none|monkey] [--audio]
                   [--screenshot out.png] [--scale N] [--palette grey|matcha|dmg] [--serial]
                   [--sav file.sav] [--ram zero|noise[:SEED]]
  matcha test <test-roms-dir> [--suite NAME] [--filter TEXT] [--threads N]
                   [--markdown out.md] [--json out.json] [--baseline old.json]
  matcha profile <rom>... [--seconds N] [--input none|monkey] [--ram zero|noise[:SEED]] [--json out.json]
  matcha disasm <rom> [--addr HEX] [--count N]
  matcha trace <rom> [--input none|monkey] [--ram zero|noise[:SEED]] [--frames N] [--skip N]
                   [--count N | --last N [--watch HEX]]

BUTTONS is a comma list: a,b,start,select,up,down,left,right
--model auto|dmg|cgb selects hardware (ROM commands default auto; test defaults dmg).
auto uses the cartridge's CGB header flag. test accepts dmg or cgb explicitly.
--ram noise fills RAM at power-on with DMG-like junk (seed 0 unless given)
instead of zeros, to catch software that reads memory before writing it.

`test` exits non-zero if any ROM fails; with --baseline (a previous --json
scoreboard) it exits non-zero only if a ROM that passed there fails now.
";

/// Minimal flag parser: positional args plus `--name value` / `--switch`.
struct Args {
    positional: Vec<String>,
    flags: Vec<(String, Option<String>)>,
}

impl Args {
    fn parse(raw: impl Iterator<Item = String>, switches: &[&str]) -> Self {
        let mut positional = Vec::new();
        let mut flags = Vec::new();
        let mut it = raw.peekable();
        while let Some(a) = it.next() {
            if let Some(name) = a.strip_prefix("--") {
                if switches.contains(&name) {
                    flags.push((name.to_string(), None));
                } else {
                    flags.push((name.to_string(), it.next()));
                }
            } else {
                positional.push(a);
            }
        }
        Self { positional, flags }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.flags.iter().rev().find(|(n, _)| n == name).and_then(|(_, v)| v.as_deref())
    }

    fn has(&self, name: &str) -> bool {
        self.flags.iter().any(|(n, _)| n == name)
    }

    fn number<T: std::str::FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.get(name) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("--{name}: '{v}' is not a valid number")),
        }
    }
}

fn parse_hex(s: &str) -> Result<u16, String> {
    let t = s.trim_start_matches("0x").trim_start_matches('$');
    u16::from_str_radix(t, 16).map_err(|_| format!("'{s}' is not a hex address"))
}

pub fn parse_buttons(list: &str) -> Result<Buttons, String> {
    list.split(',').filter(|s| !s.trim().is_empty()).try_fold(Buttons::NONE, |acc, name| {
        Buttons::from_name(name).map(|b| acc | b).ok_or_else(|| format!("unknown button '{name}'"))
    })
}

fn palette(name: Option<&str>) -> Result<&'static [u32; 4], String> {
    match name.unwrap_or("matcha") {
        "grey" | "gray" => Ok(&palettes::GREY),
        "matcha" => Ok(&palettes::MATCHA),
        "dmg" => Ok(&palettes::DMG),
        other => Err(format!("unknown palette '{other}'")),
    }
}

fn read_rom(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("{path}: {e}"))
}

/// `--ram zero|noise[:SEED]`.
pub fn parse_ram(value: Option<&str>) -> Result<Options, String> {
    let power_on_ram = match value.unwrap_or("zero") {
        "zero" => PowerOnRam::Zero,
        "noise" => PowerOnRam::Noise(0),
        other => match other.strip_prefix("noise:").map(str::parse) {
            Some(Ok(seed)) => PowerOnRam::Noise(seed),
            _ => return Err(format!("--ram: expected zero, noise or noise:SEED, got '{other}'")),
        },
    };
    Ok(Options { power_on_ram, ..Options::default() })
}

#[derive(Clone, Copy, Debug)]
pub enum ModelChoice {
    Auto,
    Dmg,
    Cgb,
}

impl ModelChoice {
    fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("auto") {
            "auto" => Ok(Self::Auto),
            "dmg" => Ok(Self::Dmg),
            "cgb" => Ok(Self::Cgb),
            other => Err(format!("--model: expected auto, dmg or cgb, got '{other}'")),
        }
    }

    pub fn resolve(self, rom: &[u8]) -> Model {
        match self {
            Self::Dmg => Model::Dmg,
            Self::Cgb => Model::Cgb,
            Self::Auto if rom.get(0x143).is_some_and(|flag| flag & 0x80 != 0) => Model::Cgb,
            Self::Auto => Model::Dmg,
        }
    }
}

pub fn model_name(model: Model) -> &'static str {
    match model {
        Model::Dmg => "dmg",
        Model::Cgb => "cgb",
    }
}

/// Creates a machine for headless use (no audio output).
fn boot(rom: Vec<u8>, path: &str, mut options: Options, model: ModelChoice) -> Result<GameBoy, String> {
    options.model = model.resolve(&rom);
    let mut gb = GameBoy::with_options(rom, options).map_err(|e| format!("{path}: {e}"))?;
    gb.set_audio_output(false);
    Ok(gb)
}

fn load(path: &str, args: &Args) -> Result<GameBoy, String> {
    boot(read_rom(path)?, path, Options::default(), ModelChoice::parse(args.get("model"))?)
}

fn cmd_info(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("info: missing <rom>")?;
    let gb = load(path, args)?;
    let h = gb.header();
    println!("title        {}", h.title);
    println!("model        {}", model_name(gb.model()));
    println!("cartridge    {:#04x} {}", h.cart_type, h.cart_type_name());
    println!("rom size     {} KiB", h.rom_size / 1024);
    println!("ram size     {} KiB", h.ram_size / 1024);
    println!(
        "cgb          {}",
        if h.cgb_only() {
            "CGB only"
        } else if h.cgb_enhanced() {
            "CGB enhanced"
        } else {
            "DMG"
        }
    );
    println!("sgb          {}", if h.sgb_flag == 0x03 { "yes" } else { "no" });
    println!("version      {}", h.version);
    println!("header sum   {:#04x} ({})", h.header_checksum, if h.header_checksum_ok { "ok" } else { "BAD" });
    println!("global sum   {:#06x}", h.global_checksum);
    Ok(())
}

fn cmd_run(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("run: missing <rom>")?;
    let rom = read_rom(path)?;
    let mut input = match args.get("input") {
        Some(mode) => {
            Some(profile::InputDriver::new(profile::InputMode::parse(Some(mode), profile::InputMode::None)?, &rom))
        }
        None => None,
    };
    let mut gb = boot(rom, path, parse_ram(args.get("ram"))?, ModelChoice::parse(args.get("model"))?)?;
    if let Some(sav) = args.get("sav") {
        if let Ok(data) = std::fs::read(sav) {
            gb.load_battery_ram(&data);
        }
    }
    let frames = args.get("frames").map(|_| args.number("frames", 60u64)).transpose()?;
    let seconds = args.number("seconds", 5.0f64)?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err("--seconds must be a nonnegative finite number".into());
    }
    let target_ticks = (seconds * 4_194_304.0).round() as u64;
    if let Some(list) = args.get("hold") {
        if input.is_some() {
            return Err("use either --hold or --input".into());
        }
        gb.set_buttons(parse_buttons(list)?);
    }
    if args.has("audio") {
        gb.set_sample_rate(48_000);
        gb.set_audio_output(true);
    }
    let started = std::time::Instant::now();
    while frames.map_or(gb.base_clock_ticks() < target_ticks, |n| gb.frame_count() < n) {
        if let Some(input) = &mut input {
            input.drive(&mut gb);
        }
        gb.run_frame();
        gb.clear_audio(); // a host would play these
    }
    let elapsed = started.elapsed().as_secs_f64();
    let frames = gb.frame_count();
    let emulated = gb.base_clock_ticks() as f64 / 4_194_304.0;
    eprintln!(
        "ran {frames} frames ({emulated:.1}s emulated) in {elapsed:.3}s — {:.1}x realtime",
        emulated / elapsed.max(1e-9)
    );
    if args.has("serial") {
        print!("{}", String::from_utf8_lossy(gb.serial_output()));
    }
    if let Some(out) = args.get("screenshot") {
        let scale = args.number("scale", 1u32)?;
        image::write_screenshot(&gb, palette(args.get("palette"))?, scale, Path::new(out))?;
        eprintln!("wrote {out}");
    }
    if let (Some(sav), Some(ram)) = (args.get("sav"), gb.battery_ram()) {
        std::fs::write(sav, ram).map_err(|e| format!("{sav}: {e}"))?;
    }
    Ok(())
}

fn cmd_test(args: &Args) -> Result<bool, String> {
    let root = PathBuf::from(args.positional.first().ok_or("test: missing <test-roms-dir>")?);
    if !root.is_dir() {
        return Err(format!("{} is not a directory (run scripts/fetch-testdata.sh)", root.display()));
    }
    let model = match args.get("model").unwrap_or("dmg") {
        "dmg" => Model::Dmg,
        "cgb" => Model::Cgb,
        _ => return Err("test: --model must be dmg or cgb; auto would mix hardware expectations".into()),
    };
    let conformance::Discovery { mut cases, mut excluded } = conformance::discover(&root, model);
    if let Some(suite) = args.get("suite") {
        cases.retain(|c| c.suite == suite);
        excluded.retain(|c| c.suite == suite);
    }
    if let Some(f) = args.get("filter") {
        cases.retain(|c| c.name.contains(f));
        excluded.retain(|c| c.name.contains(f));
    }
    if cases.is_empty() {
        return Err(format!(
            "no applicable test ROMs matched ({} require unsupported CGB DMG-compatibility mode)",
            excluded.len()
        ));
    }
    let threads = args.number("threads", std::thread::available_parallelism().map_or(2, |n| n.get()))?;
    let started = std::time::Instant::now();
    let outcomes = conformance::run_all(&cases, threads);
    for o in &outcomes {
        let mark = match &o.verdict {
            conformance::Verdict::Pass => "PASS".to_string(),
            conformance::Verdict::Fail(m) => format!("FAIL  {m}"),
            conformance::Verdict::Error(m) => format!("ERROR {m}"),
        };
        println!("{:<10} {:<70} {mark}", o.case.suite, o.case.name);
    }
    let md = conformance::scoreboard_markdown(&outcomes, &excluded);
    println!("\n{md}");
    eprintln!("{} ROMs in {:.1}s", outcomes.len(), started.elapsed().as_secs_f64());
    if let Some(out) = args.get("markdown") {
        std::fs::write(out, &md).map_err(|e| format!("{out}: {e}"))?;
    }
    if let Some(out) = args.get("json") {
        let json = serde_json::to_string_pretty(&conformance::scoreboard_json(&outcomes, &excluded)).unwrap();
        std::fs::write(out, json).map_err(|e| format!("{out}: {e}"))?;
    }
    if let Some(path) = args.get("baseline") {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let baseline: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
        let diff = conformance::compare(&outcomes, &baseline).map_err(|e| format!("{path}: {e}"))?;
        for o in &diff.fixed {
            println!("newly passing: {} {}", o.case.suite, o.case.name);
        }
        for o in &diff.regressed {
            println!("REGRESSION:    {} {}", o.case.suite, o.case.name);
        }
        println!("baseline {path}: {} regressions, {} newly passing", diff.regressed.len(), diff.fixed.len());
        return Ok(diff.regressed.is_empty());
    }
    Ok(outcomes.iter().all(|o| o.verdict == conformance::Verdict::Pass))
}

/// One executed instruction, as `trace` prints it.
struct TraceLine {
    regs: matcha_core::cpu::Registers,
    bytes: [u8; 3],
    bank: usize,
    kind: StepKind,
    /// With `--watch`: the watched byte's (old, new) value if this step changed it.
    changed: Option<(u8, u8)>,
}

impl TraceLine {
    fn print(&self) {
        let r = &self.regs;
        match self.kind {
            StepKind::Interrupt { vector } => println!("      interrupt -> {vector:04x}"),
            StepKind::Stopped => println!("{:04x}  stopped", r.pc),
            StepKind::Halted => println!("{:04x}  halted", r.pc),
            StepKind::Locked { opcode } => println!("{:04x}  {opcode:02x}        illegal opcode: CPU locked up", r.pc),
            StepKind::Instruction { .. } => {
                let ins = matcha_core::disasm::decode(r.pc, self.bytes);
                let bytes: Vec<String> = ins.bytes[..usize::from(ins.len)].iter().map(|b| format!("{b:02x}")).collect();
                let flags: String = [(0x80, 'z'), (0x40, 'n'), (0x20, 'h'), (0x10, 'c')]
                    .iter()
                    .map(|&(bit, c)| if r.f & bit != 0 { c } else { '-' })
                    .collect();
                let changed = self.changed.map(|(old, new)| format!("  watched: {old:02x} -> {new:02x}"));
                println!(
                    "{:04x}  {:<9} {:<22} af={:04x} bc={:04x} de={:04x} hl={:04x} sp={:04x} {flags} bank={}{}",
                    r.pc,
                    bytes.join(" "),
                    ins.text,
                    r.af(),
                    r.bc(),
                    r.de(),
                    r.hl(),
                    r.sp,
                    self.bank,
                    changed.unwrap_or_default()
                );
            }
        }
    }
}

/// Prints executed instructions with the registers before each one: the
/// first `--count` after `--frames`/`--skip`, or with `--last N` the final N
/// before the CPU locks up or `--frames` runs out. `--watch ADDR` limits
/// `--last` to the instructions that changed the byte at ADDR.
fn cmd_trace(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("trace: missing <rom>")?;
    let rom = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let mut input =
        profile::InputDriver::new(profile::InputMode::parse(args.get("input"), profile::InputMode::None)?, &rom);
    let mut gb = boot(rom, path, parse_ram(args.get("ram"))?, ModelChoice::parse(args.get("model"))?)?;
    let frames = args.number("frames", 0u64)?;
    let last = args.number("last", 0usize)?;
    let watch = args.get("watch").map(parse_hex).transpose()?;
    let step = |gb: &mut GameBoy, input: &mut profile::InputDriver| -> TraceLine {
        input.drive(gb);
        let regs = gb.registers();
        let bytes = [gb.peek(regs.pc), gb.peek(regs.pc.wrapping_add(1)), gb.peek(regs.pc.wrapping_add(2))];
        let bank = gb.cartridge().current_rom_bank();
        let old = watch.map(|a| gb.peek(a));
        let kind = gb.step().kind;
        let changed = watch.zip(old).and_then(|(a, old)| Some((old, gb.peek(a))).filter(|(o, n)| o != n));
        TraceLine { regs, bytes, bank, kind, changed }
    };
    if last > 0 {
        let limit = if frames == 0 { (60.0 * matcha_core::FRAME_RATE) as u64 } else { frames };
        let mut ring = std::collections::VecDeque::with_capacity(last);
        while gb.frame_count() < limit {
            let line = step(&mut gb, &mut input);
            let locked = matches!(line.kind, StepKind::Locked { .. });
            let repeat_idle = matches!(line.kind, StepKind::Halted | StepKind::Stopped)
                && ring.back().is_some_and(|l: &TraceLine| l.kind == line.kind);
            if repeat_idle || (watch.is_some() && line.changed.is_none() && !locked) {
                continue;
            }
            if ring.len() == last {
                ring.pop_front();
            }
            ring.push_back(line);
            if locked {
                break;
            }
        }
        println!("frame {} ({} M-cycles):", gb.frame_count(), gb.cycles());
        ring.iter().for_each(TraceLine::print);
        return Ok(());
    }
    while gb.frame_count() < frames {
        input.drive(&mut gb);
        gb.run_frame();
    }
    for _ in 0..args.number("skip", 0u64)? {
        if let StepKind::Locked { .. } = step(&mut gb, &mut input).kind {
            break;
        }
    }
    // A HALT or STOP that nothing ends would print forever; give up after a second.
    let idle_limit = matcha_core::MCYCLES_PER_FRAME * 60;
    let mut idle_since: Option<u64> = None;
    let mut printed = 0;
    let count = args.number("count", 64u64)?;
    while printed < count {
        let before = gb.cycles();
        let line = step(&mut gb, &mut input);
        if matches!(line.kind, StepKind::Halted | StepKind::Stopped) {
            let since = *idle_since.get_or_insert(before);
            if since == before {
                line.print();
                printed += 1;
            }
            if gb.cycles() - since > idle_limit {
                println!(
                    "      still {} after a second; stopping",
                    if line.kind == StepKind::Halted { "halted" } else { "stopped" }
                );
                break;
            }
            continue;
        }
        if let Some(since) = idle_since.take() {
            println!("      ({} M-cycles)", before - since);
        }
        line.print();
        printed += 1;
        if matches!(line.kind, StepKind::Locked { .. }) {
            break;
        }
    }
    Ok(())
}

fn cmd_disasm(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("disasm: missing <rom>")?;
    let gb = load(path, args)?;
    let mut addr = parse_hex(args.get("addr").unwrap_or("0100"))?;
    for _ in 0..args.number("count", 24usize)? {
        let ins = gb.disassemble(addr);
        let bytes: Vec<String> = ins.bytes[..usize::from(ins.len)].iter().map(|b| format!("{b:02x}")).collect();
        println!("{addr:04x}  {:<9} {}", bytes.join(" "), ins.text);
        addr = addr.wrapping_add(u16::from(ins.len));
    }
    Ok(())
}

fn main() -> ExitCode {
    let mut raw = std::env::args().skip(1);
    let Some(cmd) = raw.next() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let args = Args::parse(raw, &["serial", "audio"]);
    let result = match cmd.as_str() {
        "info" => cmd_info(&args).map(|()| true),
        "run" => cmd_run(&args).map(|()| true),
        "test" => cmd_test(&args),
        "profile" => parse_ram(args.get("ram")).and_then(|options| {
            profile::cmd_profile(
                &args.positional,
                args.get("json"),
                args.get("input"),
                args.number("seconds", 30.0f64),
                &options,
                ModelChoice::parse(args.get("model"))?,
            )
        }),
        "disasm" => cmd_disasm(&args).map(|()| true),
        "trace" => cmd_trace(&args).map(|()| true),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(true)
        }
        other => Err(format!("unknown command '{other}'\n\n{USAGE}")),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;
    #[test]
    fn auto_follows_header_and_explicit_model_overrides_it() {
        let mut rom = vec![0; 0x150];
        assert_eq!(ModelChoice::Auto.resolve(&rom), Model::Dmg);
        for flag in [0x80, 0xC0] {
            rom[0x143] = flag;
            assert_eq!(ModelChoice::Auto.resolve(&rom), Model::Cgb);
            assert_eq!(ModelChoice::Dmg.resolve(&rom), Model::Dmg);
        }
        assert_eq!(ModelChoice::Cgb.resolve(&[]), Model::Cgb);
        assert!(ModelChoice::parse(Some("bad")).is_err());
    }
}
