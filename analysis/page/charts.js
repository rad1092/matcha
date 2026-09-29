// Small SVG chart kit for the corpus report page. Every chart renders at its
// container's pixel width (text stays legible on phones), re-renders on resize,
// and pairs hover/focus tooltips with a table view in the surrounding <figure>.
"use strict";

const SVG_NS = "http://www.w3.org/2000/svg";

function svgEl(tag, attrs = {}, parent) {
  const e = document.createElementNS(SVG_NS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  if (parent) parent.append(e);
  return e;
}

function text(parent, x, y, str, attrs = {}) {
  const t = svgEl("text", { x, y, ...attrs }, parent);
  t.textContent = str;
  return t;
}

const fmt = {
  int: (v) => Math.round(v).toLocaleString("en-US"),
  pct: (v, d = 0) => `${v.toFixed(d)}%`,
  x: (v) => `${v.toFixed(0)}×`,
};

/** Rounded-top bar path: square at the baseline, 4px radius at the data end. */
function barPath(x, y, w, h, r = 4) {
  if (h <= 0) return "";
  r = Math.min(r, w / 2, h);
  return `M${x},${y + h}V${y + r}Q${x},${y} ${x + r},${y}H${x + w - r}Q${x + w},${y} ${x + w},${y + r}V${y + h}Z`;
}

function hbarPath(x, y, w, h, r = 4) {
  if (w <= 0) return "";
  r = Math.min(r, h / 2, w);
  return `M${x},${y}H${x + w - r}Q${x + w},${y} ${x + w},${y + r}V${y + h - r}Q${x + w},${y + h} ${x + w - r},${y + h}H${x}Z`;
}

function niceTicks(max, count = 4) {
  const raw = max / count;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw);
  const ticks = [];
  for (let v = 0; v <= max + step * 0.001; v += step) ticks.push(v);
  if (ticks[ticks.length - 1] < max) ticks.push(ticks[ticks.length - 1] + step);
  return ticks;
}

// --- tooltip ---------------------------------------------------------------------

const tip = document.createElement("div");
tip.className = "tip";
tip.hidden = true;
document.body.append(tip);

/** rows: [{ value, label, color? }] — value leads, label follows. */
function showTip(evt, title, rows) {
  tip.replaceChildren();
  if (title) {
    const h = document.createElement("div");
    h.className = "tip-title";
    h.textContent = title;
    tip.append(h);
  }
  for (const r of rows) {
    const row = document.createElement("div");
    row.className = "tip-row";
    if (r.color) {
      const key = document.createElement("i");
      key.style.background = r.color;
      row.append(key);
    }
    const b = document.createElement("b");
    b.textContent = r.value;
    const s = document.createElement("span");
    s.textContent = r.label;
    row.append(b, s);
    tip.append(row);
  }
  tip.hidden = false;
  let x, y;
  if (evt.clientX !== undefined && evt.type !== "focus") {
    x = evt.clientX;
    y = evt.clientY;
  } else {
    const r = evt.target.getBoundingClientRect();
    x = r.left + r.width / 2;
    y = r.top;
  }
  const w = tip.offsetWidth, h = tip.offsetHeight;
  tip.style.left = `${Math.min(Math.max(8, x + 14), innerWidth - w - 8)}px`;
  tip.style.top = `${Math.max(8, y - h - 12)}px`;
}

function hideTip() {
  tip.hidden = true;
}

/** Wires hover + keyboard focus on a mark (or its larger hit area). */
function hover(node, title, rows) {
  node.setAttribute("tabindex", "0");
  node.classList.add("hit");
  node.setAttribute("aria-label", `${title}: ${rows.map((r) => `${r.label} ${r.value}`).join(", ")}`);
  const on = (e) => showTip(e, title, rows);
  node.addEventListener("pointermove", on);
  node.addEventListener("focus", on);
  node.addEventListener("pointerleave", hideTip);
  node.addEventListener("blur", hideTip);
}

