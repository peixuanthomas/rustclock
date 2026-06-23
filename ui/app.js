"use strict";

// ----------------------------- geometry ------------------------------------
const C = 500; // centre of the 1000x1000 viewBox
const R = 415; // face radius in viewBox units

const TAU = Math.PI * 2;
const ratioToAngle = (ratio) => ratio * TAU - Math.PI / 2;
function pt(radius, ratio) {
  const ang = ratioToAngle(ratio);
  return [C + Math.cos(ang) * radius, C + Math.sin(ang) * radius];
}

// ----------------------------- svg helpers ---------------------------------
const f = (n) => (Math.round(n * 100) / 100).toString();
function line(x1, y1, x2, y2, stroke, w, extra = "") {
  return `<line x1="${f(x1)}" y1="${f(y1)}" x2="${f(x2)}" y2="${f(y2)}" stroke="${stroke}" stroke-width="${f(w)}" stroke-linecap="round" ${extra}/>`;
}
function circ(cx, cy, r, fill, extra = "") {
  return `<circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r)}" fill="${fill}" ${extra}/>`;
}
function ring(cx, cy, r, stroke, w, extra = "") {
  return `<circle cx="${f(cx)}" cy="${f(cy)}" r="${f(r)}" fill="none" stroke="${stroke}" stroke-width="${f(w)}" ${extra}/>`;
}
function text(x, y, str, size, fill, extra = "") {
  return `<text x="${f(x)}" y="${f(y)}" font-size="${f(size)}" fill="${fill}" text-anchor="middle" dominant-baseline="central" font-family="Segoe UI, system-ui, sans-serif" ${extra}>${str}</text>`;
}
function arc(radius, startRatio, endRatio, stroke, w, extra = "") {
  const sweep = endRatio - startRatio;
  if (sweep <= 0) return "";
  const [x1, y1] = pt(radius, startRatio);
  const [x2, y2] = pt(radius, endRatio);
  const large = sweep > 0.5 ? 1 : 0;
  return `<path d="M ${f(x1)} ${f(y1)} A ${f(radius)} ${f(radius)} 0 ${large} 1 ${f(x2)} ${f(y2)}" fill="none" stroke="${stroke}" stroke-width="${f(w)}" stroke-linecap="round" ${extra}/>`;
}

// ----------------------------- colour utils --------------------------------
const rgb = (a) => `rgb(${a[0]},${a[1]},${a[2]})`;
const rgba = (a, al) => `rgba(${a[0]},${a[1]},${a[2]},${al})`;
function blend(a, b, t) {
  t = Math.max(0, Math.min(1, t));
  return rgb([0, 1, 2].map((i) => Math.round(a[i] + (b[i] - a[i]) * t)));
}

