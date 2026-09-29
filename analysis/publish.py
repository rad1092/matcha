#!/usr/bin/env python3
"""Renders the corpus study from analysis/data/summary.json (report.py):

    docs/analysis.md              the report for the repository (charts: docs/img/*.svg)
    dist/analysis.html            the same report as one self-contained interactive page
    dist/analysis.fragment.html   that page without <!doctype>/<meta>, for hosts that
                                  wrap content in their own document skeleton

    python3 analysis/publish.py

Every number comes from summary.json; only the prose lives here.
"""

import html
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
S = json.loads((ROOT / "analysis/data/summary.json").read_text())
PAGE = ROOT / "analysis/page"


def n(v):
    return f"{int(v + 0.5):,}"  # half up; every value here is non-negative


def pct(v, d=0):
    return f"{v:.{d}f}%"


def share(part, whole, d=0):
    return pct(100.0 * part / whole, d)


C, M, A, R, U, O, I, MEM, B = (S[k] for k in
                               ("corpus", "compatibility", "agreement", "ram", "cpu", "opcodes", "interrupts",
                                "memory", "bench"))
H = S["history"]
TC = U["by_toolchain"]
FP = C["fingerprints"]
GBS, GBDK, OTHER = "GB Studio 3+", "GBDK-2020 (C)", "other / unknown"
HW_CGB = M["by_hardware"].get("CGB enhanced", {})
W1, W2 = C["waves"]["1997-2002"], C["waves"]["2019-"]
CRASH_WITH_NOISE = sum(t["n"] for t in R["transitions"] if t["r_outcome"] == "crashed")
TOP3 = [t["text"] for t in O["top"][:3]]

# Prose below makes these claims; check them against the data instead of trusting a rerun.
assert [d["slug"] for d in A["disagreements"]] == ["child-s-play", "hybrid-gbplot"], A["disagreements"]
assert M["stuck_in_stop_wrote_key1"] == M["stuck_in_stop"] == M["stuck_in_stop_cgb_flag"]
assert all(t.startswith(("jr", "ldh a", "ld a", "and", "cp", "bit")) for t in TOP3), TOP3
assert O["cb_top"][0]["text"] == "swap a", O["cb_top"][0]
assert max(O["ld_hl_sp_e8_rank"][GBS], O["ld_hl_sp_e8_rank"][GBDK]) <= 5 < O["ld_hl_sp_e8_rank"][OTHER]
assert all(v["entries"] == v["fingerprinted_as_tagged"] for v in FP["tagged"].values()) and FP["fingerprinted_before_2019"] == 0
assert CRASH_WITH_NOISE + R["fixed_by_noise"] == R["outcome_changed"]
assert [e["slug"] for e in R["examples"] if e["r_outcome"] == "running"] == ["fatass"]

# --- content -------------------------------------------------------------------------------
# Paragraphs use a small markdown subset (**bold**, `code`, [text](url)); figures name a
# chart that exists both as docs/img/<id>.svg and as an interactive chart on the page.

DEK = (f"Every cartridge in the Homebrew Hub database made for the original Game Boy — "
       f"{n(M['profiled'])} of {n(C['entries_with_rom'])} — played for one emulated minute by matcha "
       f"and by SameBoy with identical scripted input. What the software does with the machine, how "
       f"the two emulators compare, and the three bugs the comparison exposed.")

KPIS = [
    (n(C["entries_with_rom"]), "cartridges scanned from Homebrew Hub"),
    (n(M["profiled"]), "made for the original Game Boy, each played for a minute"),
    (f"{n(A['outcome_agree'])}/{n(M['profiled'])}", "reach the same outcome in matcha and SameBoy"),
    ("3", "emulator bugs found by the comparison, fixed"),
]

