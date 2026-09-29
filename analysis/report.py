#!/usr/bin/env python3
"""Corpus analysis: what homebrew Game Boy software does with the machine,
and how closely matcha agrees with SameBoy while running it.

    python3 analysis/report.py

Inputs (analysis/data/):
  corpus.csv              build_manifest.py: one row per Homebrew Hub entry
  profiles.json           run_profiles.sh: matcha, 60 s of monkey input per ROM
  reference.jsonl         reference/run.py zero: SameBoy, same input, RAM zeroed
  reference_random.jsonl  reference/run.py random: SameBoy, noisy power-on RAM
  history.csv             history.py: outcomes under earlier matcha versions
  stop_key1.json          stop_key1.py: do the programs stuck in STOP write KEY1 first?
  bench.csv               bench.py: emulation speed without profiling
  opcodes.json            opcode_names.mjs: mnemonics

Outputs:
  analysis/data/summary.json  every number quoted in docs/analysis.md
  analysis/data/roms.csv      one merged row per profiled ROM
  docs/img/*.svg              the report's charts
"""

import datetime
import json
import math
from pathlib import Path

import numpy as np
import pandas as pd

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "analysis/data"
IMG = ROOT / "docs/img"

W, H = 160, 144
SECONDS = 60
OUTCOMES = ["running", "static", "blank", "crashed"]
TOOLCHAINS = ["GB Studio 3+", "GBDK-2020 (C)", "other / unknown"]
INTERRUPTS = ["vblank", "stat", "timer", "serial", "joypad"]
REGIONS = ["rom0", "romx", "vram", "sram", "wram", "echo", "oam", "io", "hram", "ie"]


def fnv1a(data: bytes) -> str:
    h = 0xCBF29CE484222325
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def boot_screen_hash() -> str:
    """Screen right after the DMG boot ROM, for a cartridge with the standard
    logo, rendered independently of both emulators (tiles doubled from the
    header logo, BGP = 0xFC). Both emulators must produce this exact hash."""
    logo = (ROOT / "web/roms/libbet.gb").read_bytes()[0x104:0x134]
    fb = bytearray(W * H)

    def put_tile_row(tile_x, tile_y, row, bits):
        for i in range(8):
            if bits >> (7 - i) & 1:
                fb[(tile_y * 8 + row) * W + tile_x * 8 + i] = 3  # colour 1 -> shade 3

    for k, byte in enumerate(logo):
        tile = k // 2  # tiles 1..24 in two rows of 12 at map (4..15, 8..9)
        tx, ty = 4 + tile % 12, 8 + tile // 12
        for half, nibble in enumerate((byte >> 4, byte & 0x0F)):
            bits = 0
            for i in range(4):
                bits = (bits << 2) | ((nibble >> (3 - i)) & 1) * 3
            base = (k % 2) * 4 + half * 2
            put_tile_row(tx, ty, base, bits)
            put_tile_row(tx, ty, base + 1, bits)
    for row, bits in enumerate([0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C]):
        put_tile_row(16, 8, row, bits)  # (R) at map (16, 8)
    return fnv1a(bytes(fb))


WHITE_SCREEN = fnv1a(bytes(W * H))


def generic_mnemonic(text: str) -> str:
    """'jr nz, $0002' -> 'jr nz, e8': operand placeholders in Pan Docs style."""
    if text.startswith("rst"):
        return text
    text = text.replace("[$ff00]", "[$ff00+n8]").replace("sp+0", "sp+e8").replace("add sp, 0", "add sp, e8")
    text = text.replace("$0002", "e8").replace("[$0000]", "[a16]")
    text = text.replace("$0000", "a16" if text.startswith(("jp", "call")) else "n16")
    return text.replace("$00", "n8")


# A program that never draws can still show the boot logo for a frame before it turns the LCD
# off; how many such frames appear depends on the boot ROM's hand-over point, which differs
# between SameBoy's boot ROM and Nintendo's. Up to two non-uniform frames still count as blank.
BLANK_MAX_FRAMES = 2


