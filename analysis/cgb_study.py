#!/usr/bin/env python3
"""Revalidate the pinned homebrew corpus with automatic DMG/CGB selection.

    python3 analysis/cgb_study.py testdata/homebrew --phase reference
    python3 analysis/cgb_study.py testdata/homebrew --phase matcha
    python3 analysis/cgb_study.py testdata/homebrew --phase report

The historical DMG report is preserved. Every supported default ROM is
included, and coarse outcome agreement is never described as compatibility.
"""
import argparse
import collections
import csv
import hashlib
import html
import json
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "analysis/data"
DATABASE_COMMIT = "50293559a496a3e20382fbf6a2e84b70ec622f88"
SECONDS = 60
SAMEBOY_COMMIT = "213a12ce93d66b105a113debd9396306066a7cfc"
FIELDS = ("rom", "model", "frame_hash_format", "frames", "locked_opcode", "locked", "booted",
          "nonblank_frames", "lcd_on_frames", "distinct_frames_sampled", "static_screens",
          "base_clock_ticks", "reference_ticks_8mhz", "emulated_seconds", "boot_ms", "double_speed", "instructions", "error")


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=1, allow_nan=False) + "\n")
    temporary.replace(path)


def corpus(db):
    commit = subprocess.check_output(["git", "-C", str(db), "rev-parse", "HEAD"], text=True).strip()
    if commit != DATABASE_COMMIT:
        raise ValueError(f"use the pinned database {DATABASE_COMMIT}; found {commit}")
    tree = subprocess.check_output(["git", "-C", str(db), "ls-tree", "-rz", "HEAD", "entries"])
    blobs = {}
    for entry in tree.split(b"\0"):
        if entry:
            meta, path = entry.split(b"\t", 1)
            blobs[path.decode()] = meta.split()[2].decode()
    rows = list(csv.DictReader((DATA / "corpus.csv").open()))
    assert len(rows) == len({r["rom_path"] for r in rows}), "duplicate manifest ROM paths"
    for row in rows:
        path = db / row["rom_path"]
        if not path.is_file():
            raise ValueError(f"missing ROM (check recursive sparse checkout): {path}")
        rom = path.read_bytes()
        assert len(rom) == int(row["rom_bytes"]), f"size drift: {path}"
        blob = hashlib.sha1(f"blob {len(rom)}\0".encode() + rom).hexdigest()
        assert blobs.get(row["rom_path"]) == blob, f"ROM differs from pinned Git tree: {path}"
        assert rom[0x143] == int(row["cgb_flag"], 16), f"model drift: {path}"
        row["rom_sha256"] = hashlib.sha256(rom).hexdigest()
        row["model"] = "cgb" if rom[0x143] & 0x80 else "dmg"
    return rows


def input_id(rows):
    values = [(r["rom_path"], r["rom_sha256"]) for r in rows if r["mapper_supported"] == "1"]
    return hashlib.sha256(json.dumps(values, separators=(",", ":")).encode()).hexdigest()