FINDINGS = [
    f"**Two waves of homebrew.** Dated entries cluster in 1997–2002 ({n(C['by_era']['1995-2005'])} "
    f"entries up to 2005) and since 2019 ({n(C['by_era']['2019-'])}), peaking at {n(C['peak_year_count'])} "
    f"in {C['peak_year']}. {pct(FP['fingerprinted_share_2019_on_pct'])} of the second wave carries a GB Studio "
    f"or GBDK-2020 fingerprint.",
    f"**Colour is the biggest gap.** {share(C['hardware']['CGB only'], C['entries_with_rom'])} of the "
    f"cartridges are Game Boy Color only; {n(M['stuck_in_stop'])} more are flagged as working on a DMG but "
    f"hang in a Color speed switch, as they would on the hardware.",
    f"**Old code spins, new code sleeps.** The median program from 1995–2005 keeps the CPU busy "
    f"{pct(U['era']['1995-2005']['median_util'] * 100)} of the time; since 2019 the median is "
    f"{pct(U['era']['2019-']['median_util'] * 100)}, because GB Studio and GBDK halt between frames. "
    f"{pct(U['never_halts_pct'])} of all programs never execute HALT.",
    f"**A few dozen opcodes do the work.** Weighting every program equally, "
    f"{O['opcodes_for_share']['50']} opcodes make up half of what executes and "
    f"{O['opcodes_for_share']['90']} make up 90%. The top three — {', '.join(f'`{t}`' for t in TOP3)} — "
    f"are the busy-wait loop.",
    f"**GB Studio games lean on raster interrupts.** {pct(I['by_toolchain'][GBS]['stat'])} of them take "
    f"STAT interrupts, against {pct(I['uses_pct']['stat'])} overall — mid-frame PPU timing matters to them.",
    f"**The comparison found three matcha bugs:** the post-boot VRAM lacked the boot logo, STOP woke on "
    f"any button, and OAM DMA did not take over the CPU's bus. With them fixed, the emulators agree on "
    f"{n(A['outcome_agree'])} of {n(M['profiled'])} programs; of the two left, one traces to SameBoy's own "
    f"boot ROM and the other crashes in both, just differently.",
    f"**Some homebrew reads memory it never wrote.** Starting RAM with noise instead of zeros changes the "
    f"outcome of {n(R['outcome_changed'])} programs.",
]


def section_corpus():
    hw = C["hardware"]
    tc = C["toolchain"]
    return {
        "id": "corpus", "kicker": "01 · The corpus", "title": "Two eras of Game Boy homebrew",
        "body": [
            f"[Homebrew Hub](https://hh.gbdev.io) catalogues homebrew, demos and tools for the Game Boy. "
            f"Its database has {n(C['entries_with_rom'])} entries with a ROM file. Parsing each cartridge "
            f"header gives the hardware it targets: {n(hw['DMG'])} are plain Game Boy (DMG) programs, "
            f"{n(hw['CGB enhanced'])} are flagged as Color-enhanced but DMG-compatible, and "
            f"{n(hw['CGB only'])} ({pct(C['cgb_only_pct'])}) require a Game Boy Color.",
            f"Where the entry has a date ({n(C['dated'])} of them), two waves stand out. The first, "
            f"1997–2002, is the scene around the Game Boy Color's release: {pct(W1['color_pct'])} of its "
            f"{n(W1['n'])} entries target the Color and {pct(W1['demo_pct'])} are demos. The second, since "
            f"2019, is mostly games ({pct(W2['game_pct'])}). {pct(W2['competition_pct'])} of its entries were "
            f"made for competitions and game jams, and {pct(FP['fingerprinted_share_2019_on_pct'])} carry the "
            f"fingerprint of one of two toolchains: GB Studio 3's crash handler (found in {n(tc[GBS])} "
            f"cartridges overall) or GBDK-2020 library code ({n(tc[GBDK])}).",
        ],
        "figure": ("corpus-years", "Entries per year, by the hardware the header asks for",
                   f"Homebrew Hub entries with a ROM and a known year ({n(C['dated'])} of "
                   f"{n(C['entries_with_rom'])})."),
        "after": [
            f"{n(C['boots_on_dmg'])} cartridges pass the checks the DMG boot ROM performs (logo and header "
            f"checksum). Every one of them uses a mapper matcha implements (MBC1, 2, 3, 5 or none); the "
            f"{n(C['unsupported_mapper'])} entries with other cartridge types all fail the logo check, so "
            f"they are not bootable cartridge images. Setting those and the Color-only cartridges aside "
            f"leaves {n(C['dmg_runnable'])} programs made for the original Game Boy: the corpus for "
            f"everything below. ({n(M['bad_header']['count'])} of them fail the boot ROM's checks and would "
            f"not start on a real console; both emulators run them anyway.)",
        ],
    }


