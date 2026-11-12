// OpenInterceptor control console (protocol v1; see the docs site, Specification › Console
// protocol). Plain custom elements: each element owns one part of the page and reacts to
// the shared `link` (the WebSocket) through DOM events. No framework, no build step.
"use strict";

/** The connection to the game, shared by every element. */
const link = new EventTarget();
let socket = null;

function connect(code) {
  if (socket && socket.readyState <= WebSocket.OPEN) socket.close(); // one connection at a time
  socket = new WebSocket(`ws://${location.host}/ws`);
  socket.onopen = () => {
    socket.send(JSON.stringify({ type: "hello", code }));
    // Heartbeat: the game releases this console's held buttons if it goes quiet for a
    // second (Wi-Fi dropped, tablet asleep), so nothing stays pressed by accident.
    const beat = setInterval(() => send({ type: "ping" }), 250);
    socket.addEventListener("close", () => clearInterval(beat));
  };
  socket.onmessage = (e) => {
    const msg = JSON.parse(e.data);
    link.dispatchEvent(new CustomEvent(msg.type, { detail: msg }));
  };
  socket.onerror = () => link.dispatchEvent(new CustomEvent("failed"));
  socket.onclose = () => link.dispatchEvent(new CustomEvent("closed"));
}

function send(msg) {
  if (socket && socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify(msg));
}

/** Keeps the tablet's screen awake while the console is open.
 *
 *  1. The Screen Wake Lock API, where the browser offers it. Safari offers it only on
 *     secure (https) pages, and the console is plain http on the local network, so on an
 *     iPad this usually does not apply.
 *  2. Otherwise a tiny black clip, muted, inline and looping, out of sight: browsers do
 *     not let the screen sleep while a video plays. iOS allows muted inline playback, but
 *     only reliably after a tap, so this starts on the first touch (the pairing tap).
 *
 *  Both are released by the browser when the page is hidden, so they are taken again
 *  whenever it becomes visible. `keepAwakeState` is shown on the page. */
let keepAwakeState = "screen may sleep";
let wakeLock = null;
let wakeVideo = null;

async function keepAwake() {
  if (document.visibilityState !== "visible") return;
  if ("wakeLock" in navigator) {
    try {
      wakeLock = await navigator.wakeLock.request("screen");
      keepAwakeState = "screen kept awake (wake lock)";
      link.dispatchEvent(new CustomEvent("awake"));
      return;
    } catch (_) { /* not allowed here: fall through to the video */ }
  }
  if (!wakeVideo) {
    wakeVideo = document.createElement("video");
    wakeVideo.src = "keepawake.mp4";
    wakeVideo.muted = true;
    wakeVideo.loop = true;
    wakeVideo.setAttribute("playsinline", "");
    wakeVideo.setAttribute("muted", "");
    // Shown, small, as a "link" indicator: iOS pauses muted videos that are not visible.
    wakeVideo.className = "awake";
    wakeVideo.title = "Keeps this screen awake";
    document.body.appendChild(wakeVideo);
  }
  try {
    await wakeVideo.play();
    keepAwakeState = "screen kept awake";
  } catch (_) {
    keepAwakeState = "tap the screen to keep it awake";
  }
  link.dispatchEvent(new CustomEvent("awake"));
}

document.addEventListener("visibilitychange", keepAwake);
// Media playback needs a user gesture on iOS: try on every tap until it works.
document.addEventListener("pointerdown", () => {
  if (!keepAwakeState.startsWith("screen kept")) keepAwake();
}, { passive: true });

/** Pairing: the 6-digit code shown in the game. Accepted as soon as the sixth digit is
 *  typed (iPad number pads have no Enter key); remembered for this browser tab. */