def run_phase(db, rows, phase, jobs):
    selected = [r for r in rows if r["mapper_supported"] == "1"]
    exe = ROOT / ("target/reference/sameboy_profile" if phase == "reference" else "target/release/matcha")
    if phase == "reference":
        assert (ROOT / "target/reference/sameboy-commit.txt").read_text().strip() == SAMEBOY_COMMIT
    executable_sha256 = digest(exe)
    records = {}
    chunks = [selected[i:i + 8] for i in range(0, len(selected), 8)]

    def one(chunk):
        paths = [r["rom_path"] for r in chunk]
        if phase == "reference":
            cmd = [str(exe), str(ROOT / "target/reference"), str(SECONDS), "zero", "--model", "auto", *paths]
        else:
            cmd = [str(exe), "profile", "--model", "auto", "--seconds", str(SECONDS), "--input", "monkey", *paths]
        result = subprocess.run(cmd, cwd=db, capture_output=True, text=True, timeout=600, check=True)
        result = [json.loads(line) for line in result.stdout.splitlines()] if phase == "reference" else json.loads(result.stdout)
        assert sorted(r["rom"] for r in result) == sorted(paths), "runner omitted or duplicated a ROM"
        return result

    # The CLI profiles its eight-ROM batch concurrently. One CLI process
    # avoids multiplying that thread pool; reference batches are sequential.
    with ThreadPoolExecutor(jobs if phase == "reference" else 1) as pool:
        futures = [pool.submit(one, c) for c in chunks]
        for future in as_completed(futures):
            for record in future.result():
                assert record["rom"] not in records
                assert "error" not in record, record
                records[record["rom"]] = {k: record[k] for k in FIELDS if k in record}
            print(f"{phase}: {len(records)}/{len(selected)}", flush=True)
    assert len(records) == len(selected)
    ordered = [records[r["rom_path"]] for r in selected]
    for row, record in zip(selected, ordered):
        assert record["model"] == row["model"]
        assert record["frame_hash_format"] == ("rgba8" if row["model"] == "cgb" else "dmg-shade-indices")
        assert record["frames"] > 0 or record.get("booted") is False
        if record.get("booted") is not False:
            assert SECONDS <= record["emulated_seconds"] < SECONDS + 0.1
    assert digest(exe) == executable_sha256, "executable changed during the run"
    result = {
        "database_commit": DATABASE_COMMIT, "input_sha256": input_id(rows),
        "seconds": SECONDS, "input": "monkey", "ram": "zero", "model_selection": "auto",
        "created_at": datetime.now(timezone.utc).isoformat(), "executable_sha256": executable_sha256,
        "records": ordered,
    }
    if phase == "reference":
        result["sameboy_commit"] = SAMEBOY_COMMIT
        result["models"] = {"dmg": "DMG-B", "cgb": "CGB-C"}
        result["boot_sha256"] = {m: digest(ROOT / f"target/reference/{m}_boot.bin") for m in ("dmg", "cgb")}
    else:
        # The executable hash is authoritative even during an uncommitted run.
        result["source_base_commit"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        clean = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "crates", "Cargo.toml", "Cargo.lock"], cwd=ROOT).returncode == 0
        if clean:
            result["source_commit"] = result["source_base_commit"]
    write_json(DATA / f"cgb-{phase}.json", result)


def outcome(record):
    if record.get("booted") is False:
        return "boot failed"
    if record.get("locked") or record.get("locked_opcode") is not None:
        return "crashed"
    if record["nonblank_frames"] <= 2:
        return "blank"
    return "static" if record["distinct_frames_sampled"] <= 1 else "running"