def section_outcomes():
    m = M["matcha"]
    hw = M["by_hardware"]
    return {
        "id": "outcomes", "kicker": "02 · Does it run?", "title": "One minute of play, four outcomes",
        "body": [
            "Each program ran for 60 emulated seconds (3,584 frames) with a scripted \"monkey\" player: "
            "Start and A every two seconds for the first ten seconds, then a random direction with A/B "
            "every eight frames. The run is classified by what the screen and CPU did: **running** (the "
            "picture changed), **static** (one picture the whole minute), **blank** (never more than a "
            "frame or two of anything), or **crashed** (the CPU hit an illegal opcode and locked up).",
        ],
        "table": [["Outcome", "matcha", "SameBoy", "matcha, DMG programs", "matcha, Color-flagged"]] + [
            [o, n(m[o]), n(A["sameboy"][o]), n(hw.get("DMG", {}).get(o, 0)), n(hw.get("CGB enhanced", {}).get(o, 0))]
            for o in ("running", "static", "blank", "crashed")
        ],
        "after": [
            f"{share(m['running'], M['profiled'])} of the programs run. The failures concentrate in "
            f"Color-flagged cartridges: {n(HW_CGB.get('blank', 0))} of the {n(m['blank'])} blank screens and "
            f"{n(HW_CGB.get('crashed', 0))} of the {n(m['crashed'])} crashes. {n(M['stuck_in_stop'])} "
            f"Color-flagged programs sit in STOP for most of the minute: replaying each with "
            f"`matcha trace --input monkey --last 40` shows it writing KEY1, the Color's speed-switch "
            f"register, just before STOP — code meant for a Color. On an original Game Boy, STOP then waits "
            f"for a button whose row nobody selected. {n(M['logo_only'])} static programs never draw "
            f"anything themselves and leave the boot logo on screen.",
        ],
    }


def section_agreement():
    hist = H["agreement"]
    return {
        "id": "agreement", "kicker": "03 · Checking the emulator", "title": "matcha against SameBoy, program by program",
        "body": [
            "Test suites check what their authors thought to test. Running the whole corpus through a second, "
            "independent emulator checks everything else. SameBoy — among the most accurate Game Boy "
            "emulators — ran every program with matcha's exact input schedule and screen metrics (a small C "
            "harness around its core, in `analysis/reference/`), starting RAM at zero as matcha does.",
            "Disagreements were chased to their first diverging instruction with side-by-side instruction "
            "traces of both emulators. They exposed three matcha bugs, and fixing them resolved all but two:",
        ],
        "list": [
            "**Post-boot VRAM.** The DMG boot ROM leaves the cartridge's logo in VRAM; matcha started with "
            "it empty, so programs that never clear VRAM showed a blank screen instead of the logo.",
            "**STOP.** matcha woke from STOP on any button press. On a DMG only a selected button row can "
            "end it, a held button turns STOP into HALT, and the clock really stops (PPU, timer and APU "
            "freeze). Color programs attempting a speed switch now hang as on hardware instead of running on "
            "into garbage.",
            "**OAM DMA bus conflicts.** While DMA copies to OAM it owns the bus it reads from; a CPU fetching "
            "code from ROM at that moment reads the DMA's bytes. Demos that start DMA from ROM (where "
            "games copy the routine to HRAM) diverged within a frame. Gambatte's hardware-verified oamdma "
            "tests then pinned the timing: 143 → 333 of 393 pass.",
        ],
        "table": [["matcha version", "Same outcome as SameBoy"]] + [
            [label, f"{n(v)} of {n(M['profiled'])}"] for label, v in hist
        ],
        "after": [
            "Two differences remain. `hybrid-gbplot` reads STAT a few instructions after boot and branches "
            "on the PPU mode. SameBoy runs its own DMG boot ROM, which hands over at a different point of the "
            "frame than Nintendo's: Mooneye's `boot_hwio` and `boot_div` tests, which matcha passes, fail in "
            "SameBoy with that boot ROM. `child-s-play` goes off the rails in both: 45 seconds in, `matcha "
            "trace` shows it executing data from $0000 with the stack at $0BC5 until it reaches a STOP byte, "
            "while SameBoy's run reaches an illegal opcode at 47 seconds — so one counts as crashed and the "
            "other does not.",
            f"Pictures agree pixel for pixel too. A static screen is a picture held for half a second; of the "
            f"{n(A['with_static_screens_both'])} programs that showed one in both runs, "
            f"{n(A['static_share_one'])} share at least one bit-identical screen and for "
            f"{n(A['static_sets_identical'])} the sets are the same. Where sets differ, the likely cause is "
            f"timing rather than drawing: the boot ROMs hand over at different PPU phases, so a button press "
            f"lands at a different point of a program's frame and the runs catch different in-between "
            f"pictures.",
        ],
    }


