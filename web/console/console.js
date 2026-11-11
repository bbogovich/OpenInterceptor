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
    // Out of sight, but still "playing" as far as the browser is concerned.
    wakeVideo.style.cssText = "position:fixed;width:1px;height:1px;opacity:0.01;pointer-events:none;bottom:0;left:0";
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
    this.innerHTML = `<label for="thr">Throttle</label><input id="thr" type="range" min="0" max="100" value="0">
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
    link.addEventListener("telemetry", (e) => {
      if (!dragging && e.detail.alive) {
        input.value = Math.round(e.detail.throttle * 100);
        out.textContent = `${input.value}%`;
      }
    });
  }
}

/** Every action from the game's catalogue, grouped. Held actions send down/up; taps send once. */
class OiActions extends HTMLElement {
  connectedCallback() {
    link.addEventListener("catalogue", (e) => this.build(e.detail));
  }

  build(cat) {
    this.innerHTML = "";
    for (const g of cat.groups) {
      const section = document.createElement("section");
      section.innerHTML = `<h2>${g.title}</h2><div class="buttons"></div>`;
      const box = section.querySelector(".buttons");
      for (const a of cat.actions.filter((a) => a.group === g.id)) {
        const b = document.createElement("button");
        b.textContent = a.label;
        b.className = a.kind;
        b.disabled = !a.available; // not fitted on this craft: shown, but disabled
        if (a.kind === "hold") {
          const down = (ev) => { ev.preventDefault(); b.classList.add("down"); send({ type: "action", name: a.name, value: 1 }); };
          const up = () => { if (b.classList.contains("down")) { b.classList.remove("down"); send({ type: "action", name: a.name, value: 0 }); } };
          b.addEventListener("pointerdown", down);
          for (const t of ["pointerup", "pointercancel", "pointerleave"]) b.addEventListener(t, up);
        } else {
          b.addEventListener("click", () => send({ type: "action", name: a.name, value: 1 }));
        }
        box.appendChild(b);
      }
      this.appendChild(section);
    }
  }
}

customElements.define("oi-pairing", OiPairing);
customElements.define("oi-telemetry", OiTelemetry);
customElements.define("oi-throttle", OiThrottle);
customElements.define("oi-actions", OiActions);
