// OpenInterceptor control console (protocol v1; see the docs site, Specification › Console
// protocol). Plain custom elements: each element owns one part of the page and reacts to
// the shared `link` (the WebSocket) through DOM events. No framework, no build step.
"use strict";

/** The connection to the game, shared by every element. */
const link = new EventTarget();
let socket = null;

function connect(code) {
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
  socket.onclose = () => link.dispatchEvent(new CustomEvent("closed"));
}

function send(msg) {
  if (socket && socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify(msg));
}

/** Pairing: the 6-digit code shown in the game. Remembered for this browser tab. */
class OiPairing extends HTMLElement {
  connectedCallback() {
    this.innerHTML = `<form><h1>OpenInterceptor console</h1>
      <p>Enter the pairing code shown in the game.</p>
      <input inputmode="numeric" maxlength="6" autocomplete="off" aria-label="Pairing code">
      <p class="error"></p></form>`;
    const input = this.querySelector("input");
    const error = this.querySelector(".error");
    this.querySelector("form").addEventListener("submit", (e) => {
      e.preventDefault();
      sessionStorage.setItem("oi-code", input.value);
      connect(input.value);
    });
    input.addEventListener("input", () => { if (input.value.length === 6) this.querySelector("form").requestSubmit(); });
    link.addEventListener("denied", (e) => { error.textContent = e.detail.reason; input.value = ""; });
    link.addEventListener("catalogue", () => { this.hidden = true; document.getElementById("console").hidden = false; });
    link.addEventListener("closed", () => {
      this.hidden = false; document.getElementById("console").hidden = true;
      error.textContent = "Disconnected. Enter the code to reconnect.";
    });
    let saved = null;
    try { saved = sessionStorage.getItem("oi-code"); } catch (_) { /* private mode */ }
    if (saved) { input.value = saved; connect(saved); }
  }
}

/** The craft's condition, updated by every telemetry message. */
class OiTelemetry extends HTMLElement {
  connectedCallback() {
    const cells = [["craft", "Craft"], ["speed", "Speed m/s"], ["drift", "Drift X / Y"], ["hull", "Hull"],
      ["energy", "Energy"], ["shields", "Shields fore / aft"], ["missile", "Missile"], ["target", "Target"]];
    this.innerHTML = cells.map(([k, l]) =>
      `<div class="cell"><div class="label">${l}</div><div class="value" data-k="${k}">—</div></div>`).join("");
    const set = (k, v) => { this.querySelector(`[data-k="${k}"]`).textContent = v; };
    const pct = (f) => `${Math.round(f * 100)}%`;
    link.addEventListener("telemetry", (e) => {
      const t = e.detail;
      if (!t.alive) { set("craft", "DESTROYED"); return; }
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