def section_cpu():
    return {
        "id": "cpu", "kicker": "04 · CPU time", "title": "Old code spins, new code sleeps",
        "body": [
            f"A Game Boy program that has finished its frame's work can HALT until the next interrupt, "
            f"saving battery; or it can spin in a loop polling LY or STAT. The profiler counts every M-cycle "
            f"as busy, halted or stopped. Across the {n(U['n'])} programs that did not crash, the median "
            f"keeps the CPU busy {pct(U['median_util'] * 100)} of the time, and {pct(U['never_halts_pct'])} "
            f"never execute HALT at all.",
        ],
        "figure": ("cpu-utilization", "Share of time the CPU is busy, per program",
                   "Distribution per toolchain; the right-most bar (95–100% busy) is mostly programs "
                   "that never halt."),
        "after": [
            f"The split is generational. GB Studio (median {pct(TC[GBS]['median_util'] * 100)}, "
            f"{pct(TC[GBS]['never_halts_pct'])} never halt) and GBDK-2020 (median "
            f"{pct(TC[GBDK]['median_util'] * 100)}) wait for VBlank with HALT; hand-written code of the "
            f"first wave mostly spins (median {pct(U['era']['1995-2005']['median_util'] * 100)} for "
            f"1995–2005, {pct(U['era']['1995-2005']['never_halts_pct'])} never halt). For an emulator this "
            f"matters because halted time is nearly free: matcha skips it in bulk (see the HALT fast path in "
            f"docs/ARCHITECTURE.md).",
        ],
    }


def section_opcodes():
    top = O["top"][:10]
    return {
        "id": "opcodes", "kicker": "05 · Instruction mix", "title": "A few dozen opcodes do almost all the work",
        "body": [
            f"The programs executed {n(O['instructions_total'])} instructions. Weighting every program "
            f"equally, {O['opcodes_for_share']['50']} opcodes make up half of them, "
            f"{O['opcodes_for_share']['80']} make up 80% and {O['opcodes_for_share']['90']} make up 90%; "
            f"the CB prefix (bit operations, shifts, `swap`) accounts for {pct(O['cb_prefix_share'], 1)}.",
        ],
        "figure": ("opcode-pareto", "Cumulative share of executed instructions",
                   "Opcodes ranked by their average share across programs."),
        "table": [["#", "Instruction", "Share"]] + [[str(i + 1), f"`{t['text']}`", pct(t["share"], 1)]
                                                    for i, t in enumerate(top)],
        "after": [
            f"The top three are the busy-wait loop — load a hardware register, test it, jump back. Compiled "
            f"code has its own fingerprint: `ld hl, sp+e8`, the SDCC compiler's way of reaching local "
            f"variables, ranks #{O['ld_hl_sp_e8_rank'][GBS]} in GB Studio games and #{O['ld_hl_sp_e8_rank'][GBDK]} "
            f"in GBDK programs, but #{O['ld_hl_sp_e8_rank'][OTHER]} in everything else. Within the CB prefix, "
            f"`swap a` alone is {pct(O['cb_top'][0]['share_of_cb'])} — the usual way to split a byte into BCD "
            f"or hex digits.",
        ],
    }