def outcome(locked, nonblank, distinct):
    if locked:
        return "crashed"
    if nonblank <= BLANK_MAX_FRAMES:
        return "blank"
    if distinct <= 1:
        return "static"
    return "running"


def load():
    corpus = pd.read_csv(DATA / "corpus.csv", keep_default_na=False)
    corpus["year"] = pd.to_numeric(corpus["year"], errors="coerce")
    prof = pd.DataFrame(json.loads((DATA / "profiles.json").read_text()))
    prof = prof.rename(columns={"rom": "rom_path"})
    prof["m_outcome"] = [outcome(l is not None and not pd.isna(l), n, d) for l, n, d in
                         zip(prof.locked_opcode, prof.nonblank_frames, prof.distinct_frames_sampled)]
    keep = ["slug", "title", "developer", "platform", "typetag", "year", "cgb_mode", "mapper",
            "cart_type_name", "rom_path", "rom_bytes", "battery", "header_checksum_ok", "logo_ok", "toolchain"]
    df = corpus[corpus.dmg_runnable == 1][keep].merge(
        prof.drop(columns=["title", "cart_type", "cart_type_name", "cgb_flag", "sgb_flag",
                           "header_checksum_ok", "rom_bytes"]), on="rom_path", how="left", validate="one_to_one")
    assert df.instructions.notna().all(), "every runnable ROM needs a profile"
    for name, col in [("reference.jsonl", "z"), ("reference_random.jsonl", "r")]:
        path = DATA / name
        if not path.exists():
            continue
        ref = pd.read_json(path, lines=True, dtype=False)
        ref[f"{col}_outcome"] = [outcome(l, n, d) for l, n, d in
                                 zip(ref.locked, ref.nonblank_frames, ref.distinct_frames_sampled)]
        ref = ref.rename(columns={c: f"{col}_{c}" for c in ref.columns if c not in ("rom", f"{col}_outcome")})
        df = df.merge(ref.rename(columns={"rom": "rom_path"}), on="rom_path", how="left", validate="one_to_one")
    return corpus, df


def jaccard(a, b):
    a, b = set(a), set(b)
    return len(a & b) / len(a | b) if a | b else 1.0


def pct(n, d):
    return round(100.0 * n / d, 2) if d else None


def fingerprint_check(corpus):
    """Byte-signature toolchains against what the authors tagged, and against dates."""
    fp = corpus.toolchain.isin(TOOLCHAINS[:2])
    tagged = {"GB Studio": TOOLCHAINS[0], "GBDK": TOOLCHAINS[1]}
    dated = corpus.dropna(subset=["year"])
    modern = dated[dated.year >= 2019]
    return {
        "tagged": {tag: {"entries": int((corpus.tool_tag == tag).sum()),
                         "fingerprinted_as_tagged": int(((corpus.tool_tag == tag) & (corpus.toolchain == tc)).sum())}
                   for tag, tc in tagged.items()},
        "fingerprinted_before_2019": int(fp[corpus.year < 2019].sum()),
        "dated_before_2019": int((dated.year < 2019).sum()),
        "fingerprinted_share_2019_on_pct": pct(modern.toolchain.isin(TOOLCHAINS[:2]).sum(), len(modern)),
    }


