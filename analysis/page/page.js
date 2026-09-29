// Mounts the report's charts from the embedded summary data.
"use strict";

const DATA = JSON.parse(document.getElementById("data").textContent);
const SERIES = ["var(--s1)", "var(--s2)", "var(--s3)"];
const TOOLCHAINS = ["GB Studio 3+", "GBDK-2020 (C)", "other / unknown"];
const byId = (id) => document.getElementById(id);

// 01 · entries per year by hardware target
{
  const py = DATA.corpus.per_year;
  const series = ["DMG", "CGB enhanced", "CGB only"].map((name, i) => ({ name, values: py[name], color: SERIES[i] }));
  legend(byId("legend-corpus-years"), series);
  stackedColumns(byId("chart-corpus-years"), { categories: py.years, series, yLabel: "entries" });
  table(byId("table-corpus-years"), [["Year", ...series.map((s) => s.name)], ...py.years.map((y, i) => [String(y), ...series.map((s) => String(s.values[i]))])]);
}

// 04 · CPU utilisation histograms, one per toolchain (same y scale)
{
  const cpu = DATA.cpu;
  const holder = byId("chart-cpu-utilization");
  holder.classList.add("multiples");
  // Shares, not counts: the toolchains differ eightfold in size, and one y scale keeps them comparable.
  const shares = Object.fromEntries(TOOLCHAINS.map((t) => [t, cpu.hist[t].map((c) => (100 * c) / cpu.by_toolchain[t].n)]));
  const maxY = Math.max(...TOOLCHAINS.flatMap((t) => shares[t]));
  const edges = cpu.hist_edges.map((e) => Math.round(e * 100));
  for (const tc of TOOLCHAINS) {
    const info = cpu.by_toolchain[tc];
    const panel = document.createElement("div");
    const title = document.createElement("div");
    title.className = "panel-title";
    title.textContent = tc;
    const sub = document.createElement("div");
    sub.className = "panel-sub";
    sub.textContent = `${info.n} programs · median ${Math.round(info.median_util * 100)}% busy · ${Math.round(info.never_halts_pct)}% never halt`;
    const chart = document.createElement("div");
    chart.className = "chart";
    panel.append(title, sub, chart);
    holder.append(panel);
    histogram(chart, { shares: shares[tc], counts: cpu.hist[tc], edges, color: SERIES[0], xLabel: "CPU busy", maxY });
  }
  table(byId("table-cpu-utilization"), [["CPU busy", ...TOOLCHAINS], ...edges.slice(0, -1).map((e, i) => [`${e}–${edges[i + 1]}%`, ...TOOLCHAINS.map((t) => `${cpu.hist[t][i]} (${shares[t][i].toFixed(1)}%)`)])]);
}

// 05 · opcode Pareto curve
{
  const op = DATA.opcodes;
  const labels = op.top.map((t) => t.text);
  const milestones = ["50", "90"].map((q) => [op.opcodes_for_share[q], q]);
  paretoLine(byId("chart-opcode-pareto"), { cumulative: op.cumulative, milestones, color: SERIES[0], labels });
  table(byId("table-opcode-pareto"), [["Rank", "Instruction", "Share", "Cumulative"], ...op.top.map((t, i) => [String(i + 1), t.text, `${t.share.toFixed(2)}%`, `${op.cumulative[i].toFixed(1)}%`])]);
}

// 06 · interrupts
{
  const names = { vblank: "VBlank", stat: "STAT (LCD)", timer: "Timer", serial: "Serial", joypad: "Joypad" };
  const it = DATA.interrupts;
  const items = Object.keys(names)
    .map((k) => ({ label: names[k], value: it.uses_pct[k], note: "of programs" }))
    .sort((a, b) => b.value - a.value);
  hbars(byId("chart-interrupts"), { items, color: SERIES[0], format: (v) => `${v.toFixed(v < 10 ? 1 : 0)}%` });
  const p1 = (v) => `${v.toFixed(1)}%`;
  table(byId("table-interrupts"), [["Interrupt", "All programs", ...TOOLCHAINS], ...Object.keys(names).map((k) => [names[k], p1(it.uses_pct[k]), ...TOOLCHAINS.map((t) => p1(it.by_toolchain[t][k]))])]);
}

// 08 · speed against CPU load
{
  const b = DATA.bench;
  const series = TOOLCHAINS.map((name, i) => ({ name, color: SERIES[i] }));
  legend(byId("legend-speed"), series, "dot");
  const points = b.points.map((p) => ({ x: p.cpu_utilization * 100, y: p.speed, s: p.toolchain, name: p.title }));
  scatter(byId("chart-speed"), { points, series, xLabel: "CPU busy (% of time)", yLabel: "× real time" });
  table(byId("table-speed"), [["Program", "Toolchain", "CPU busy", "Speed"], ...b.points.slice().sort((p, q) => q.speed - p.speed).map((p) => [p.title, p.toolchain, `${(p.cpu_utilization * 100).toFixed(0)}%`, `${p.speed.toFixed(1)}×`])]);
}