def section_interrupts():
    u = I["uses_pct"]
    return {
        "id": "interrupts", "kicker": "06 · Interrupts", "title": "What homebrew listens to",
        "body": [
            f"{pct(u['vblank'])} of the programs take the VBlank interrupt, {pct(u['stat'])} the STAT "
            f"(LCD) interrupt, {pct(u['timer'])} the timer — a common way to pace music — and almost none the "
            f"serial port ({pct(u['serial'], 1)}) or joypad ({pct(u['joypad'], 1)}). "
            f"{pct(I['no_interrupts_pct'])} use no interrupts at all.",
        ],
        "figure": ("interrupts", "Share of programs that took each interrupt at least once", ""),
        "after": [
            f"STAT is the raster-effect interrupt: it fires at a chosen line or mode, and code that runs "
            f"then changes scroll registers or palettes mid-frame. {pct(I['by_toolchain'][GBS]['stat'])} "
            f"of GB Studio games take it, against {pct(I['by_toolchain'][OTHER]['stat'])} of other code. "
            f"That is the case for a pixel-accurate PPU (roadmap item 2).",
        ],
    }


def section_ram():
    return {
        "id": "ram", "kicker": "07 · Uninitialised memory", "title": "What happens if RAM starts with noise",
        "body": [
            f"Real Game Boy RAM powers up with a noisy bit pattern, not zeros. SameBoy emulates that; "
            f"matcha starts at zero so runs are reproducible. Running SameBoy again with its DMG noise "
            f"pattern (fixed seed) changed the outcome of {n(R['outcome_changed'])} programs "
            f"({pct(R['outcome_changed_pct'], 1)}) — {n(CRASH_WITH_NOISE)} crash with noise, "
            f"{n(R['fixed_by_noise'])} only runs with it — and {n(R['static_screens_changed'])} programs "
            f"showed at least one different static screen.",
            "These are programs that read memory before writing it. One of them, the music tool Fatass "
            "Tracker, draws bytes from uninitialised RAM until it meets an end marker ($FB or above). With "
            "zeros it never finds one, writes on past its buffer over its own variables and crashes — in "
            "both emulators; with noise it stops early and runs.",
        ],
        "table": [["SameBoy, zero RAM", "SameBoy, noisy RAM", "Programs"]] + [
            [t["z_outcome"], t["r_outcome"], n(t["n"])] for t in R["transitions"]
        ],
    }


def section_speed():
    return {
        "id": "speed", "kicker": "08 · Emulation speed", "title": "Idle games emulate fastest",
        "body": [
            f"On one core of the 2-vCPU build container, with the profiler off and the same scripted input, "
            f"a random sample of {n(B['n'])} programs ran at a median {n(B['median_speed'])}× real time "
            f"(the slowest {n(B['min_speed'])}×). Speed follows CPU load (correlation "
            f"{B['corr_util_speed']:.2f}): programs busy under 30% of the time run at a median "
            f"{n(B['median_speed_util_below_30'])}×, those busy more than 95% of the time at "
            f"{n(B['median_speed_util_above_95'])}×.",
        ],
        "figure": ("speed", "Emulation speed against CPU load", f"Random sample of {n(B['n'])} programs."),
    }


def section_implications():
    return {
        "id": "implications", "kicker": "09 · What this changes", "title": "What this means for matcha",
        "list": [
            f"**Game Boy Color mode first.** {share(C['hardware']['CGB only'], C['entries_with_rom'])} of the "
            f"corpus cannot run without it, and most failures in the rest are Color programs.",
            f"**Then a pixel FIFO.** {pct(I['uses_pct']['stat'])} of programs use raster interrupts; "
            f"Mealybug and Gambatte's mid-mode-3 tests are the remaining PPU gaps.",
            "**Keep differential testing.** The SameBoy harness found three bugs that the suites matcha "
            "ran had missed; CI can run it on a sample of the corpus.",
            f"**Offer noisy power-on RAM** as an option: {n(R['outcome_changed'])} programs depend on it.",
            "**Keep HALT cheap.** GB Studio and GBDK programs spend more than half their time halted; "
            "bulk-skipping it is why matcha runs them fastest.",
        ],
    }