class OiPairing extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<form><h1>OpenInterceptor console</h1>
      <p>Enter the pairing code shown in the game.</p>
      <input inputmode="numeric" pattern="[0-9]*" maxlength="6" autocomplete="off" aria-label="Pairing code">
      <p class="status"></p></form>`;
    const input = this.querySelector("input");
    const status = this.querySelector(".status");
    const show = (text, bad) => { status.textContent = text; status.classList.toggle("error", !!bad); };
    const pair = () => {
      const code = input.value.replace(/\D/g, "");
      if (code.length !== 6) { show("The code has six digits.", true); return; }
      try { sessionStorage.setItem("oi-code", code); } catch (_) { /* private mode */ }
      show("Connecting…");
      keepAwake();
      connect(code);
    };
    this.querySelector("form").addEventListener("submit", (e) => { e.preventDefault(); pair(); });
    input.addEventListener("input", () => {
      input.value = input.value.replace(/\D/g, "").slice(0, 6);
      if (input.value.length === 6) pair();
    });
    link.addEventListener("denied", (e) => { show(e.detail.reason, true); input.value = ""; input.focus(); });
    link.addEventListener("welcome", () => { this.hidden = true; document.getElementById("console").hidden = false; });
    link.addEventListener("closed", () => {
      this.hidden = false; document.getElementById("console").hidden = true;
      show("Disconnected. Enter the code to reconnect.", true);
    });
    link.addEventListener("failed", () => show("Could not reach the game. Is it still running?", true));
    // A code in the address (…/?code=123456, e.g. from a QR code) pairs straight away.
    let saved = new URLSearchParams(location.search).get("code");
    try { saved = saved || sessionStorage.getItem("oi-code"); } catch (_) { /* private mode */ }
    if (saved) { input.value = saved; pair(); } else { input.focus(); }
  }
}

/** The craft's condition, updated by every telemetry message. */
class OiTelemetry extends HTMLElement {
  connectedCallback() {
    const cells = [["craft", "Craft"], ["speed", "Speed m/s"], ["drift", "Drift X / Y"], ["hull", "Hull"],
      ["energy", "Energy"], ["shields", "Shields fore / aft"], ["missile", "Missile"], ["target", "Target"], ["awake", "Tablet"]];
    this.innerHTML = cells.map(([k, l]) =>
      `<div class="cell"><div class="label">${l}</div><div class="value" data-k="${k}">—</div></div>`).join("");
    const set = (k, v) => { this.querySelector(`[data-k="${k}"]`).textContent = v; };
    const pct = (f) => `${Math.round(f * 100)}%`;
    const awake = () => set("awake", keepAwakeState);
    link.addEventListener("awake", awake);
    awake();
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive) { set("craft", t.state === "launcher" ? "Not flying (launcher)" : "DESTROYED"); return; }
      const [x, y, z] = t.velocity;
      set("craft", t.craft);
      set("speed", Math.round(z));
      set("drift", `${x >= 0 ? "+" : ""}${Math.round(x)} / ${y >= 0 ? "+" : ""}${Math.round(y)}`);
      set("hull", pct(t.hull));
      set("energy", pct(t.energy));
      set("shields", `${pct(t.shields[0])} / ${pct(t.shields[1])}`);
      set("missile", t.missile ? `${t.missile.name} ×${t.missile.count}` : "none");
      set("target", t.target ? `${t.target.name} ${(t.target.distance / 1000).toFixed(1)} km` : "none");
    });
  }
}

/** A throttle slider (absolute position, 0–100 %). */
class OiThrottle extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<label>Throttle <input type="range" min="0" max="100" value="0"></label>
      <span class="value">0%</span>`;
    const input = this.querySelector("input");
    const out = this.querySelector(".value");
    let dragging = false;
    input.addEventListener("pointerdown", () => { dragging = true; });
    input.addEventListener("pointerup", () => { dragging = false; });
    input.addEventListener("input", () => {
      out.textContent = `${input.value}%`;
      send({ type: "throttle", value: input.value / 100 });
    });
    this.onTelemetry = (e) => {
      if (!dragging && e.detail.alive) {
        input.value = Math.round(e.detail.throttle * 100);
        out.textContent = `${input.value}%`;
      }
    };
    link.addEventListener("telemetry", this.onTelemetry);
  }

  // A throttle inside a tile strip is removed when the strip closes: stop listening.
  disconnectedCallback() {
    link.removeEventListener("telemetry", this.onTelemetry);
  }
}

/** The campaign's theme (R-CON-020): CSS variables set on the page. Variables a theme
 *  leaves out return to the page's own defaults (console.css). */
let themed = [];
link.addEventListener("theme", (e) => {
  const root = document.documentElement.style;
  for (const k of themed) root.removeProperty(k);
  themed = Object.keys(e.detail.vars || {});
  for (const k of themed) root.setProperty(k, e.detail.vars[k]);
});

/** The latest catalogue and telemetry, for elements built after they arrive. */
let catalogue = null;
let latest = null;
link.addEventListener("catalogue", (e) => { catalogue = e.detail; });
link.addEventListener("telemetry", (e) => { latest = e.detail; });

/** One action as a button. Held actions send down/up; taps send once. Every button
 *  lights up while pressed (class "down"): a hold for as long as it is held, a tap for
 *  at least MIN_LIT ms so a quick tap is still seen. A feature this craft has not got
 *  is shown, disabled, as "not fitted" (never hidden). The action's one-sentence
 *  description is a small line under the label (a tablet has no hover, so a tooltip
 *  alone would never be seen) and the button's title, for a mouse and screen readers. */
function makeButton(a) {
  const b = document.createElement("button");
  b.textContent = a.label;
  if (a.description) {
    // textContent, never innerHTML: the text is shown as text, whatever it contains.
    const d = document.createElement("small");
    d.className = "desc";
    d.textContent = a.description;
    b.appendChild(d);
    b.title = a.description;
  }
  b.className = a.kind;
  b.disabled = !a.available;
  let since = 0;
  let lit = null;
  const press = (ev) => {
    ev.preventDefault();
    clearTimeout(lit);
    since = performance.now();
    b.classList.add("down");
    send({ type: "action", name: a.name, value: 1 });
  };
  const release = () => {
    if (!b.classList.contains("down")) return;
    if (a.kind === "hold") send({ type: "action", name: a.name, value: 0 });
    const left = Math.max(0, MIN_LIT - (performance.now() - since));
    lit = setTimeout(() => b.classList.remove("down"), left);
  };
  b.addEventListener("pointerdown", press);
  for (const t of ["pointerup", "pointercancel", "pointerleave"]) b.addEventListener(t, release);
  return b;
}