def corpus_section(corpus):
    n = len(corpus)
    years = corpus.year.dropna()
    era = pd.cut(years, [0, 2005, 2018, 3000], labels=["1995-2005", "2006-2018", "2019-"])
    first = corpus[(corpus.year >= 1997) & (corpus.year <= 2002)]
    second = corpus[corpus.year >= 2019]
    return {
        "fingerprints": fingerprint_check(corpus),
        "waves": {
            "1997-2002": {"n": len(first), "color_pct": pct(first.cgb_mode.isin(["CGB enhanced", "CGB only"]).sum(),
                                                           len(first)),
                          "demo_pct": pct((first.typetag == "demo").sum(), len(first))},
            "2019-": {"n": len(second), "competition_pct": pct((second.event_tags != "").sum(), len(second)),
                      "game_pct": pct((second.typetag == "game").sum(), len(second))},
        },
        "entries_with_rom": n,
        "hardware": corpus.cgb_mode.value_counts().to_dict(),
        "cgb_only_pct": pct((corpus.cgb_mode == "CGB only").sum(), n),
        "mapper": corpus.mapper.value_counts().to_dict(),
        "unsupported_mapper": int((corpus.mapper_supported == 0).sum()),
        "unsupported_mapper_types": corpus[corpus.mapper_supported == 0].cart_type_name.value_counts().to_dict(),
        "unsupported_mapper_with_valid_logo": int(((corpus.mapper_supported == 0) & (corpus.logo_ok == 1)).sum()),
        "sgb_flag": int((corpus.sgb == 1).sum()),
        "toolchain": corpus.toolchain.value_counts().to_dict(),
        "typetag": corpus.typetag.value_counts().to_dict(),
        "dated": int(years.size),
        "by_era": era.value_counts().sort_index().to_dict(),
        "peak_year": int(years.value_counts().idxmax()),
        "peak_year_count": int(years.value_counts().max()),
        "boots_on_dmg": int(((corpus.logo_ok == 1) & (corpus.header_checksum_ok == 1)).sum()),
        "bad_logo": int((corpus.logo_ok == 0).sum()),
        "bad_header_checksum": int((corpus.header_checksum_ok == 0).sum()),
        "dmg_runnable": int(corpus.dmg_runnable.sum()),
    }


def year_chart_data(corpus):
    y = corpus.dropna(subset=["year"])
    t = pd.crosstab(y.year.astype(int), y.cgb_mode).reindex(columns=["DMG", "CGB enhanced", "CGB only"], fill_value=0)
    t = t.reindex(range(int(y.year.min()), int(y.year.max()) + 1), fill_value=0)
    return {"years": t.index.tolist(), **{c: t[c].tolist() for c in t.columns}}


def compat_section(df):
    n = len(df)
    out = {"profiled": n, "matcha": df.m_outcome.value_counts().reindex(OUTCOMES, fill_value=0).to_dict()}
    statics = df.static_screens.map(set)
    boot = boot_screen_hash()
    out["boot_screen_hash"] = boot
    only_boot = statics.map(lambda s: bool(s) and s <= {boot, WHITE_SCREEN})
    out["logo_only"] = int(((df.m_outcome == "static") & only_boot).sum())
    by_hw = pd.crosstab(df.cgb_mode, df.m_outcome).reindex(columns=OUTCOMES, fill_value=0)
    out["by_hardware"] = {k: v for k, v in by_hw.to_dict(orient="index").items()}
    bad = df[(df.logo_ok == 0) | (df.header_checksum_ok == 0)]
    out["bad_header"] = {"count": len(bad), "outcomes": bad.m_outcome.value_counts().to_dict()}
    stop_bound = df.stopped_cycles / df.total_cycles > 0.5
    out["stuck_in_stop"] = int(stop_bound.sum())
    out["stuck_in_stop_cgb_flag"] = int((stop_bound & (df.cgb_mode == "CGB enhanced")).sum())
    key1 = DATA / "stop_key1.json"
    if key1.exists():
        check = json.loads(key1.read_text())
        assert check["stuck_in_stop"] == out["stuck_in_stop"], "stop_key1.json is stale: rerun stop_key1.py"
        out["stuck_in_stop_wrote_key1"] = check["wrote_key1_before_stop"]
    return out