def section_method():
    return {
        "id": "method", "kicker": "10 · Method", "title": "Data, method and caveats",
        "list": [
            "**Corpus.** [gbdev/database](https://github.com/gbdev/database) at commit `5029355` "
            "(2026-09-19): every entry's default `.gb`/`.gbc`. Headers parsed by "
            "`analysis/build_manifest.py`; ROMs are not redistributed.",
            f"**Toolchain fingerprints.** Byte signatures for GB Studio 3+ and GBDK-2020 agree with every "
            f"entry whose authors tagged the tool ({n(FP['tagged']['GB Studio']['entries'])} GB Studio, "
            f"{n(FP['tagged']['GBDK']['entries'])} GBDK) and fire on none of the "
            f"{n(FP['dated_before_2019'])} entries dated before 2019. Older GB Studio and GBDK versions and "
            f"assembly are not told apart ({n(C['toolchain'][OTHER])} \"other / unknown\").",
            "**Play.** 60 emulated seconds per program with the scripted monkey input, identical in both "
            "emulators (`InputDriver` in `crates/matcha-cli/src/profile.rs`). A minute of random input is "
            f"not a playthrough: the median program started instructions at only "
            f"{pct(MEM['coverage_median_pct'], 1)} of its ROM's byte addresses (ROMs also hold graphics and "
            f"data).",
            "**Reference.** SameBoy at commit `213a12c`, DMG-B model, its open-source DMG boot ROM, RAM at "
            "zero (and, for section 07, noise with a fixed seed). SameBoy is a reference, not ground truth; "
            "its boot ROM hands over at a different PPU phase than Nintendo's.",
            "**Outcomes.** Crashed = illegal opcode; blank = at most two frames that are not one uniform "
            "colour; static = one distinct picture among every 15th frame; running = anything else. Static "
            "screens are FNV-1a hashes of pictures held for 30 frames, computed the same way in both "
            "harnesses.",
            "**Reproduce.** `analysis/README.md` lists the commands; `analysis/data/` holds every input "
            "and `summary.json` every number quoted here.",
        ],
    }


SECTIONS = [section_corpus(), section_outcomes(), section_agreement(), section_cpu(), section_opcodes(),
            section_interrupts(), section_ram(), section_speed(), section_implications(), section_method()]

# --- markdown ---------------------------------------------------------------------------------


def md_table(rows):
    out = ["| " + " | ".join(rows[0]) + " |", "|" + "---|" * len(rows[0])]
    out += ["| " + " | ".join(r) + " |" for r in rows[1:]]
    return "\n".join(out)


def render_md():
    out = ["# Game Boy Homebrew Census", "", f"*matcha corpus study · {S['date']}*", "", DEK, ""]
    out += ["| " + " | ".join(v for v, _ in KPIS) + " |", "|" + "---|" * len(KPIS),
            "| " + " | ".join(t for _, t in KPIS) + " |", "", "## Key findings", ""]
    out += [f"{i}. {f}" for i, f in enumerate(FINDINGS, 1)] + [""]
    for s in SECTIONS:
        out += [f"## {s['kicker'].split(' · ')[0]}. {s['title']}", ""]
        for p in s.get("body", []):
            out += [p, ""]
        if "list" in s:
            out += [f"- {x}" for x in s["list"]] + [""]
        if "figure" in s:
            fid, title, caption = s["figure"]
            out += [f"![{title}](img/{fid}.svg)", ""]
            if caption:
                out += [f"*{caption}*", ""]
        if "table" in s:
            out += [md_table(s["table"]), ""]
        for p in s.get("after", []):
            out += [p, ""]
    return "\n".join(out)


# --- html -------------------------------------------------------------------------------------


def inline(text):
    t = html.escape(text, quote=False)
    t = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", t)
    t = re.sub(r"`(.+?)`", r"<code>\1</code>", t)
    t = re.sub(r"\[(.+?)\]\((https?://[^)]+)\)", r'<a href="\2" target="_blank" rel="noopener">\1</a>', t)
    return t