/** All-in-one mode: every action group expanded as a grid of buttons. */
class OiActions extends HTMLElement {
  connectedCallback() {
    link.addEventListener("catalogue", (e) => this.build(e.detail));
    if (catalogue) this.build(catalogue);
  }

  build(cat) {
    this.innerHTML = "";
    for (const g of cat.groups) {
      const section = document.createElement("section");
      section.innerHTML = `<h2>${g.title}</h2><div class="buttons"></div>`;
      const box = section.querySelector(".buttons");
      for (const a of cat.actions.filter((a) => a.group === g.id)) box.appendChild(makeButton(a));
      this.appendChild(section);
    }
  }
}

/** A brief burst of static over the console when the player's craft is hit: lighter
 *  when the shields take it, stronger when the hull does (telemetry `hits` and
 *  `hull_hits` are running counts; a rise means new hits). Drawn at low resolution and
 *  scaled up, so it costs little; fades out over STATIC_MS. */
class OiStatic extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<canvas width="160" height="100"></canvas>`;
    const canvas = this.querySelector("canvas");
    const g = canvas.getContext("2d");
    const img = g.createImageData(canvas.width, canvas.height);
    let last = null; // [hits, hull_hits] seen
    let until = 0;
    let strength = 0;
    let frame = 0;
    const draw = (now) => {
      const left = until - now;
      if (left <= 0) { this.style.opacity = 0; frame = 0; return; }
      // Grey noise with faint scan lines; the overlay's opacity fades it out.
      for (let i = 0, y = 0; y < canvas.height; y++) {
        const band = y % 3 === 0 ? 0.6 : 1;
        for (let x = 0; x < canvas.width; x++, i += 4) {
          const v = Math.random() * 255 * band;
          img.data[i] = img.data[i + 1] = img.data[i + 2] = v;
          img.data[i + 3] = 255;
        }
      }
      g.putImageData(img, 0, 0);
      this.style.opacity = strength * (left / STATIC_MS);
      frame = requestAnimationFrame(draw);
    };
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive || t.hits === undefined) { last = null; return; }
      const now = [t.hits, t.hull_hits];
      if (last && (now[0] > last[0] || now[1] > last[1])) {
        const hull = now[1] > last[1];
        strength = Math.max(hull ? 0.55 : 0.25, until > performance.now() ? strength : 0);
        until = performance.now() + STATIC_MS;
        if (!frame) frame = requestAnimationFrame(draw);
      }
      last = now;
    });
  }
}

/* ---------------------------------------------------------------------------------
 * Cockpit mode (the layout of the design proposal): HUD strip on top, shields left,
 * the display console centre, power right, function-group tiles below.
 * ------------------------------------------------------------------------------- */

const pct = (f) => `${Math.round((f || 0) * 100)}%`;
const signed = (n) => `${n >= 0 ? "+" : ""}${Math.round(n)}`;
const LEVELS = "ABCDEFGHIJ";

/** Chooses the layout: "allinone" or "cockpit", remembered on this tablet. */
class OiMode extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<button data-m="allinone">All-in-one</button><button data-m="cockpit">Cockpit</button>`;
    let mode = "allinone";
    try { mode = localStorage.getItem("oi-mode") || mode; } catch (_) { /* private mode */ }
    mode = new URLSearchParams(location.search).get("mode") || mode; // e.g. …/?mode=cockpit
    const set = (m) => {
      document.body.dataset.mode = m;
      for (const b of this.querySelectorAll("button")) b.classList.toggle("on", b.dataset.m === m);
      try { localStorage.setItem("oi-mode", m); } catch (_) { /* private mode */ }
    };
    for (const b of this.querySelectorAll("button")) b.addEventListener("click", () => set(b.dataset.m));
    set(mode);
  }
}

/** The exterior-view HUD as a strip: speed, drift, throttle, hull, energy, target, flags. */
class OiHudStrip extends HTMLElement {
  connectedCallback() {
    const cells = [["spd", "SPD"], ["drift", "DRIFT X/Y"], ["thr", "THR"], ["hull", "HULL"], ["nrg", "ENERGY"],
      ["tgt", "TARGET"], ["flags", "STATUS"]];
    this.innerHTML = cells.map(([k, l]) => `<div><span>${l}</span><b data-k="${k}">—</b></div>`).join("");
    const set = (k, v) => { this.querySelector(`[data-k="${k}"]`).textContent = v; };
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive) { set("spd", t.state === "launcher" ? "not flying" : "DESTROYED"); return; }
      const [x, y, z] = t.velocity;
      set("spd", `${Math.round(z)} m/s`);
      set("drift", `${signed(x)} / ${signed(y)}`);
      set("thr", pct(t.throttle));
      set("hull", pct(t.hull));
      set("nrg", pct(t.energy));
      set("tgt", t.target ? `${t.target.name} ${(t.target.distance / 1000).toFixed(1)} km` : "none");
      set("flags", [t.link.toUpperCase(), t.afterburner && "BURN", t.braking && "BRAKE"].filter(Boolean).join(" "));
    });
  }
}