def agreement_section(df):
    if "z_outcome" not in df:
        return None
    n = len(df)
    same = df.m_outcome == df.z_outcome
    both = df.static_screens.map(len).gt(0) & df.z_static_screens.map(len).gt(0)
    jac = pd.Series([jaccard(a, b) for a, b in zip(df.static_screens, df.z_static_screens)], index=df.index)
    shared = pd.Series([bool(set(a) & set(b)) for a, b in zip(df.static_screens, df.z_static_screens)], index=df.index)
    cm = pd.crosstab(df.m_outcome, df.z_outcome).reindex(index=OUTCOMES, columns=OUTCOMES, fill_value=0)
    disagree = df[~same][["slug", "cgb_mode", "m_outcome", "z_outcome"]]
    out = {
        "outcome_agreement_pct": pct(same.sum(), n),
        "outcome_agree": int(same.sum()),
        "sameboy": df.z_outcome.value_counts().reindex(OUTCOMES, fill_value=0).to_dict(),
        "confusion": {k: v for k, v in cm.to_dict(orient="index").items()},
        "with_static_screens_both": int(both.sum()),
        "static_sets_identical": int((both & (jac == 1.0)).sum()),
        "static_share_one": int((both & shared).sum()),
        "static_mean_jaccard": round(float(jac[both].mean()), 3),
        "static_screens_total_matcha": int(df.static_screens.map(len).sum()),
        "static_screens_found_in_sameboy": int(sum(len(set(a) & set(b)) for a, b in
                                                   zip(df.static_screens, df.z_static_screens))),
        "disagreements": disagree.to_dict(orient="records"),
    }
    # The frame counters agree to within a frame or two when both show the same thing.
    diff = (df.lcd_on_frames - df.z_lcd_on_frames).abs()
    out["lcd_on_frames_median_abs_diff"] = float(diff.median())
    return out


# matcha versions in history.csv (history.py), oldest first, and what each one changed.
VERSIONS = {
    "22c3fcc": "before the comparison",
    "9e9b8be": "+ boot logo in VRAM, DMG STOP",
    "2f864c7": "+ OAM DMA bus conflicts",
}


def history_section(df):
    path = DATA / "history.csv"
    if "z_outcome" not in df or not path.exists():
        return None
    hist = pd.read_csv(path).merge(df[["rom_path", "m_outcome", "z_outcome"]], on="rom_path", validate="one_to_one")
    assert len(hist) == len(df), "history.csv must cover every profiled ROM"
    commits = [c for c in hist.columns if c in VERSIONS]
    assert (hist[commits[-1]] == hist.m_outcome).all(), "the newest history column must match profiles.json"
    return {"agreement": [[f"`{c}` {VERSIONS[c]}", int((hist[c] == hist.z_outcome).sum())] for c in commits]}


def ram_section(df):
    if "r_outcome" not in df:
        return None
    changed = df.z_outcome != df.r_outcome
    jac = pd.Series([jaccard(a, b) for a, b in zip(df.z_static_screens, df.r_static_screens)], index=df.index)
    screens_changed = jac < 1.0
    rows = df[changed][["slug", "year", "toolchain", "z_outcome", "r_outcome"]]
    return {
        "outcome_changed": int(changed.sum()),
        "outcome_changed_pct": pct(changed.sum(), len(df)),
        "static_screens_changed": int(screens_changed.sum()),
        "fixed_by_noise": int(((df.z_outcome != "running") & (df.r_outcome == "running")).sum()),
        "broken_by_noise": int(((df.z_outcome == "running") & (df.r_outcome != "running")).sum()),
        "transitions": rows.groupby(["z_outcome", "r_outcome"]).size().rename("n").reset_index().to_dict(orient="records"),
        "examples": rows.head(20).to_dict(orient="records"),
    }