NUMERIC = re.compile(r"[\d,.%×]+( of [\d,]+)?")


def html_table(rows):
    """Columns whose every cell is a number are right-aligned."""
    num = [all(NUMERIC.fullmatch(r[i]) for r in rows[1:]) for i in range(len(rows[0]))]
    attr = [' class="right"' if x else "" for x in num]
    cell = lambda tag, i, c: f"<{tag}{attr[i]}>{inline(c)}</{tag}>"
    head = "".join(cell("th", i, c) for i, c in enumerate(rows[0]))
    body = "".join("<tr>" + "".join(cell("td", i, c) for i, c in enumerate(r)) + "</tr>" for r in rows[1:])
    return f'<div class="table-wrap"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>'


def render_section(s):
    parts = [f'<section id="{s["id"]}"><div class="stack"><span class="num">{html.escape(s["kicker"])}</span>'
             f'<h2>{inline(s["title"])}</h2></div>']
    for p in s.get("body", []):
        parts.append(f'<p class="prose">{inline(p)}</p>')
    if "list" in s:
        parts.append('<ul class="findings">' + "".join(f"<li><span>{inline(x)}</span></li>" for x in s["list"]) + "</ul>")
    if "figure" in s:
        fid, title, caption = s["figure"]
        cap = f"<p>{inline(caption)}</p>" if caption else ""
        parts.append(f'<figure><figcaption><h3>{inline(title)}</h3>{cap}</figcaption>'
                     f'<div class="legend" id="legend-{fid}"></div><div class="chart" id="chart-{fid}"></div>'
                     f'<details class="data"><summary>Show the data</summary><div class="table-wrap">'
                     f'<table id="table-{fid}"></table></div></details></figure>')
    if "table" in s:
        parts.append(html_table(s["table"]))
    for p in s.get("after", []):
        parts.append(f'<p class="prose">{inline(p)}</p>')
    parts.append("</section>")
    return "\n".join(parts)


def render_html():
    kpis = "".join(f'<div class="kpi"><b>{html.escape(v)}</b><span>{html.escape(t)}</span></div>' for v, t in KPIS)
    findings = ('<section id="findings"><div class="stack"><span class="num">Key findings</span></div>'
                '<ol class="findings">' + "".join(f"<li><span>{inline(f)}</span></li>" for f in FINDINGS) +
                "</ol></section>")
    sections = findings + "\n" + "\n".join(render_section(s) for s in SECTIONS)
    data = {k: S[k] for k in ("corpus", "cpu", "opcodes", "interrupts", "bench")}
    data["corpus"] = {"per_year": S["corpus"]["per_year"]}
    footer = ("Made with matcha, a cycle-accurate Game Boy emulator. Data: Homebrew Hub (gbdev/database); "
              "reference runs: SameBoy. Every figure on this page is generated from analysis/data/summary.json.")
    out = (PAGE / "template.html").read_text()
    for key, value in {
        "date": html.escape(S["date"]),
        "dek": inline(DEK),
        "kpis": kpis,
        "sections": sections,
        "footer": html.escape(footer),
        "data": json.dumps(data, separators=(",", ":")).replace("</", "<\\/"),
        "charts_js": (PAGE / "charts.js").read_text(),
        "page_js": (PAGE / "page.js").read_text(),
    }.items():
        out = out.replace("{{" + key + "}}", value)
    assert "{{" not in out, "unfilled placeholder"
    return out


DOCUMENT_HEAD = ('<!doctype html>\n<html lang="en">\n<meta charset="utf-8">\n'
                 '<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">\n')


def main():
    (ROOT / "docs/analysis.md").write_text(render_md())
    (ROOT / "dist").mkdir(exist_ok=True)
    page = render_html()
    (ROOT / "dist/analysis.fragment.html").write_text(page)
    (ROOT / "dist/analysis.html").write_text(DOCUMENT_HEAD + page)
    print("docs/analysis.md, dist/analysis.html, dist/analysis.fragment.html")


if __name__ == "__main__":
    main()