/** Fore and aft shield rings (green, amber, red, blinking when down) and the balance. */
class OiShields extends HTMLElement {
  connectedCallback() {
    const ring = (id, label) => `<figure><svg viewBox="0 0 100 100"><circle class="track" cx="50" cy="50" r="40"/>
      <circle class="level" data-r="${id}" cx="50" cy="50" r="40" transform="rotate(-90 50 50)"/>
      <text x="50" y="47">${label}</text><text class="pc" data-p="${id}" x="50" y="64">—</text></svg></figure>`;
    this.innerHTML = `<h2>Shields</h2>${ring("fore", "FORE")}${ring("aft", "AFT")}
      <div class="seg"><button data-b="forward">Fore</button><button data-b="equal">Equal</button><button data-b="aft">Aft</button></div>`;
    const length = 2 * Math.PI * 40;
    for (const c of this.querySelectorAll(".level")) c.style.strokeDasharray = `0 ${length}`;
    for (const b of this.querySelectorAll("[data-b]")) {
      b.addEventListener("click", () => send({ type: "power", balance: b.dataset.b }));
    }
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive) return;
      for (const [k, f] of [["fore", t.shields[0]], ["aft", t.shields[1]]]) {
        const c = this.querySelector(`[data-r="${k}"]`);
        c.style.strokeDasharray = `${length * f} ${length}`;
        c.dataset.level = f > 0.66 ? "ok" : f > 0.33 ? "low" : f > 0.01 ? "bad" : "down";
        this.querySelector(`[data-p="${k}"]`).textContent = pct(f);
      }
      for (const b of this.querySelectorAll("[data-b]")) b.classList.toggle("on", b.dataset.b === t.balance);
    });
  }
}

/** The display console: the target's data, with tabs for the target view (C3) and the
 *  map (C4), which arrive in later steps. */