// --- responsive mount --------------------------------------------------------------

function mount(container, render) {
  let last = 0;
  const draw = () => {
    const w = Math.floor(container.clientWidth);
    if (!w || w === last) return;
    last = w;
    container.replaceChildren();
    render(w);
  };
  new ResizeObserver(draw).observe(container);
  draw();
}

function axisY(g, { x0, x1, scale, ticks, format }) {
  for (const t of ticks) {
    const y = scale(t);
    svgEl("line", { x1: x0, x2: x1, y1: y, y2: y, class: t === 0 ? "baseline" : "grid" }, g);
    text(g, x0 - 8, y + 4, format(t), { class: "tick", "text-anchor": "end" });
  }
}

// --- charts ------------------------------------------------------------------------

/** Stacked columns: categories along x, series stacked (2px surface gap). */
function stackedColumns(container, { categories, series, yLabel, labelEvery = 5 }) {
  mount(container, (W) => {
    const H = 260, m = { t: 12, r: 8, b: 28, l: 44 };
    const svg = svgEl("svg", { width: W, height: H, role: "img" }, container);
    const totals = categories.map((_, i) => series.reduce((a, s) => a + s.values[i], 0));
    const ticks = niceTicks(Math.max(...totals));
    const y = (v) => m.t + (H - m.t - m.b) * (1 - v / ticks[ticks.length - 1]);
    axisY(svg, { x0: m.l, x1: W - m.r, scale: y, ticks, format: fmt.int });
    text(svg, 4, m.t + 2, yLabel, { class: "axis-label" });
    const band = (W - m.l - m.r) / categories.length;
    const bw = Math.min(24, band * 0.72);
    categories.forEach((c, i) => {
      const x = m.l + band * i + (band - bw) / 2;
      let acc = 0;
      series.forEach((s, k) => {
        const v = s.values[i];
        if (!v) return;
        const top = y(acc + v), bottom = y(acc);
        const isTop = series.slice(k + 1).every((t) => !t.values[i]);
        const h = bottom - top - (acc > 0 ? 2 : 0);
        svgEl("path", { d: isTop ? barPath(x, top, bw, h) : `M${x},${top}h${bw}v${h}h${-bw}Z`, style: `fill:${s.color}` }, svg);
        acc += v;
      });
      const hit = svgEl("rect", { x: m.l + band * i, y: m.t, width: band, height: H - m.t - m.b, fill: "transparent" }, svg);
      hover(hit, String(c), [
        ...series.map((s) => ({ value: fmt.int(s.values[i]), label: s.name, color: s.color })),
        { value: fmt.int(totals[i]), label: "total" },
      ]);
      if (i % labelEvery === 0) text(svg, m.l + band * i + band / 2, H - 8, String(c), { class: "tick", "text-anchor": "middle" });
    });
  });
}

/** One histogram (share of items per bin); x in percent 0..100. */
function histogram(container, { counts, edges, color, xLabel, maxY }) {
  mount(container, (W) => {
    const H = 170, m = { t: 10, r: 8, b: 34, l: 34 };
    const svg = svgEl("svg", { width: W, height: H, role: "img" }, container);
    const ticks = niceTicks(maxY ?? Math.max(...counts), 3);
    const y = (v) => m.t + (H - m.t - m.b) * (1 - v / ticks[ticks.length - 1]);
    axisY(svg, { x0: m.l, x1: W - m.r, scale: y, ticks, format: fmt.int });
    const bw = (W - m.l - m.r) / counts.length;
    counts.forEach((n, i) => {
      const x = m.l + bw * i + 1;
      svgEl("path", { d: barPath(x, y(n), bw - 2, y(0) - y(n)), style: `fill:${color}` }, svg);
      const hit = svgEl("rect", { x: m.l + bw * i, y: m.t, width: bw, height: H - m.t - m.b, fill: "transparent" }, svg);
      hover(hit, `${edges[i]}–${edges[i + 1]}% busy`, [{ value: fmt.int(n), label: n === 1 ? "ROM" : "ROMs" }]);
    });
    for (const v of [0, 50, 100]) {
      text(svg, m.l + (W - m.l - m.r) * (v / 100), H - 18, `${v}%`, { class: "tick", "text-anchor": v === 0 ? "start" : v === 100 ? "end" : "middle" });
    }
    text(svg, m.l + (W - m.l - m.r) / 2, H - 3, xLabel, { class: "axis-label", "text-anchor": "middle" });
  });
}