def cpu_section(df):
    ok = df[df.m_outcome != "crashed"].copy()
    ok["util"] = ok.cpu_utilization
    ok["never_halts"] = ok.halted_cycles == 0
    by_tc = ok.groupby("toolchain").agg(n=("util", "size"), median_util=("util", "median"),
                                        never_halts=("never_halts", "mean"))
    old = ok[ok.year <= 2005]
    new = ok[ok.year >= 2019]
    hist_edges = np.linspace(0, 1, 21)
    hist = {tc: np.histogram(ok[ok.toolchain == tc].util, bins=hist_edges)[0].tolist() for tc in TOOLCHAINS}
    return {
        "n": len(ok),
        "median_util": round(float(ok.util.median()), 4),
        "mean_util": round(float(ok.util.mean()), 4),
        "never_halts_pct": pct(ok.never_halts.sum(), len(ok)),
        "busy_over_95_pct": pct((ok.util > 0.95).sum(), len(ok)),
        "by_toolchain": {tc: {"n": int(r.n), "median_util": round(float(r.median_util), 4),
                              "never_halts_pct": round(100 * float(r.never_halts), 2)}
                         for tc, r in by_tc.iterrows()},
        "era": {"1995-2005": {"n": len(old), "median_util": round(float(old.util.median()), 4),
                              "never_halts_pct": pct(old.never_halts.sum(), len(old))},
                "2019-": {"n": len(new), "median_util": round(float(new.util.median()), 4),
                          "never_halts_pct": pct(new.never_halts.sum(), len(new))}},
        "hist_edges": hist_edges.round(2).tolist(),
        "hist": hist,
    }


def opcode_section(df, names):
    ran = df[df.instructions > 0]
    base = np.array(ran.opcodes.tolist(), dtype=float)  # [rom][256], 0xCB counts prefixed ones
    cb = np.array(ran.cb_opcodes.tolist(), dtype=float)
    share = base / base.sum(axis=1, keepdims=True)  # each ROM weighs the same
    mix = share.mean(axis=0)
    order = np.argsort(-mix)
    cum = np.cumsum(mix[order])
    need = {str(q): int(np.searchsorted(cum, q / 100) + 1) for q in (50, 80, 90, 99)}
    label = lambda op: "prefix cb" if op == 0xCB else generic_mnemonic(names["base"][op])
    top = [{"opcode": f"{op:02x}", "text": label(op), "share": round(float(mix[op]) * 100, 2)} for op in order[:15]]
    used = (base > 0).sum(axis=0)
    never = [f"{op:02x}" for op in range(256) if used[op] == 0 and not names["base"][op].startswith("db ")]
    cb_share = cb / np.maximum(cb.sum(axis=1, keepdims=True), 1)
    cb_mix = cb_share[cb.sum(axis=1) > 0].mean(axis=0)
    cb_top = [{"opcode": f"cb {op:02x}", "text": names["cb"][op], "share_of_cb": round(float(cb_mix[op]) * 100, 2)}
              for op in np.argsort(-cb_mix)[:8]]
    by_tc, sdcc_rank = {}, {}
    for tc in TOOLCHAINS:
        s = share[(ran.toolchain == tc).values].mean(axis=0)
        ranked = list(np.argsort(-s))
        by_tc[tc] = [{"text": label(op), "share": round(float(s[op]) * 100, 2)} for op in ranked[:5]]
        sdcc_rank[tc] = ranked.index(0xF8) + 1  # ld hl, sp+e8: how SDCC-compiled C reaches locals
    return {
        "roms": len(ran),
        "instructions_total": int(ran.instructions.sum()),
        "opcodes_for_share": need,
        "top": top,
        "cumulative": [round(float(c) * 100, 2) for c in cum],
        "cb_prefix_share": round(float(mix[0xCB]) * 100, 2),
        "cb_top": cb_top,
        "never_executed": never,
        "top_by_toolchain": by_tc,
        "ld_hl_sp_e8_rank": sdcc_rank,
    }