class OiDisplay extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<nav><button data-tab="target" class="on">Target</button><button data-tab="map">Map</button></nav>
      <div data-pane="target"><div class="tdata"></div><oi-target-view class="viewport"></oi-target-view></div>
      <div data-pane="map" hidden><oi-map></oi-map></div>`;
    for (const b of this.querySelectorAll("[data-tab]")) {
      b.addEventListener("click", () => {
        for (const x of this.querySelectorAll("[data-tab]")) x.classList.toggle("on", x === b);
        for (const p of this.querySelectorAll("[data-pane]")) p.hidden = p.dataset.pane !== b.dataset.tab;
      });
    }
    const box = this.querySelector(".tdata");
    const bar = (label, f) => `<div class="bar"><span>${label}</span><i style="width:${pct(f)}"></i><em>${pct(f)}</em></div>`;
    link.addEventListener("telemetry", (e) => {
      const t = e.detail.target;
      if (!e.detail.alive || !t) { box.innerHTML = `<p class="none">No target</p>`; return; }
      box.innerHTML = `<h3 class="${t.relation}">${t.name}${t.protect ? ' <small>PROTECT</small>' : ""}</h3>
        <p>${(t.distance / 1000).toFixed(2)} km · closing ${Math.round(t.closing)} m/s</p>
        ${bar("Hull", t.hull)}${bar("Fore", t.shields[0])}${bar("Aft", t.shields[1])}`;
    });
    // The last few lines of the pilot's message log, under the target data.
    const log = document.createElement("ol");
    log.className = "log";
    this.querySelector('[data-pane="target"]').appendChild(log);
    link.addEventListener("event", (e) => {
      const li = document.createElement("li");
      li.textContent = e.detail.text;
      log.appendChild(li);
      while (log.children.length > 4) log.firstChild.remove();
    });
  }
}

/** The target viewport (C3): the target's model, flat-shaded with WebGL, turned to its
 *  orientation as seen from the player's craft (telemetry `target.rotation`, a
 *  quaternion in the player's body axes). The model arrives once per ship class in a
 *  `model` message, normalised to radius 1. Plain WebGL 1, no library. */
class OiTargetView extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<canvas></canvas><p class="label">No target</p>`;
    const canvas = this.querySelector("canvas");
    const label = this.querySelector(".label");
    const gl = canvas.getContext("webgl", { antialias: true, premultipliedAlpha: false });
    if (!gl) { label.textContent = "3D view not available in this browser"; return; }
    // Flat-shaded: each corner carries its triangle's normal. A textured group samples
    // its image at the corner's uv; an untextured one uses its colour.
    const vs = `attribute vec3 p; attribute vec3 n; attribute vec3 c; attribute vec2 t; uniform mat4 mvp; uniform mat3 rot;
      varying vec3 col; varying vec2 uv; varying float light; void main() {
        vec3 nn = normalize(rot * n);
        light = 0.35 + 0.65 * max(dot(nn, normalize(vec3(-0.5, 0.7, 0.6))), 0.0);
        col = c; uv = t; gl_Position = mvp * vec4(p, 1.0); }`;
    const fs = `precision mediump float; varying vec3 col; varying vec2 uv; varying float light;
      uniform sampler2D tex; uniform float textured;
      void main() { vec3 base = textured > 0.5 ? texture2D(tex, uv).rgb : col; gl_FragColor = vec4(base * light, 1.0); }`;
    const shader = (type, src) => { const s = gl.createShader(type); gl.shaderSource(s, src); gl.compileShader(s); return s; };
    const prog = gl.createProgram();
    gl.attachShader(prog, shader(gl.VERTEX_SHADER, vs));
    gl.attachShader(prog, shader(gl.FRAGMENT_SHADER, fs));
    gl.linkProgram(prog);
    gl.useProgram(prog);
    const loc = { p: gl.getAttribLocation(prog, "p"), n: gl.getAttribLocation(prog, "n"), c: gl.getAttribLocation(prog, "c"),
      t: gl.getAttribLocation(prog, "t"), mvp: gl.getUniformLocation(prog, "mvp"), rot: gl.getUniformLocation(prog, "rot"),
      tex: gl.getUniformLocation(prog, "tex"), textured: gl.getUniformLocation(prog, "textured") };
    let parts = []; // per group: { first, count, texture (a WebGL texture once its image loads) }
    // WebGL 1 repeats a texture only if its sides are powers of two, and the original
    // models' uvs do run past 0..1, so each image is redrawn onto a 128 × 128 canvas.
    const textureFrom = (url, part) => {
      const img = new Image();
      img.onload = () => {
        const c = document.createElement("canvas");
        c.width = c.height = 128;
        c.getContext("2d").drawImage(img, 0, 0, 128, 128);
        const tx = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, tx);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, c);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        part.texture = tx;
      };
      img.src = url;
    };
    const buf = gl.createBuffer();
    let count = 0;
    let quat = [0, 0, 0, 1];
    let name = null;
    // Expand each triangle with its own flat normal: [x y z nx ny nz r g b u v] per corner.
    const load = (m) => {
      const out = [];
      for (const tp of parts) if (tp.texture) gl.deleteTexture(tp.texture);
      parts = [];
      for (const g of m.groups || []) {
        const P = g.positions, I = g.indices, [r, gg, b] = g.color, U = g.uvs || [];
        const part = { first: out.length / 11, count: 0, texture: null };
        for (let i = 0; i + 2 < I.length; i += 3) {
          const a = I[i] * 3, b2 = I[i + 1] * 3, c = I[i + 2] * 3;
          const u = [P[b2] - P[a], P[b2 + 1] - P[a + 1], P[b2 + 2] - P[a + 2]];
          const v = [P[c] - P[a], P[c + 1] - P[a + 1], P[c + 2] - P[a + 2]];
          let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
          const len = Math.hypot(...n) || 1;
          n = n.map((x) => x / len);
          for (const k of [a, b2, c]) {
            const q = (k / 3) * 2;
            out.push(P[k], P[k + 1], P[k + 2], ...n, r, gg, b, U[q] || 0, U[q + 1] || 0);
          }
        }
        part.count = out.length / 11 - part.first;
        parts.push(part);
        if (g.texture) textureFrom(g.texture, part);
      }
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(out), gl.STATIC_DRAW);
      count = out.length / 11;
      name = m.name;
    };
    link.addEventListener("model", (e) => load(e.detail));
    link.addEventListener("telemetry", (e) => {
      const t = e.detail.target;
      if (t && t.rotation) quat = t.rotation;
      label.textContent = t ? (count ? "" : t.name) : "No target";
      this.classList.toggle("empty", !t || !count);
    });
    // Quaternion (x, y, z, w) → 3×3 rotation, column-major (as WebGL expects).
    const mat3 = ([x, y, z, w]) => [
      1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w),
      2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w),
      2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y)];
    const draw = () => {
      requestAnimationFrame(draw);
      const w = canvas.clientWidth, h = canvas.clientHeight;
      if (!w || !h || !count || this.classList.contains("empty")) return;
      if (canvas.width !== w * devicePixelRatio) { canvas.width = w * devicePixelRatio; canvas.height = h * devicePixelRatio; }
      gl.viewport(0, 0, canvas.width, canvas.height);
      gl.clearColor(0, 0, 0, 0); // transparent: the themed panel shows through
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
      gl.enable(gl.DEPTH_TEST);
      // The player looks along −Z; the target sits 3 radii ahead, turned by `quat`.
      const r = mat3(quat), f = 1 / Math.tan(0.35), aspect = w / h, near = 0.1, far = 10, d = -3;
      const P = [f / aspect, 0, 0, 0, 0, f, 0, 0, 0, 0, (far + near) / (near - far), -1, 0, 0, 2 * far * near / (near - far), 0];
      const M = [r[0], r[1], r[2], 0, r[3], r[4], r[5], 0, r[6], r[7], r[8], 0, 0, 0, d, 1];
      const mvp = new Array(16).fill(0);
      for (let i = 0; i < 4; i++) for (let j = 0; j < 4; j++) for (let k = 0; k < 4; k++) mvp[j * 4 + i] += P[k * 4 + i] * M[j * 4 + k];
      gl.uniformMatrix4fv(loc.mvp, false, new Float32Array(mvp));
      gl.uniformMatrix3fv(loc.rot, false, new Float32Array(r));
      gl.bindBuffer(gl.ARRAY_BUFFER, buf);
      const stride = 44;
      gl.enableVertexAttribArray(loc.p); gl.vertexAttribPointer(loc.p, 3, gl.FLOAT, false, stride, 0);
      gl.enableVertexAttribArray(loc.n); gl.vertexAttribPointer(loc.n, 3, gl.FLOAT, false, stride, 12);
      gl.enableVertexAttribArray(loc.c); gl.vertexAttribPointer(loc.c, 3, gl.FLOAT, false, stride, 24);
      if (loc.t >= 0) { gl.enableVertexAttribArray(loc.t); gl.vertexAttribPointer(loc.t, 2, gl.FLOAT, false, stride, 36); }
      gl.uniform1i(loc.tex, 0);
      gl.activeTexture(gl.TEXTURE0);
      for (const part of parts) {
        gl.uniform1f(loc.textured, part.texture ? 1 : 0);
        if (part.texture) gl.bindTexture(gl.TEXTURE_2D, part.texture);
        gl.drawArrays(gl.TRIANGLES, part.first, part.count);
      }
    };
    requestAnimationFrame(draw);
  }
}