/** Cumulative-share line on a log x axis, with labelled milestones. */
function paretoLine(container, { cumulative, milestones, color, labels }) {
  mount(container, (W) => {
    const H = 250, m = { t: 12, r: 16, b: 40, l: 44 };
    const svg = svgEl("svg", { width: W, height: H, role: "img" }, container);
    const n = cumulative.length;
    const x = (k) => m.l + (W - m.l - m.r) * (Math.log(k) / Math.log(n));
    const y = (v) => m.t + (H - m.t - m.b) * (1 - v / 100);
    axisY(svg, { x0: m.l, x1: W - m.r, scale: y, ticks: [0, 25, 50, 75, 100], format: (v) => `${v}%` });
    for (const k of [1, 2, 5, 10, 20, 50, 100, 256].filter((k) => k <= n)) {
      text(svg, x(k), H - 22, String(k), { class: "tick", "text-anchor": "middle" });
    }
    text(svg, m.l + (W - m.l - m.r) / 2, H - 4, "opcodes, most used first (log scale)", { class: "axis-label", "text-anchor": "middle" });
    const d = cumulative.map((v, i) => `${i ? "L" : "M"}${x(i + 1).toFixed(1)},${y(v).toFixed(1)}`).join("");
    svgEl("path", { d, class: "line", style: `stroke:${color}` }, svg);
    for (const [k, share] of milestones) {
      const cx = x(k), cy = y(cumulative[k - 1]);
      svgEl("circle", { cx, cy, r: 5, class: "ring", style: `fill:${color}` }, svg);
      const right = cx < W - 170;
      text(svg, cx + (right ? 10 : -10), cy + 18, `${k} opcodes → ${share}%`, { class: "note", "text-anchor": right ? "start" : "end" });
    }
    // Nearest-rank crosshair.
    const cross = svgEl("line", { y1: m.t, y2: H - m.b, class: "cross", visibility: "hidden" }, svg);
    const hit = svgEl("rect", { x: m.l, y: m.t, width: W - m.l - m.r, height: H - m.t - m.b, fill: "transparent" }, svg);
    hit.addEventListener("pointermove", (e) => {
      const px = e.offsetX;
      const k = Math.min(n, Math.max(1, Math.round(Math.exp(((px - m.l) / (W - m.l - m.r)) * Math.log(n)))));
      cross.setAttribute("x1", x(k));
      cross.setAttribute("x2", x(k));
      cross.setAttribute("visibility", "visible");
      showTip(e, `top ${k} opcode${k > 1 ? "s" : ""}`, [
        { value: fmt.pct(cumulative[k - 1], 1), label: "of executed instructions" },
        ...(labels[k - 1] ? [{ value: labels[k - 1], label: `#${k}` }] : []),
      ]);
    });
    hit.addEventListener("pointerleave", () => {
      cross.setAttribute("visibility", "hidden");
      hideTip();
    });
  });
}