def interrupt_section(df):
    ran = df[df.m_outcome != "crashed"]
    counts = pd.DataFrame(ran.interrupts.tolist(), index=ran.index)
    uses = {k: pct((counts[k] > 0).sum(), len(ran)) for k in INTERRUPTS}
    by_tc = {tc: {k: pct((counts[k][ran.toolchain == tc] > 0).sum(), (ran.toolchain == tc).sum()) for k in INTERRUPTS}
             for tc in TOOLCHAINS}
    per_sec = (counts["stat"] / SECONDS)
    return {"n": len(ran), "uses_pct": uses, "by_toolchain": by_tc,
            "no_interrupts_pct": pct((counts.sum(axis=1) == 0).sum(), len(ran)),
            "stat_median_per_second_among_users": round(float(per_sec[counts["stat"] > 0].median()), 1)}


def memory_section(df):
    ran = df[df.instructions > 0]
    reads = pd.DataFrame(ran.reads.tolist(), index=ran.index)[REGIONS]
    writes = pd.DataFrame(ran.writes.tolist(), index=ran.index)[REGIONS]
    rshare = reads.div(reads.sum(axis=1), axis=0).mean()
    wshare = writes.div(writes.sum(axis=1).replace(0, np.nan), axis=0).mean()
    cover = ran.covered_rom_bytes / ran.rom_bytes
    return {
        "read_share_pct": (rshare * 100).round(2).to_dict(),
        "write_share_pct": (wshare * 100).round(2).to_dict(),
        "echo_ram_roms": int(((reads.echo + writes.echo) > 0).sum()),
        "direct_oam_writers_pct": pct((writes.oam > 0).sum(), len(ran)),
        "sram_writers": int((writes.sram > 0).sum()),
        "battery_carts": int((ran.battery == 1).sum()),
        "coverage_median_pct": round(float(cover.median() * 100), 1),
        "coverage_median_bytes": int(ran.covered_rom_bytes.median()),
    }


def bench_section(df):
    path = DATA / "bench.csv"
    if not path.exists():
        return None
    b = pd.read_csv(path).merge(df[["rom_path", "title", "cpu_utilization", "toolchain"]], on="rom_path")
    corr = float(np.corrcoef(b.cpu_utilization, b.speed)[0, 1])
    lo = b[b.cpu_utilization < 0.3].speed.median()
    hi = b[b.cpu_utilization > 0.95].speed.median()
    return {"n": len(b), "median_speed": round(float(b.speed.median()), 1),
            "p10_speed": round(float(b.speed.quantile(0.1)), 1), "min_speed": round(float(b.speed.min()), 1),
            "corr_util_speed": round(corr, 2),
            "median_speed_util_below_30": round(float(lo), 1), "median_speed_util_above_95": round(float(hi), 1),
            "points": b[["title", "cpu_utilization", "speed", "toolchain"]].round(3).to_dict(orient="records")}


# --- charts ---------------------------------------------------------------------------------

INK, INK2, MUTED, GRID, BASE, SURFACE = "#0b0b0b", "#52514e", "#898781", "#e1e0d9", "#c3c2b7", "#fcfcfb"
SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100"]


def style(plt):
    plt.rcParams.update({
        "font.family": "sans-serif", "font.size": 10, "axes.edgecolor": BASE, "axes.labelcolor": INK2,
        "xtick.color": MUTED, "ytick.color": MUTED, "axes.titlecolor": INK, "axes.titlesize": 12,
        "axes.titleweight": "bold", "axes.titlelocation": "left", "figure.facecolor": SURFACE,
        "axes.facecolor": SURFACE, "savefig.facecolor": SURFACE, "axes.grid": True, "grid.color": GRID,
        "grid.linewidth": 0.8, "axes.spines.top": False, "axes.spines.right": False, "svg.fonttype": "none",
        "legend.frameon": False, "legend.labelcolor": INK2,
        "svg.hashsalt": "matcha-analysis",  # stable element ids: reruns give identical files
    })


