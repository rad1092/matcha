//! `matcha` — headless runner, conformance scoreboard and profiler.

mod conformance;
mod image;
mod profile;

use matcha_core::{Buttons, GameBoy, RunEvent, palettes};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
matcha — a Game Boy (DMG) emulator you can see inside

USAGE:
  matcha info <rom>
  matcha run <rom> [--seconds N | --frames N] [--hold BUTTONS] [--screenshot out.png]
                   [--scale N] [--palette grey|matcha|dmg] [--serial] [--sav file.sav]
  matcha test <test-roms-dir> [--suite NAME] [--filter TEXT] [--threads N]
                   [--markdown out.md] [--json out.json]
  matcha profile <rom>... [--seconds N] [--input none|monkey] [--json out.json]
  matcha disasm <rom> [--addr HEX] [--count N]

BUTTONS is a comma list: a,b,start,select,up,down,left,right
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
    list.split(',')
        .filter(|s| !s.trim().is_empty())
        .try_fold(Buttons::NONE, |acc, name| {
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

fn load(path: &str) -> Result<GameBoy, String> {
    let rom = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    GameBoy::new(rom).map_err(|e| format!("{path}: {e}"))
}

fn cmd_info(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("info: missing <rom>")?;
    let gb = load(path)?;
    let h = gb.header();
    println!("title        {}", h.title);
    println!("cartridge    {:#04x} {}", h.cart_type, h.cart_type_name());
    println!("rom size     {} KiB", h.rom_size / 1024);
    println!("ram size     {} KiB", h.ram_size / 1024);
    println!(
        "cgb          {}",
        if h.cgb_only() { "CGB only" } else if h.cgb_enhanced() { "CGB enhanced" } else { "DMG" }
    );
    println!("sgb          {}", if h.sgb_flag == 0x03 { "yes" } else { "no" });
    println!("version      {}", h.version);
    println!("header sum   {:#04x} ({})", h.header_checksum, if h.header_checksum_ok { "ok" } else { "BAD" });
    println!("global sum   {:#06x}", h.global_checksum);
    Ok(())
}

fn cmd_run(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("run: missing <rom>")?;
    let mut gb = load(path)?;
    if let Some(sav) = args.get("sav") {
        if let Ok(data) = std::fs::read(sav) {
            gb.load_battery_ram(&data);
        }
    }
    let frames: u64 = match args.get("frames") {
        Some(_) => args.number("frames", 60u64)?,
        None => (args.number("seconds", 5.0f64)? * matcha_core::FRAME_RATE).round() as u64,
    };
    if let Some(list) = args.get("hold") {
        gb.set_buttons(parse_buttons(list)?);
    }
    let started = std::time::Instant::now();
    while gb.frame_count() < frames {
        if let RunEvent::Breakpoint { pc } = gb.run_frame() {
            return Err(format!("unexpected breakpoint at {pc:#06x}"));
        }
        gb.clear_audio();
    }
    let elapsed = started.elapsed().as_secs_f64();
    let emulated = frames as f64 / matcha_core::FRAME_RATE;
    eprintln!(
        "ran {frames} frames ({emulated:.1}s emulated) in {elapsed:.2}s — {:.0}x realtime",
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
    let mut cases = conformance::discover(&root);
    if let Some(suite) = args.get("suite") {
        cases.retain(|c| c.suite == suite);
    }
    if let Some(f) = args.get("filter") {
        cases.retain(|c| c.name.contains(f));
    }
    if cases.is_empty() {
        return Err("no test ROMs matched".into());
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
    let md = conformance::scoreboard_markdown(&outcomes);
    println!("\n{md}");
    eprintln!("{} ROMs in {:.1}s", outcomes.len(), started.elapsed().as_secs_f64());
    if let Some(out) = args.get("markdown") {
        std::fs::write(out, &md).map_err(|e| format!("{out}: {e}"))?;
    }
    if let Some(out) = args.get("json") {
        let json = serde_json::to_string_pretty(&conformance::scoreboard_json(&outcomes)).unwrap();
        std::fs::write(out, json).map_err(|e| format!("{out}: {e}"))?;
    }
    Ok(outcomes.iter().all(|o| o.verdict == conformance::Verdict::Pass))
}

fn cmd_disasm(args: &Args) -> Result<(), String> {
    let path = args.positional.first().ok_or("disasm: missing <rom>")?;
    let gb = load(path)?;
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
    let args = Args::parse(raw, &["serial"]);
    let result = match cmd.as_str() {
        "info" => cmd_info(&args).map(|()| true),
        "run" => cmd_run(&args).map(|()| true),
        "test" => cmd_test(&args),
        "profile" => profile::cmd_profile(&args.positional, args.get("json"), args.get("input"), args.number("seconds", 30.0f64)),
        "disasm" => cmd_disasm(&args).map(|()| true),
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