/** Horizontal bars with the value at the tip. */
function hbars(container, { items, color, max = 100, format = fmt.pct }) {
  mount(container, (W) => {
    const row = 34, m = { t: 6, r: 48, b: 6, l: Math.min(130, W * 0.32) };
    const H = m.t + m.b + row * items.length;
    const svg = svgEl("svg", { width: W, height: H, role: "img" }, container);
    const x = (v) => m.l + (W - m.l - m.r) * (v / max);
    svgEl("line", { x1: m.l, x2: m.l, y1: m.t, y2: H - m.b, class: "baseline" }, svg);
    items.forEach((it, i) => {
      const yy = m.t + row * i;
      text(svg, m.l - 10, yy + row / 2 + 4, it.label, { class: "cat", "text-anchor": "end" });
      svgEl("path", { d: hbarPath(m.l, yy + (row - 16) / 2, x(it.value) - m.l, 16), style: `fill:${color}` }, svg);
      text(svg, x(it.value) + 6, yy + row / 2 + 4, format(it.value), { class: "value" });
      const hit = svgEl("rect", { x: 0, y: yy, width: W, height: row, fill: "transparent" }, svg);
      hover(hit, it.label, [{ value: format(it.value), label: it.note ?? "" }]);
    });
  });
}

/** Scatter with up to three series and a nearest-point tooltip. */
function scatter(container, { points, series, xLabel, yLabel }) {
  mount(container, (W) => {
    const H = 280, m = { t: 12, r: 12, b: 40, l: 44 };
    const svg = svgEl("svg", { width: W, height: H, role: "img" }, container);
    const ymax = niceTicks(Math.max(...points.map((p) => p.y)));
    const x = (v) => m.l + (W - m.l - m.r) * (v / 100);
    const y = (v) => m.t + (H - m.t - m.b) * (1 - v / ymax[ymax.length - 1]);
    axisY(svg, { x0: m.l, x1: W - m.r, scale: y, ticks: ymax, format: (v) => `${v}×` });
    for (const v of [0, 25, 50, 75, 100]) text(svg, x(v), H - 22, `${v}%`, { class: "tick", "text-anchor": "middle" });
    text(svg, m.l + (W - m.l - m.r) / 2, H - 4, xLabel, { class: "axis-label", "text-anchor": "middle" });
    text(svg, 4, m.t + 2, yLabel, { class: "axis-label" });
    const color = Object.fromEntries(series.map((s) => [s.name, s.color]));
    for (const p of points) svgEl("circle", { cx: x(p.x), cy: y(p.y), r: 4, class: "ring", style: `fill:${color[p.s]}` }, svg);
    const hit = svgEl("rect", { x: m.l, y: m.t, width: W - m.l - m.r, height: H - m.t - m.b, fill: "transparent" }, svg);
    hit.addEventListener("pointermove", (e) => {
      let best = null, bd = Infinity;
      for (const p of points) {
        const d = (x(p.x) - e.offsetX) ** 2 + (y(p.y) - e.offsetY) ** 2;
        if (d < bd) [best, bd] = [p, d];
      }
      if (!best || bd > 30 ** 2) return hideTip();
      showTip(e, best.name, [
        { value: fmt.x(best.y), label: "real time", color: color[best.s] },
        { value: fmt.pct(best.x), label: "CPU busy" },
        { value: best.s, label: "" },
      ]);
    });
    hit.addEventListener("pointerleave", hideTip);
  });
}

/** Legend row: swatch + label per series. */
function legend(container, series, shape = "rect") {
  for (const s of series) {
    const item = document.createElement("span");
    item.className = "legend-item";
    const sw = document.createElement("i");
    sw.className = shape;
    sw.style.background = s.color;
    const label = document.createElement("span");
    label.textContent = s.name;
    item.append(sw, label);
    container.append(item);
  }
}

/** Fills a <table> from rows of cells (first row = header). */
function table(el, rows) {
  const [head, ...body] = rows;
  const thead = document.createElement("thead");
  const tr = document.createElement("tr");
  for (const h of head) {
    const th = document.createElement("th");
    th.textContent = h;
    tr.append(th);
  }
  thead.append(tr);
  const tbody = document.createElement("tbody");
  for (const r of body) {
    const row = document.createElement("tr");
    for (const c of r) {
      const td = document.createElement("td");
      td.textContent = c;
      row.append(td);
    }
    tbody.append(row);
  }
  el.replaceChildren(thead, tbody);
}