def charts(summary):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import matplotlib.ticker
    style(plt)
    IMG.mkdir(parents=True, exist_ok=True)

    # 1. Entries per year by hardware target (stacked columns with a surface gap).
    yc = summary["corpus"]["per_year"]
    fig, ax = plt.subplots(figsize=(9, 3.4))
    bottom = np.zeros(len(yc["years"]))
    for key, color in zip(["DMG", "CGB enhanced", "CGB only"], SERIES):
        vals = np.array(yc[key])
        ax.bar(yc["years"], vals, bottom=bottom, width=0.7, color=color, label=key, edgecolor=SURFACE, linewidth=1)
        bottom += vals
    ax.set_title("Two eras of Game Boy homebrew", pad=14)
    ax.text(0, 1.01, f"Homebrew Hub entries with a ROM and a known year ({summary['corpus']['dated']} of "
            f"{summary['corpus']['entries_with_rom']})", transform=ax.transAxes, color=INK2, fontsize=9)
    ax.set_ylabel("entries")
    ax.grid(axis="x", visible=False)
    ax.legend(loc="upper center", ncol=3)
    fig.tight_layout()
    fig.savefig(IMG / "corpus-years.svg", metadata={"Date": None})
    plt.close(fig)

    # 2. CPU utilisation histograms, one panel per toolchain. Shares, not counts: the toolchains
    # differ eightfold in size, and one y scale keeps the panels comparable.
    cpu = summary["cpu"]
    edges = np.array(cpu["hist_edges"]) * 100
    fig, axes = plt.subplots(1, 3, figsize=(9, 2.8), sharey=True)
    for ax, tc in zip(axes, TOOLCHAINS):
        info = cpu["by_toolchain"].get(tc, {})
        shares = np.array(cpu["hist"][tc]) * 100 / max(info.get("n", 0), 1)
        ax.bar(edges[:-1] + 2.5, shares, width=4, color=SERIES[0])
        ax.set_title(f"{tc}\nn = {info.get('n', 0)} · median {info.get('median_util', 0) * 100:.0f}% busy",
                     fontsize=9.5, fontweight="normal", color=INK2, loc="left")
        ax.set_xlim(0, 100)
        ax.set_xlabel("CPU busy (% of time)")
        ax.grid(axis="x", visible=False)
    axes[0].set_ylabel("share of programs (%)")
    fig.suptitle("How hard homebrew works the CPU", x=0.01, ha="left", fontweight="bold", color=INK)
    fig.tight_layout()
    fig.savefig(IMG / "cpu-utilization.svg", metadata={"Date": None})
    plt.close(fig)

    # 3. Opcode Pareto curve.
    op = summary["opcodes"]
    cum = np.array(op["cumulative"])
    fig, ax = plt.subplots(figsize=(9, 3.2))
    ax.plot(np.arange(1, 257), cum, color=SERIES[0], linewidth=2)
    for q in ("50", "90"):
        k = op["opcodes_for_share"][q]
        ax.plot([k], [cum[k - 1]], "o", color=SERIES[0], markersize=6, markeredgecolor=SURFACE, markeredgewidth=2)
        ax.annotate(f"{k} opcodes → {q}%", (k, cum[k - 1]), xytext=(8, -14), textcoords="offset points",
                    color=INK2, fontsize=9)
    ax.set_xscale("log")
    ax.set_xlim(1, 256)
    ax.set_xticks([1, 2, 5, 10, 20, 50, 100, 256])
    ax.get_xaxis().set_major_formatter(matplotlib.ticker.ScalarFormatter())
    ax.minorticks_off()
    ax.set_ylim(0, 101)
    ax.set_xlabel("opcodes, most used first (log scale)")
    ax.set_ylabel("share of executed instructions (%)")
    ax.set_title("A few dozen opcodes do almost all the work", pad=10)
    fig.tight_layout()
    fig.savefig(IMG / "opcode-pareto.svg", metadata={"Date": None})
    plt.close(fig)

    # 4. Interrupt usage.
    it = summary["interrupts"]
    names = {"vblank": "VBlank", "stat": "STAT (LCD)", "timer": "Timer", "serial": "Serial", "joypad": "Joypad"}
    keys = sorted(INTERRUPTS, key=lambda k: it["uses_pct"][k])
    fig, ax = plt.subplots(figsize=(9, 2.6))
    vals = [it["uses_pct"][k] for k in keys]
    ax.barh([names[k] for k in keys], vals, height=0.5, color=SERIES[0])
    for y, v in enumerate(vals):
        ax.text(v + 1, y, f"{v:.1f}%" if v < 10 else f"{v:.0f}%", va="center", color=INK2, fontsize=9)
    ax.set_xlim(0, 105)
    ax.set_xlabel("share of ROMs that took the interrupt at least once")
    ax.grid(axis="y", visible=False)
    ax.set_title("Which interrupts homebrew relies on", pad=10)
    fig.tight_layout()
    fig.savefig(IMG / "interrupts.svg", metadata={"Date": None})
    plt.close(fig)

    # 5. Speed vs utilisation.
    b = summary.get("bench")
    if b:
        fig, ax = plt.subplots(figsize=(9, 3.4))
        pts = pd.DataFrame(b["points"])
        for tc, color in zip(TOOLCHAINS, SERIES):
            p = pts[pts.toolchain == tc]
            ax.scatter(p.cpu_utilization * 100, p.speed, s=22, color=color, label=tc, edgecolors=SURFACE, linewidths=1)
        ax.set_xlabel("CPU busy (% of time)")
        ax.set_ylabel("emulation speed (× real time)")
        ax.set_ylim(0, None)
        ax.legend(loc="upper right")
        ax.set_title("Idle games emulate fastest: HALT is skipped in bulk", pad=10)
        fig.tight_layout()
        fig.savefig(IMG / "speed.svg", metadata={"Date": None})
        plt.close(fig)