/** The 3D map (C4), after TIE Fighter's: the player at the centre, a reference grid at
 *  the player's height, every radar contact on a height stalk down (or up) to the grid,
 *  in its relation colour, with its name, range and a short velocity tick; range rings
 *  at a quarter, half and full radar range. Drag to orbit, pinch or wheel to zoom, and
 *  tap a contact (or a row of the list) to target it. Positions come in the player's own
 *  axes (x right, y up, z ahead), so "up the screen" in the top view is the nose. */
class OiMap extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<div class="views"><button data-v="behind">Behind</button><button data-v="top">Top</button>
      <button data-v="side">Side</button></div><canvas></canvas><ol class="list"></ol>`;
    const canvas = this.querySelector("canvas");
    const g = canvas.getContext("2d");
    const list = this.querySelector(".list");
    const presets = { behind: [0, 0.45], top: [0, 1.45], side: [1.5, 0.15] };
    let [yaw, pitch] = presets.behind;
    let zoom = 1;
    let data = { range: 6000, list: [] };
    let shown = []; // [{id, x, y}] screen positions of the last draw, for taps
    const colour = { friendly: "#3ddc6b", hostile: "#ff5c4d", neutral: "#f2d36b", objective: "#6bc8ff" };
    for (const b of this.querySelectorAll("[data-v]")) b.addEventListener("click", () => { [yaw, pitch] = presets[b.dataset.v]; });

    // Orbit: rotate the player's axes by yaw (about up) then pitch (about right), then
    // perspective-project, the camera standing back from the player.
    const project = (p, w, h) => {
      const s = (data.range / zoom) || 1;
      let [x, y, z] = [p[0] / s, p[1] / s, p[2] / s];
      [x, z] = [x * Math.cos(yaw) - z * Math.sin(yaw), x * Math.sin(yaw) + z * Math.cos(yaw)];
      [y, z] = [y * Math.cos(pitch) + z * Math.sin(pitch), -y * Math.sin(pitch) + z * Math.cos(pitch)];
      const depth = 2.2 - z; // the camera is 2.2 units back, looking ahead (+z in, away)
      const f = Math.min(w, h * 1.6) * 0.75 / Math.max(depth, 0.2);
      return [w / 2 + x * f, h / 2 - y * f, depth];
    };
    const target = (id) => send({ type: "target", id });

    const draw = () => {
      requestAnimationFrame(draw);
      if (this.offsetParent === null) return; // hidden tab: nothing to draw
      const w = canvas.clientWidth, h = canvas.clientHeight;
      if (canvas.width !== Math.round(w * devicePixelRatio)) {
        canvas.width = Math.round(w * devicePixelRatio); canvas.height = Math.round(h * devicePixelRatio);
      }
      g.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
      g.clearRect(0, 0, w, h);
      const R = data.range;
      // Grid: range rings and spokes on the plane at the player's height.
      g.strokeStyle = "rgba(61,220,107,0.22)";
      g.lineWidth = 1;
      for (const k of [0.25, 0.5, 1]) {
        g.beginPath();
        for (let a = 0; a <= 64; a++) {
          const t = a / 64 * Math.PI * 2;
          const [x, y] = project([Math.cos(t) * R * k, 0, Math.sin(t) * R * k], w, h);
          a ? g.lineTo(x, y) : g.moveTo(x, y);
        }
        g.stroke();
      }
      for (let a = 0; a < 8; a++) {
        const t = a / 8 * Math.PI * 2;
        const [x0, y0] = project([0, 0, 0], w, h), [x1, y1] = project([Math.cos(t) * R, 0, Math.sin(t) * R], w, h);
        g.beginPath(); g.moveTo(x0, y0); g.lineTo(x1, y1); g.stroke();
      }
      // The player: a small arrowhead pointing ahead (+z).
      const tri = [[0, 0, R * 0.05], [-R * 0.025, 0, -R * 0.025], [R * 0.025, 0, -R * 0.025]].map((p) => project(p, w, h));
      g.fillStyle = "#c8f5d2";
      g.beginPath(); g.moveTo(tri[0][0], tri[0][1]); g.lineTo(tri[1][0], tri[1][1]); g.lineTo(tri[2][0], tri[2][1]); g.fill();
      // Contacts, far ones first so near ones draw on top.
      const items = data.list.map((c) => ({ c, s: project(c.pos, w, h) })).sort((a, b) => b.s[2] - a.s[2]);
      shown = [];
      g.font = "11px Helvetica, Arial, sans-serif";
      for (const { c, s } of items) {
        const col = colour[c.relation] || "#ccc";
        const base = project([c.pos[0], 0, c.pos[2]], w, h);
        g.strokeStyle = col; g.globalAlpha = 0.55;
        g.beginPath(); g.moveTo(base[0], base[1]); g.lineTo(s[0], s[1]); g.stroke();
        g.globalAlpha = 1;
        const v = project([c.pos[0] + c.vel[0] * 2, c.pos[1] + c.vel[1] * 2, c.pos[2] + c.vel[2] * 2], w, h);
        g.beginPath(); g.moveTo(s[0], s[1]); g.lineTo(v[0], v[1]); g.stroke();
        g.fillStyle = col;
        const size = c.station ? 5 : 3.5;
        c.station ? g.fillRect(s[0] - size, s[1] - size, size * 2, size * 2)
          : (g.beginPath(), g.arc(s[0], s[1], size, 0, Math.PI * 2), g.fill());
        if (c.target) { g.beginPath(); g.arc(s[0], s[1], 9, 0, Math.PI * 2); g.stroke(); }
        const dist = Math.hypot(...c.pos);
        g.fillText(`${c.name} ${(dist / 1000).toFixed(1)}`, s[0] + 8, s[1] - 6);
        shown.push({ id: c.id, x: s[0], y: s[1] });
      }
    };
    requestAnimationFrame(draw);

    // Drag to orbit; pinch or wheel to zoom; a tap (little movement) targets.
    const pts = new Map();
    let moved = 0, pinch = 0;
    canvas.addEventListener("pointerdown", (e) => { canvas.setPointerCapture(e.pointerId); pts.set(e.pointerId, [e.clientX, e.clientY]); moved = 0; });
    canvas.addEventListener("pointermove", (e) => {
      if (!pts.has(e.pointerId)) return;
      const [px, py] = pts.get(e.pointerId);
      pts.set(e.pointerId, [e.clientX, e.clientY]);
      if (pts.size === 2) {
        const [a, b] = [...pts.values()];
        const d = Math.hypot(a[0] - b[0], a[1] - b[1]);
        if (pinch) zoom = Math.min(8, Math.max(0.5, zoom * d / pinch));
        pinch = d;
        return;
      }
      const dx = e.clientX - px, dy = e.clientY - py;
      moved += Math.abs(dx) + Math.abs(dy);
      yaw += dx * 0.01;
      pitch = Math.min(1.55, Math.max(-1.55, pitch + dy * 0.01));
    });
    const up = (e) => {
      pts.delete(e.pointerId);
      if (pts.size < 2) pinch = 0;
      if (moved < 8 && e.type === "pointerup") {
        const r = canvas.getBoundingClientRect(), x = e.clientX - r.left, y = e.clientY - r.top;
        const hit = shown.map((s) => ({ s, d: Math.hypot(s.x - x, s.y - y) })).sort((a, b) => a.d - b.d)[0];
        if (hit && hit.d < 28) target(hit.s.id);
      }
    };
    canvas.addEventListener("pointerup", up);
    canvas.addEventListener("pointercancel", up);
    canvas.addEventListener("wheel", (e) => { e.preventDefault(); zoom = Math.min(8, Math.max(0.5, zoom * Math.exp(-e.deltaY * 0.002))); }, { passive: false });

    link.addEventListener("contacts", (e) => {
      data = e.detail;
      const rows = [...data.list].sort((a, b) => Math.hypot(...a.pos) - Math.hypot(...b.pos));
      list.innerHTML = "";
      for (const c of rows.slice(0, 14)) {
        const li = document.createElement("li");
        li.className = c.relation + (c.target ? " on" : "");
        li.textContent = `${c.name}  ${(Math.hypot(...c.pos) / 1000).toFixed(1)} km`;
        li.addEventListener("click", () => target(c.id));
        list.appendChild(li);
      }
    });
  }
}

/** Power: beam and shield charge levels as vertical sliders (A = lowest), within the
 *  power budget; the game lowers the other setting if the budget needs it. */
class OiPower extends HTMLElement {
  connectedCallback() {
    const slider = (k, l) => `<label><span>${l}</span><input data-s="${k}" type="range" min="0" max="4" step="1" value="0"><b data-v="${k}">A</b></label>`;
    this.innerHTML = `<h2>Power</h2><div class="sliders">${slider("beam", "Beams")}${slider("shield", "Shields")}</div>
      <p class="budget">—</p>`;
    let dragging = null;
    for (const input of this.querySelectorAll("input")) {
      input.addEventListener("pointerdown", () => { dragging = input; });
      input.addEventListener("pointerup", () => { dragging = null; });
      input.addEventListener("input", () => {
        this.querySelector(`[data-v="${input.dataset.s}"]`).textContent = LEVELS[input.value];
        send({ type: "power", [input.dataset.s]: Number(input.value) });
      });
    }
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive) return;
      const [beam, shield, levels, used, budget] = t.power;
      for (const [k, v] of [["beam", beam], ["shield", shield]]) {
        const input = this.querySelector(`[data-s="${k}"]`);
        input.max = Math.max(0, levels - 1);
        if (input !== dragging) input.value = v;
        this.querySelector(`[data-v="${k}"]`).textContent = LEVELS[input.value];
      }
      this.querySelector(".budget").textContent = `${used} of ${budget} power in use`;
    });
  }
}

/** Function-group tiles: a status card per group. Tapping a card opens a full-width
 *  strip with that group's controls and folds the other cards to their titles; tapping
 *  the open title closes it; tapping another title switches to that group. */
class OiTiles extends HTMLElement {
  connectedCallback() {
    this.open = null;
    link.addEventListener("catalogue", () => this.render());
    link.addEventListener("telemetry", () => this.status());
    if (catalogue) this.render();
  }

  /** The one-line status of a group, from the latest telemetry. */
  line(id) {
    const t = latest;
    if (!t || !t.alive) return "—";
    const off = catalogue.actions.filter((a) => a.group === id && !a.available).length;
    const missing = off ? ` · ${off} not fitted` : "";
    const sys = (k) => (t.systems || []).find((s) => s.kind === k);
    switch (id) {
      case "flight": return `drift ${signed(t.velocity[0])} / ${signed(t.velocity[1])}${missing}`;
      case "throttle": return `${Math.round(t.velocity[2])} m/s · ${pct(t.throttle)}${t.afterburner ? " · BURN" : ""}`;
      case "weapons": return `${t.link.toUpperCase()} · ${t.missile ? `${t.missile.name} ×${t.missile.count}` : "no missiles"}${missing}`;
      case "targeting": return t.target ? `${t.target.name} · ${(t.target.distance / 1000).toFixed(1)} km` : "no target";
      case "power": {
        const on = ["cloak", "ecm", "tractor"].filter((k) => sys(k) && sys(k).on).map((k) => k.toUpperCase());
        return `Beam ${LEVELS[t.power[0]]} · Shield ${LEVELS[t.power[1]]} · ${t.balance}${on.length ? " · " + on.join(" ") : ""}${missing}`;
      }
      default: return missing ? missing.slice(3) : "ready";
    }
  }

  render() {
    if (!catalogue) return;
    this.innerHTML = `<div class="cards"></div><div class="strip" hidden></div>`;
    const cards = this.querySelector(".cards");
    for (const g of catalogue.groups) {
      const card = document.createElement("div");
      card.className = "card";
      card.dataset.g = g.id;
      card.innerHTML = `<h3>${g.title}</h3><p class="line">—</p>`;
      card.addEventListener("click", () => {
        this.open = this.open === g.id ? null : g.id;
        this.layout();
      });
      cards.appendChild(card);
    }
    this.layout();
    this.status();
  }

  layout() {
    this.classList.toggle("folded", this.open !== null);
    for (const c of this.querySelectorAll(".card")) c.classList.toggle("on", c.dataset.g === this.open);
    const strip = this.querySelector(".strip");
    strip.hidden = this.open === null;
    strip.innerHTML = "";
    if (this.open === null) return;
    if (this.open === "throttle") strip.appendChild(document.createElement("oi-throttle"));
    const box = document.createElement("div");
    box.className = "buttons";
    for (const a of catalogue.actions.filter((a) => a.group === this.open)) box.appendChild(makeButton(a));
    strip.appendChild(box);
  }

  status() {
    if (!catalogue) return;
    for (const c of this.querySelectorAll(".card")) c.querySelector(".line").textContent = this.line(c.dataset.g);
  }
}

const MIN_LIT = 150; // ms a tapped button stays lit
const STATIC_MS = 400; // ms of static after a hit

customElements.define("oi-static", OiStatic);
customElements.define("oi-pairing", OiPairing);
customElements.define("oi-telemetry", OiTelemetry);
customElements.define("oi-throttle", OiThrottle);
customElements.define("oi-actions", OiActions);
customElements.define("oi-mode", OiMode);
customElements.define("oi-hud-strip", OiHudStrip);
customElements.define("oi-shields", OiShields);
customElements.define("oi-target-view", OiTargetView);
customElements.define("oi-map", OiMap);
customElements.define("oi-display", OiDisplay);
customElements.define("oi-power", OiPower);
customElements.define("oi-tiles", OiTiles);
