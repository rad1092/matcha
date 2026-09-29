//! SingleStepTests/sm83 conformance: 1000 randomized cases for every legal
//! opcode, checking final registers, memory, and the exact sequence of bus
//! cycles (address, data, read/write/idle) against the ares SM83 core.
//!
//! Data: `testdata/sst/v1/*.json` (fetch with `scripts/fetch-testdata.sh`).
//! Set `MATCHA_REQUIRE_TESTDATA=1` to turn missing data into a failure (CI).

use matcha_core::cpu::{Cpu, CpuBus, Registers};
use serde_json::Value;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Read,
    Write,
    Idle,
}

#[derive(Clone, Copy, Debug)]
struct Cycle {
    addr: u16,
    data: u8,
    kind: Kind,
}

struct FlatBus {
    mem: Vec<u8>,
    cycles: Vec<Cycle>,
}

impl CpuBus for FlatBus {
    fn read(&mut self, addr: u16) -> u8 {
        let data = self.mem[addr as usize];
        self.cycles.push(Cycle { addr, data, kind: Kind::Read });
        data
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
        self.cycles.push(Cycle { addr, data: value, kind: Kind::Write });
    }
    fn idle(&mut self) {
        self.cycles.push(Cycle { addr: 0, data: 0, kind: Kind::Idle });
    }
    fn pending_interrupts(&self) -> u8 {
        0
    }
    fn acknowledge_interrupt(&mut self, _mask: u8) {}
}

/// STOP and HALT are modelled by the reference core as multi-cycle
/// instructions tied to its own scheduler; they are covered by the
/// Blargg/mooneye ROM suites instead.
const SKIPPED: &[&str] = &["10", "76"];

fn data_dir() -> PathBuf {
    std::env::var_os("MATCHA_SST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/sst/v1"))
}

fn u8_of(v: &Value, key: &str) -> u8 {
    v[key].as_u64().unwrap_or_else(|| panic!("missing {key}")) as u8
}

fn u16_of(v: &Value, key: &str) -> u16 {
    v[key].as_u64().unwrap_or_else(|| panic!("missing {key}")) as u16
}

fn regs_of(v: &Value) -> Registers {
    Registers {
        a: u8_of(v, "a"),
        f: u8_of(v, "f"),
        b: u8_of(v, "b"),
        c: u8_of(v, "c"),
        d: u8_of(v, "d"),
        e: u8_of(v, "e"),
        h: u8_of(v, "h"),
        l: u8_of(v, "l"),
        sp: u16_of(v, "sp"),
        pc: u16_of(v, "pc"),
    }
}

/// Runs one test case; returns a description of the first mismatch.
fn run_case(case: &Value, bus: &mut FlatBus) -> Result<(), String> {
    let init = &case["initial"];
    let fin = &case["final"];
    bus.mem.fill(0);
    bus.cycles.clear();
    for entry in init["ram"].as_array().unwrap() {
        let addr = entry[0].as_u64().unwrap() as usize;
        bus.mem[addr] = entry[1].as_u64().unwrap() as u8;
    }
    let mut cpu = Cpu::new();
    cpu.regs = regs_of(init);
    cpu.ime = init["ime"].as_u64() == Some(1);

    cpu.step(bus);

    let expected = regs_of(fin);
    if cpu.regs != expected {
        return Err(format!("registers\n  got      {:x?}\n  expected {expected:x?}", cpu.regs));
    }
    let want_ime = fin["ime"].as_u64() == Some(1);
    if cpu.ime != want_ime {
        return Err(format!("ime: got {} expected {want_ime}", cpu.ime));
    }
    let want_ei = fin["ei"].as_u64() == Some(1);
    if cpu.ei_pending() != want_ei {
        return Err(format!("ei pending: got {} expected {want_ei}", cpu.ei_pending()));
    }
    for entry in fin["ram"].as_array().unwrap() {
        let addr = entry[0].as_u64().unwrap() as usize;
        let want = entry[1].as_u64().unwrap() as u8;
        if bus.mem[addr] != want {
            return Err(format!("ram[{addr:#06x}] = {:#04x}, expected {want:#04x}", bus.mem[addr]));
        }
    }
    let want_cycles = case["cycles"].as_array().unwrap();
    if want_cycles.len() != bus.cycles.len() {
        return Err(format!(
            "cycle count {} expected {}\n  got {:?}\n  expected {want_cycles:?}",
            bus.cycles.len(),
            want_cycles.len(),
            bus.cycles
        ));
    }
    for (i, (got, want)) in bus.cycles.iter().zip(want_cycles).enumerate() {
        let pins = want[2].as_str().unwrap();
        let kind = match pins {
            "r-m" => Kind::Read,
            "-wm" => Kind::Write,
            _ => Kind::Idle,
        };
        if got.kind != kind {
            return Err(format!("cycle {i}: {:?}, expected {pins}", got.kind));
        }
        if kind != Kind::Idle {
            let addr = want[0].as_u64().unwrap() as u16;
            let data = want[1].as_u64().unwrap() as u8;
            if got.addr != addr || got.data != data {
                return Err(format!(
                    "cycle {i}: {:?} {:#06x}={:#04x}, expected {pins} {addr:#06x}={data:#04x}",
                    got.kind, got.addr, got.data
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn single_step_tests() {
    let dir = data_dir();
    if !dir.is_dir() {
        assert!(
            std::env::var_os("MATCHA_REQUIRE_TESTDATA").is_none(),
            "SingleStepTests data missing at {}",
            dir.display()
        );
        eprintln!("skipping: SingleStepTests data not found at {}", dir.display());
        return;
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();

    let mut bus = FlatBus { mem: vec![0; 0x10000], cycles: Vec::with_capacity(8) };
    let (mut total, mut failed_cases) = (0usize, 0usize);
    let mut report = String::new();
    for path in &files {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        if SKIPPED.contains(&stem.as_str()) {
            continue;
        }
        let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut first_failure = None;
        let mut failures = 0;
        for case in &cases {
            total += 1;
            if let Err(msg) = run_case(case, &mut bus) {
                failures += 1;
                first_failure.get_or_insert_with(|| format!("{}: {msg}", case["name"]));
            }
        }
        if failures > 0 {
            failed_cases += failures;
            let _ = writeln!(report, "[{stem}] {failures}/{} failed; first: {}", cases.len(), first_failure.unwrap());
        }
    }
    eprintln!("SingleStepTests: {} cases in {} files, {failed_cases} failed", total, files.len() - SKIPPED.len());
    assert!(failed_cases == 0, "\n{report}");
}