def main():
    names = json.loads((DATA / "opcodes.json").read_text())
    corpus, df = load()
    summary = {"date": datetime.date.today().isoformat(), "corpus": corpus_section(corpus)}
    summary["corpus"]["per_year"] = year_chart_data(corpus)
    summary["compatibility"] = compat_section(df)
    summary["agreement"] = agreement_section(df)
    summary["history"] = history_section(df)
    summary["ram"] = ram_section(df)
    summary["cpu"] = cpu_section(df)
    summary["opcodes"] = opcode_section(df, names)
    summary["interrupts"] = interrupt_section(df)
    summary["memory"] = memory_section(df)
    summary["bench"] = bench_section(df)

    cols = ["slug", "title", "developer", "year", "typetag", "cgb_mode", "mapper", "toolchain", "logo_ok",
            "header_checksum_ok", "m_outcome", "cpu_utilization", "instructions", "covered_rom_bytes",
            "lcd_on_frames", "nonblank_frames", "distinct_frames_sampled"]
    cols += [c for c in ("z_outcome", "r_outcome") if c in df]
    df[cols].to_csv(DATA / "roms.csv", index=False)
    # Missing years are pandas NaN internally; JSON consumers require null.
    # Keep the serializer strict so new non-finite values cannot leak again.
    summary = json_safe(summary)
    (DATA / "summary.json").write_text(json.dumps(summary, indent=1, default=int, allow_nan=False) + "\n")
    charts(summary)
    print(json.dumps({k: v for k, v in summary.items() if k not in ("opcodes",)}, indent=1, default=int)[:6000])


def json_safe(value):
    if isinstance(value, dict):
        return {k: json_safe(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_safe(v) for v in value]
    if isinstance(value, float) and not math.isfinite(value):
        return None
    return value


if __name__ == "__main__":
    main()