def report(rows):
    matcha, reference = [json.loads((DATA / f"cgb-{name}.json").read_text()) for name in ("matcha", "reference")]
    for data in (matcha, reference):
        assert data["database_commit"] == DATABASE_COMMIT
        assert data["input_sha256"] == input_id(rows)
        assert data["seconds"] == SECONDS and data["input"] == "monkey" and data["ram"] == "zero"
        assert data["model_selection"] == "auto"
        for rec in data["records"]:
            assert rec["frame_hash_format"] == ("rgba8" if rec["model"] == "cgb" else "dmg-shade-indices")
            if rec.get("booted") is not False:
                assert SECONDS <= rec["emulated_seconds"] < SECONDS + 0.1
    assert reference["sameboy_commit"] == SAMEBOY_COMMIT
    profiles = {r["rom"]: r for r in matcha["records"]}
    refs = {r["rom"]: r for r in reference["records"]}
    selected = [r for r in rows if r["mapper_supported"] == "1"]
    assert len(profiles) == len(matcha["records"]) == len(refs) == len(reference["records"]) == len(selected)
    assert set(profiles) == set(refs) == {r["rom_path"] for r in selected}
    groups = collections.defaultdict(lambda: {"n": 0, "agree": 0, "matcha": collections.Counter(), "sameboy": collections.Counter()})
    disagreements = []
    records = []
    old_profiles = {r["rom"]: r for r in json.loads((DATA / "profiles.json").read_text())}
    transitions = collections.Counter()
    for row in selected:
        p, r = profiles[row["rom_path"]], refs[row["rom_path"]]
        assert p["model"] == r["model"] == row["model"]
        a, b = outcome(p), outcome(r)
        group = groups[row["cgb_mode"]]
        group["n"] += 1; group["agree"] += a == b
        group["matcha"][a] += 1; group["sameboy"][b] += 1
        rec = {k: row[k] for k in ("slug", "title", "cgb_mode", "model", "rom_path", "rom_sha256", "logo_ok", "header_checksum_ok")}
        rec.update(matcha=a, sameboy=b, agreement=a == b)
        records.append(rec)
        if a != b: disagreements.append(rec)
        if row["rom_path"] in old_profiles:
            transitions[(outcome(old_profiles[row["rom_path"]]), a)] += 1
    summary = {
        "database_commit": DATABASE_COMMIT, "input_sha256": input_id(rows), "seconds": SECONDS,
        "corpus_entries": len(rows), "tested": len(selected), "unsupported_mapper": len(rows) - len(selected),
        "agree": sum(g["agree"] for g in groups.values()), "groups": dict(groups), "disagreements": disagreements,
        "historical_dmg_to_auto": [{"before": a, "after": b, "n": n} for (a, b), n in sorted(transitions.items())],
        "matcha_executable_sha256": matcha["executable_sha256"], "sameboy_commit": SAMEBOY_COMMIT,
        "matcha_source_commit": matcha.get("source_commit", matcha["source_base_commit"]),
    }
    write_json(DATA / "cgb-summary.json", summary)
    with (DATA / "cgb-roms.csv").open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(records[0])); writer.writeheader(); writer.writerows(records)
    lines = ["# DMG and Game Boy Color corpus revalidation", "",
        f"All **{len(selected):,} supported-mapper ROMs** from the pinned {len(rows):,}-entry corpus were run for {SECONDS} emulated seconds with identical deterministic input in matcha and SameBoy. The cartridge header selects DMG or native CGB. {summary['unsupported_mapper']} unsupported-mapper entries are explicitly excluded.", "",
        f"**{summary['agree']:,}/{len(selected):,} coarse outcomes agree.** This only distinguishes a changing picture, a static picture, blank output, a locked CPU, and boot failure. It does not establish correct pixels, sound, gameplay, or complete game compatibility.", "",
        "| Cartridge group | Tested | Outcome agreement | matcha running / static / blank / crashed |",
        "|---|---:|---:|---|",
    ]
    for name in ("DMG", "CGB enhanced", "CGB only"):
        g = groups[name]
        counts = " / ".join(str(g["matcha"].get(k, 0)) for k in ("running", "static", "blank", "crashed"))
        lines.append(f"| {name} | {g['n']} | {g['agree']}/{g['n']} | {counts} |")
    lines += ["", "## Method and limits", "",
        f"- Database: [gbdev/database at `{DATABASE_COMMIT[:7]}`](https://github.com/gbdev/database/tree/{DATABASE_COMMIT}). Every default ROM is included if its mapper is implemented; nested ROM paths and bytes against the pinned Git tree are checked. SHA-256 per ROM is in `analysis/data/cgb-roms.csv`.",
        f"- Reference: [SameBoy `{SAMEBOY_COMMIT[:7]}`](https://github.com/LIJI32/SameBoy/tree/{SAMEBOY_COMMIT}), DMG-B or CGB-C, its open-source boot ROMs, system-RAM randomness disabled, colour correction disabled. matcha starts at post-boot state. Palette and fresh cartridge RAM defaults differ (notably cartridge RAM 00 in matcha versus FF in SameBoy); these startup-input differences can affect outcomes.",
        "- Inputs: the existing FNV-seeded monkey schedule. Distinct frames are sampled every 15 frames; at most two non-uniform frames count as blank. CGB hashes include all RGBA channels; DMG retains the historical shade-index hash.",
        "- Elapsed time uses the base 4,194,304 Hz clock, so double-speed CPU execution does not halve the test duration. Runner errors, missing records, duplicate keys, model mismatches, and changed ROM inputs abort publication.",
        "- Cartridge logo/header checks are recorded in the per-ROM CSV but do not filter supported-mapper inputs. Invalid images can therefore appear as crashes or blank output; neither is automatically an emulator defect.",
        "- The historical DMG study is retained in `docs/analysis.md`. Its comparison with this automatic-model run changes both the emulator and, for enhanced cartridges, the console model; those changes cannot be attributed solely to a timing fix.",
        "- The `ram: zero` configuration means system-RAM initialization; it does not assert identical palette RAM, cartridge RAM or boot-written state. Follow-up probes below isolate these differences without changing the main results.",
        "- Raw compact results and executable/boot-ROM hashes: `analysis/data/cgb-matcha.json` and `analysis/data/cgb-reference.json`. Reproduce with `analysis/cgb_study.py`; see `analysis/README.md`.",
        "", "## Disagreements to investigate", "", "| ROM | Model | matcha | SameBoy |", "|---|---|---|---|",
    ]
    for r in disagreements:
        lines.append(f"| {r['slug']} | {r['model']} | {r['matcha']} | {r['sameboy']} |")
    if not disagreements: lines.append("| None | — | — | — |")
    triage = json.loads((DATA / "cgb-triage.json").read_text())
    lines += ["", "## Follow-up diagnosis", "",
        "These controlled probes explain some disagreements; they do not alter the original results or turn an agreement into a compatibility claim. Measurements and intervention details are recorded in `analysis/data/cgb-triage.json`.", ""]
    for rec in disagreements:
        finding = triage["cases"].get(rec["slug"])
        if finding:
            lines.append(f"- **{rec['slug']} — {finding['category']}.** {finding['note']}")
    lines += ["", "CGB OBJ colors are uninitialized after boot according to [Pan Docs](https://gbdev.io/pandocs/Palettes.html). Reference RAM randomization follows the pinned [SameBoy initialization](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/gb.c). The original DMG diagnosis is preserved in `docs/analysis.md`."]
    (ROOT / "docs/cgb-analysis.md").write_text("\n".join(lines) + "\n")
    # A self-contained, selectable-text report with no runtime dependencies.
    def inline(text):
        text = html.escape(text)
        text = re.sub(r"\[([^\]]+)\]\((https://[^)]+)\)", r'<a href="\2">\1</a>', text)
        text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
        return re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)

    content = []
    table = False
    for line in lines:
        if line.startswith("|"):
            if line.startswith("|---"): continue
            if not table: content.append("<table>"); table = True
            content.append("<tr>" + "".join(f"<td>{inline(c.strip())}</td>" for c in line.strip("|").split("|")) + "</tr>")
        else:
            if table: content.append("</table>"); table = False
            if line.startswith("# "): content.append(f"<h1>{html.escape(line[2:])}</h1>")
            elif line.startswith("## "): content.append(f"<h2>{html.escape(line[3:])}</h2>")
            elif line: content.append(f"<p>{inline(line)}</p>")
    if table: content.append("</table>")
    page = '<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>matcha · Color corpus</title><style>body{max-width:1000px;margin:48px auto;padding:0 24px;background:#f6f7f0;color:#20372b;font:16px/1.6 system-ui}h1{font-size:34px;line-height:1.2}h2{margin-top:36px}table{width:100%;border-collapse:collapse;font-size:14px}td{padding:10px;border-bottom:1px solid #becab8}tr:first-child{font-weight:700}p{overflow-wrap:anywhere}@media(max-width:600px){body{padding:0 14px}table{font-size:11px}td{padding:5px}}</style>' + "\n".join(content)
    (ROOT / "dist").mkdir(exist_ok=True)
    (ROOT / "dist/cgb-analysis.html").write_text(page)
    print(json.dumps({k: v for k, v in summary.items() if k not in ("disagreements", "historical_dmg_to_auto")}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("database", type=Path)
    parser.add_argument("--phase", choices=("reference", "matcha", "report"), required=True)
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    if not 1 <= args.jobs <= 32: parser.error("--jobs must be 1..32")
    rows = corpus(args.database.resolve())
    if args.phase == "report": report(rows)
    else: run_phase(args.database.resolve(), rows, args.phase, args.jobs)