const ARABIC = ["12", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];
const ROMAN = ["XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
const WEEKDAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

// ----------------------------- shared pieces -------------------------------
function numerals(dial, color, scale) {
  if (dial === "none") return "";
  const arr = dial === "roman" ? ROMAN : ARABIC;
  let s = "";
  for (let i = 0; i < 12; i++) {
    const [x, y] = pt(R * 0.63, i / 12);
    const size = R * (dial === "roman" ? 0.11 : 0.12) * scale;
    s += text(x, y, arr[i], size, color, 'font-weight="500"');
  }
  return s;
}

function standardTicks(dim) {
  let s = "";
  for (let m = 0; m < 60; m++) {
    const r = m / 60;
    const [ox, oy] = pt(R * 0.94, r);
    let inner, w, col;
    if (m % 15 === 0) {
      inner = R * 0.74; w = 4; col = `rgba(255,255,255,${dim ? 0.36 : 0.86})`;
    } else if (m % 5 === 0) {
      inner = R * 0.79; w = 2.6; col = `rgba(255,255,255,${dim ? 0.3 : 0.67})`;
    } else {
      inner = R * 0.86; w = 1.2; col = `rgba(165,177,203,${dim ? 0.15 : 0.38})`;
    }
    const [ix, iy] = pt(inner, r);
    s += line(ix, iy, ox, oy, col, w);
  }
  return s;
}

function orbitTracks() {
  let s = "";
  for (const tr of [R * 0.47, R * 0.69, R * 0.87]) {
    s += ring(C, C, tr, "rgba(160,172,205,0.16)", 1);
  }
  return s;
}

function centerCap(color) {
  return (
    circ(C, C, R * 0.085, "rgba(200,220,250,0.12)") +
    circ(C, C, R * 0.05, color) +
    circ(C - R * 0.012, C - R * 0.014, R * 0.02, "rgba(255,255,255,0.55)")
  );
}

// ----------------------------- hands (svg) ---------------------------------
function hand(ratio, len, tail, w, color) {
  const [tx, ty] = pt(len, ratio);
  const [bx, by] = pt(tail, ratio + 0.5);
  return line(bx, by, tx, ty, color, w, 'filter="url(#handShadow)"');
}

function classicSecond(ratio) {
  const acc = "#ffbd5c";
  const [tx, ty] = pt(R * 0.88, ratio);
  const [bx, by] = pt(R * 0.18, ratio + 0.5);
  let s = line(bx, by, tx, ty, acc, 2.4);
  s += circ(tx, ty, R * 0.03, "rgba(255,189,92,0.5)", 'filter="url(#glow2)"');
  const [cx, cy] = pt(R * 0.18, ratio + 0.5);
  s += circ(cx, cy, R * 0.02, acc);
  return s;
}

// ----------------------------- face styles ---------------------------------
function faceClassic(dial, hr, mr, sr, showSecond) {
  let s = standardTicks(false);
  s += numerals(dial, "#ebf0f8", 1.0);
  s += hand(hr, R * 0.55, R * 0.08, 7, "#f0f3fa");
  s += hand(mr, R * 0.78, R * 0.1, 4.2, "#cdd6e7");
  if (showSecond) s += classicSecond(sr);
  s += centerCap("#f5f8fc");
  return s;
}

function faceFrosted(dial) {
  // Hands are drawn as frosted-glass DOM elements; SVG only paints the dial.
  let s = circ(C, C, R * 0.9, "rgba(150,180,230,0.04)");
  s += standardTicks(true);
  s += numerals(dial, "rgba(225,235,250,0.6)", 0.86);
  return s;
}

function luminousRing(ringR, ratio, divisions, base, glow, baseW, span) {
  let s = "";
  for (let k = 0; k < divisions; k++) {
    const r = k / divisions;
    const [ix, iy] = pt(ringR - R * 0.04, r);
    const [ox, oy] = pt(ringR + R * 0.04, r);
    let d = (((ratio - r) % 1) + 1) % 1;
    const g = d <= span ? (1 - d / span) ** 2 : 0;
    const w = (divisions === 12 ? 3 : k % 5 === 0 ? 2.2 : 1.2) + g * 2.2;
    s += line(ix, iy, ox, oy, blend(base, glow, g), w);
  }
  s += arc(ringR, ratio - span * 0.85, ratio, rgba(glow, 0.5), baseW + 2, 'filter="url(#glow4)"');
  const [hx, hy] = pt(ringR, ratio);
  s += circ(hx, hy, baseW * 2.4, rgba(glow, 0.5), 'filter="url(#glow8)"');
  s += circ(hx, hy, baseW * 1.1, rgb(glow));
  s += circ(hx - baseW * 0.3, hy - baseW * 0.3, baseW * 0.4, "rgba(255,255,255,0.6)");
  return s;
}

function faceLuminous(dial, hr, mr, sr, showSecond) {
  let s = circ(C, C, R * 0.96, "rgba(56,94,160,0.05)");
  s += luminousRing(R * 0.54, hr, 12, [70, 110, 200], [146, 198, 255], 2.8, 0.16);
  s += luminousRing(R * 0.71, mr, 60, [60, 150, 150], [108, 244, 228], 2.2, 0.13);
  if (showSecond)
    s += luminousRing(R * 0.88, sr, 60, [180, 90, 80], [255, 138, 112], 1.9, 0.09);
  s += numerals(dial, "rgba(214,229,255,0.46)", 0.82);
  s += circ(C, C, R * 0.11, "rgba(132,190,255,0.18)", 'filter="url(#glow4)"');
  s += circ(C, C, R * 0.062, "#e5f1ff");
  s += circ(C, C, R * 0.034, "#6addff");
  return s;
}

function triLane(trackR, ratio, color, size) {
  let s = ring(C, C, trackR, rgba(color, 0.12), 1.4);
  const [px, py] = pt(trackR, ratio);
  s += circ(px, py, size * 1.7, rgba(color, 0.18), 'filter="url(#glow4)"');
  const [tx, ty] = pt(trackR + size * 0.62, ratio);
  const [bx, by] = pt(trackR - size * 0.48, ratio);
  const ang = ratioToAngle(ratio);
  const tnx = -Math.sin(ang) * size * 0.56;
  const tny = Math.cos(ang) * size * 0.56;
  s += `<polygon points="${f(tx)},${f(ty)} ${f(bx + tnx)},${f(by + tny)} ${f(bx - tnx)},${f(by - tny)}" fill="${rgb(color)}"/>`;
  s += circ(tx, ty, size * 0.22, "rgba(255,255,255,0.6)");
  return s;
}

function faceTriangle(dial, hr, mr, sr, showSecond) {
  let s = orbitTracks();
  s += numerals(dial, "rgba(235,240,248,0.6)", 0.88);
  s += triLane(R * 0.48, hr, [231, 238, 252], 16);
  s += triLane(R * 0.7, mr, [112, 237, 228], 13);
  if (showSecond) s += triLane(R * 0.89, sr, [255, 138, 112], 10);
  s += circ(C, C, R * 0.034, "#f5f8fc");
  return s;
}

function orbitDot(trackR, ratio, size, color) {
  let s = "";
  for (let i = 1; i <= 6; i++) {
    const [x, y] = pt(trackR, ratio - i * 0.006);
    s += circ(x, y, size * (1 - i * 0.1), rgba(color, Math.max(0, 0.18 - i * 0.025)));
  }
  const [px, py] = pt(trackR, ratio);
  s += circ(px, py, size * 2.4, rgba(color, 0.18), 'filter="url(#glow8)"');
  s += circ(px, py, size, rgb(color));
  s += circ(px - size * 0.2, py - size * 0.22, size * 0.42, "rgba(255,255,255,0.5)");
  return s;
}

function faceOrbit(dial, hr, mr, sr, showSecond) {
  let s = orbitTracks();
  s += numerals(dial, "rgba(235,240,248,0.5)", 0.82);
  s += orbitDot(R * 0.47, hr, 9, [168, 172, 181]);
  s += orbitDot(R * 0.69, mr, 7, [212, 156, 108]);
  if (showSecond) {
    s += orbitDot(R * 0.87, sr, 5.8, [82, 154, 232]);
    const [ex, ey] = pt(R * 0.87, sr);
    s += circ(ex + 2, ey - 1.5, 1.7, "#6ad07a");
  }
  // sun centre
  s += circ(C, C, R * 0.1, "rgba(255,192,80,0.18)", 'filter="url(#glow8)"');
  s += circ(C, C, R * 0.042, "#ffca60");
  s += circ(C, C, R * 0.025, "#ffefb0");
  return s;
}

function arcTracks(showSecond) {
  let s = "";
  for (const tr of [R * 0.48, R * 0.68, R * 0.88]) {
    if (!showSecond && tr > R * 0.8) continue;
    s += ring(C, C, tr, "rgba(155,168,198,0.14)", 1);
  }
  return s;
}

function band(radius, ratio, color, w) {
  if (ratio <= 0) return "";
  let s = arc(radius, 0, ratio, rgba(color, 0.45), w + 2.5, 'filter="url(#glow4)"');
  s += arc(radius, 0, ratio, rgb(color), w);
  const [hx, hy] = pt(radius, ratio);
  s += circ(hx, hy, w * 1.8, rgba(color, 0.5), 'filter="url(#glow4)"');
  s += circ(hx, hy, w * 0.9, rgb(color));
  return s;
}

function faceArc(dial, hr, mr, sr, showSecond) {
  let s = arcTracks(showSecond);
  s += numerals(dial, "rgba(235,240,248,0.32)", 0.7);
  s += band(R * 0.46, hr, [240, 244, 252], 2.8);
  s += band(R * 0.66, mr, [122, 236, 226], 2.2);
  if (showSecond) s += band(R * 0.86, sr, [255, 136, 112], 1.5);
  s += circ(C, C, R * 0.1, "rgba(132,150,196,0.14)", 'filter="url(#glow4)"');
  s += circ(C, C, R * 0.045, "#f0f4fc");
  s += circ(C, C, R * 0.022, "#ffffff");
  return s;
}

// ----------------------------- static face ---------------------------------
function buildFace() {
  let s = "";
  s += circ(C, C, R * 1.06, "rgba(110,150,235,0.05)", 'filter="url(#glow16)"');
  s += circ(C, C, R, "url(#face-grad)");
  // glass highlight, upper-left
  s += `<ellipse cx="${f(C - R * 0.28)}" cy="${f(C - R * 0.34)}" rx="${f(R * 0.42)}" ry="${f(R * 0.3)}" fill="rgba(200,225,255,0.05)" filter="url(#glow8)"/>`;
  s += ring(C, C, R * 1.01, "rgba(100,140,220,0.18)", 3);
  s += ring(C, C, R, "rgba(255,255,255,0.28)", 1.8);
  s += ring(C, C, R * 0.92, "rgba(120,146,220,0.16)", 0.7);
  document.getElementById("face").innerHTML = s;
}

// ----------------------------- state ---------------------------------------
const state = {
  face: "classic",
  dial: "arabic",
  smooth: true,
  showSecond: true,
  fullscreen: true,
};

let hourRatio = 0, minuteRatio = 0, secondRatio = 0;

// countdown timers
let countdowns = []; // {id, total(sec), start(ms perf), finishedAt(ms|null)}
let nextId = 1;
let selectedId = null;

function selectedCountdown() {
  return countdowns.find((c) => c.id === selectedId) || null;
}

function refreshCountdowns(nowPerf) {
  for (const c of countdowns) {
    if (c.finishedAt === null && (nowPerf - c.start) / 1000 >= c.total) {
      c.finishedAt = nowPerf;
    }
  }
  if (selectedId !== null && !countdowns.some((c) => c.id === selectedId)) {
    selectedId = countdowns.length ? countdowns[0].id : null;
  } else if (selectedId === null && countdowns.length) {
    selectedId = countdowns[0].id;
  }
}

function startCountdown() {
  const h = parseInt(document.getElementById("cd-h").value || "0", 10) || 0;
  const m = parseInt(document.getElementById("cd-m").value || "0", 10) || 0;
  const s = parseInt(document.getElementById("cd-s").value || "0", 10) || 0;
  const total = h * 3600 + m * 60 + s;
  if (total <= 0) return;
  const id = nextId++;
  countdowns.push({ id, total, start: performance.now(), finishedAt: null });
  selectedId = id;
  document.getElementById("cd-h").value = "";
  document.getElementById("cd-m").value = "";
  document.getElementById("cd-s").value = "";
}

function deleteCountdown(id) {
  countdowns = countdowns.filter((c) => c.id !== id);
  if (selectedId === id) selectedId = countdowns.length ? countdowns[0].id : null;
}

function fmtHMS(totalSeconds) {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  const p = (n) => String(n).padStart(2, "0");
  return `${p(h)}:${p(m)}:${p(s)}`;
}

function countdownArc(nowPerf) {
  const cd = selectedCountdown();
  if (!cd) return "";
  const remaining = Math.max(0, cd.total - (nowPerf - cd.start) / 1000);
  if (remaining <= 0) return "";
  let start, radius, w, cycle;
  if (remaining < 60) {
    start = secondRatio; radius = R * 0.9; w = 4; cycle = 60;
  } else if (remaining < 3600) {
    start = minuteRatio; radius = R * 0.78; w = 6; cycle = 3600;
  } else {
    start = hourRatio; radius = R * 0.56; w = 8; cycle = 12 * 3600;
  }
  const sweep = Math.min(0.999, remaining / cycle);
  if (sweep <= 0) return "";
  let s = arc(radius, start, start + sweep, "rgba(255,84,84,0.3)", w + 3, 'filter="url(#glow4)"');
  s += arc(radius, start, start + sweep, "#ff5454", w);
  return s;
}

// ----------------------------- countdown list ------------------------------
const cdListEl = document.getElementById("cd-list");
const cdEmptyEl = document.getElementById("cd-empty");

// Cards are built once per change to the timer set and updated in place every
// frame. Rebuilding innerHTML on each frame would destroy the node a click
// started on before mouseup, so click events (select / delete) never fired.
let renderedSig = "";

function buildCdCards() {
  let html = "";
  for (const c of countdowns) {
    html += `<div class="cd-card" data-id="${c.id}">
      <div class="cd-time"></div>
      <div class="cd-note"></div>
      <button class="cd-del" data-id="${c.id}">Delete</button>
    </div>`;
  }
  cdListEl.innerHTML = html;
}

function renderCdList(nowPerf) {
  if (!countdowns.length) {
    cdEmptyEl.classList.remove("hidden");
    if (cdListEl.childElementCount) cdListEl.innerHTML = "";
    renderedSig = "";
    return;
  }
  cdEmptyEl.classList.add("hidden");

  // Rebuild the DOM only when the set of timers changes.
  const sig = countdowns.map((c) => c.id).join(",");
  if (sig !== renderedSig) {
    buildCdCards();
    renderedSig = sig;
  }

  const cards = cdListEl.children;
  for (let i = 0; i < countdowns.length; i++) {
    const c = countdowns[i];
    const card = cards[i];
    if (!card) continue;
    const remainingMs = c.total * 1000 - (nowPerf - c.start);
    const display = remainingMs <= 0 ? 0 : Math.ceil(remainingMs / 1000);
    const finished = c.finishedAt !== null;
    const flash = finished && Math.floor((nowPerf - c.finishedAt) / 350) % 2 === 0;
    const sel = c.id === selectedId;
    const note = finished
      ? "Finished"
      : sel
      ? "Shown on analog face"
      : "Click to show on face";

    card.classList.toggle("selected", sel);
    card.classList.toggle("finished", finished);
    card.classList.toggle("flash", flash);
    card.querySelector(".cd-time").textContent = fmtHMS(display);
    card.querySelector(".cd-note").textContent = note;
  }
}

cdListEl.addEventListener("click", (e) => {
  const del = e.target.closest(".cd-del");
  if (del) {
    deleteCountdown(parseInt(del.dataset.id, 10));
    return;
  }
  const card = e.target.closest(".cd-card");
  if (card) selectedId = parseInt(card.dataset.id, 10);
});

// ----------------------------- frosted hands -------------------------------
const handsEl = document.getElementById("hands");
const fhHour = document.getElementById("fh-hour");
const fhMin = document.getElementById("fh-min");
const fhSec = document.getElementById("fh-sec");

function updateFrostedHands() {
  fhHour.style.transform = `translateX(-50%) rotate(${hourRatio * 360}deg)`;
  fhMin.style.transform = `translateX(-50%) rotate(${minuteRatio * 360}deg)`;
  fhSec.style.transform = `translateX(-50%) rotate(${secondRatio * 360}deg)`;
  fhSec.style.display = state.showSecond ? "" : "none";
}

// ----------------------------- main loop -----------------------------------
const dyn = document.getElementById("dyn");
const elTime = document.getElementById("t-time");
const elSec = document.getElementById("t-sec");
const elDate = document.getElementById("t-date");
const elWeek = document.getElementById("t-weekday");

function render() {
  const nowPerf = performance.now();
  const d = new Date();
  const ms = d.getMilliseconds();
  const sec = d.getSeconds();
  const min = d.getMinutes();
  const hr = d.getHours();

  const pSec = sec + ms / 1000;
  const pMin = min + pSec / 60;
  const pHour = (hr % 12) + pMin / 60;

  secondRatio = state.smooth ? pSec / 60 : sec / 60;
  minuteRatio = state.smooth ? pMin / 60 : min / 60;
  hourRatio = state.smooth ? pHour / 12 : (hr % 12) / 12 + min / 720;

  refreshCountdowns(nowPerf);

  // info panel text
  const p2 = (n) => String(n).padStart(2, "0");
  elTime.textContent = `${p2(hr)}:${p2(min)}`;
  elSec.textContent = p2(sec);
  elSec.classList.toggle("off", !state.showSecond);
  elDate.textContent = `${d.getFullYear()}-${p2(d.getMonth() + 1)}-${p2(d.getDate())}`;
  elWeek.textContent = WEEKDAYS[d.getDay()];

  // analog face
  let parts = countdownArc(nowPerf);
  switch (state.face) {
    case "frosted":
      parts += faceFrosted(state.dial);
      break;
    case "luminous":
      parts += faceLuminous(state.dial, hourRatio, minuteRatio, secondRatio, state.showSecond);
      break;
    case "triangle":
      parts += faceTriangle(state.dial, hourRatio, minuteRatio, secondRatio, state.showSecond);
      break;
    case "orbit":
      parts += faceOrbit(state.dial, hourRatio, minuteRatio, secondRatio, state.showSecond);
      break;
    case "arc":
      parts += faceArc(state.dial, hourRatio, minuteRatio, secondRatio, state.showSecond);
      break;
    default:
      parts += faceClassic(state.dial, hourRatio, minuteRatio, secondRatio, state.showSecond);
  }
  dyn.innerHTML = parts;

  if (state.face === "frosted") {
    handsEl.hidden = false;
    updateFrostedHands();
  } else {
    handsEl.hidden = true;
  }

  renderCdList(nowPerf);
  requestAnimationFrame(render);
}

// ----------------------------- fullscreen / close --------------------------
function tauriWindow() {
  try {
    return window.__TAURI__?.window?.getCurrentWindow?.() ?? null;
  } catch (_) {
    return null;
  }
}

async function setFullscreen(v) {
  state.fullscreen = v;
  const w = tauriWindow();
  if (w) {
    try {
      await w.setFullscreen(v);
      await w.setDecorations(!v);
      return;
    } catch (_) {
      /* fall through to browser */
    }
  }
  if (v) document.documentElement.requestFullscreen?.().catch(() => {});
  else document.exitFullscreen?.().catch(() => {});
}

async function closeApp() {
  const w = tauriWindow();
  if (w) {
    try {
      await w.close();
      return;
    } catch (_) {
      /* fall through to browser */
    }
  }
  window.close();
}

const closeModal = document.getElementById("close-modal");

function requestClose() {
  // Confirm only when a countdown is still counting down.
  const running = countdowns.some((c) => c.finishedAt === null);
  if (running) {
    closeModal.classList.remove("hidden");
  } else {
    closeApp();
  }
}

// ----------------------------- wiring --------------------------------------
function digitsOnly(el, maxLen) {
  el.addEventListener("input", () => {
    let v = el.value.replace(/\D/g, "");
    if (v.length > maxLen) v = v.slice(0, maxLen);
    el.value = v;
  });
}

function init() {
  buildFace();

  digitsOnly(document.getElementById("cd-h"), 3);
  digitsOnly(document.getElementById("cd-m"), 2);
  digitsOnly(document.getElementById("cd-s"), 2);

  document.getElementById("cd-start").addEventListener("click", startCountdown);

  // Press Enter in any countdown input to start.
  for (const id of ["cd-h", "cd-m", "cd-s"]) {
    document.getElementById(id).addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        startCountdown();
      }
    });
  }

  document.getElementById("sel-face").addEventListener("change", (e) => {
    state.face = e.target.value;
  });
  document.getElementById("sel-dial").addEventListener("change", (e) => {
    state.dial = e.target.value;
  });
  document.getElementById("chk-smooth").addEventListener("change", (e) => {
    state.smooth = e.target.checked;
  });
  document.getElementById("chk-second").addEventListener("change", (e) => {
    state.showSecond = e.target.checked;
  });
  document.getElementById("btn-close").addEventListener("click", requestClose);
  document.getElementById("close-cancel").addEventListener("click", () => {
    closeModal.classList.add("hidden");
  });
  document.getElementById("close-confirm").addEventListener("click", closeApp);
  closeModal.addEventListener("click", (e) => {
    if (e.target === closeModal) closeModal.classList.add("hidden");
  });

  window.addEventListener("keydown", (e) => {
    if (e.key === "F11") {
      e.preventDefault();
      setFullscreen(!state.fullscreen);
    } else if (e.key === "Escape") {
      if (!closeModal.classList.contains("hidden")) {
        closeModal.classList.add("hidden");
      } else if (state.fullscreen) {
        setFullscreen(false);
      }
    }
  });

  requestAnimationFrame(render);
}

init();
