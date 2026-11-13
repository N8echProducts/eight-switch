import "./style.css";
import * as device from "./device";
import { getState, onSendPress, pressSend, setState, subscribe, toggleBit, type State } from "./store";
import type { Mode } from "./modes/mode";
import { bin, el } from "./modes/mode";
import { placeValues } from "./modes/placeValues";
import { bases } from "./modes/bases";
import { signed } from "./modes/signed";
import { ascii } from "./modes/ascii";
import { challenge } from "./modes/challenge";

const MODES: Mode[] = [placeValues, bases, signed, ascii, challenge];

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const switchesEl = $("switches");
const readoutEl = $("readout-bin");
const sendEl = $("send-indicator");
const hintEl = $("input-hint");
const tabsEl = $("tabs");
const modeEl = $("mode");
const statusEl = $("status");
const connectBtn = $<HTMLButtonElement>("connect");

// --- Switch bank (mirrors the device; clickable when no device is connected) ---

const switchEls: HTMLButtonElement[] = [];
for (let bit = 7; bit >= 0; bit--) {
  const sw = el("button", "switch");
  sw.setAttribute("aria-label", `Bit ${bit} (value ${1 << bit})`);
  sw.append(el("span", "switch-key", String(8 - bit)), el("span", "switch-lever"), el("span", "switch-weight", String(1 << bit)));
  sw.addEventListener("click", () => {
    if (getState().source !== "device") toggleBit(bit);
  });
  switchesEl.append(sw);
  switchEls.push(sw);
}

function renderSwitches(state: State) {
  switchEls.forEach((sw, i) => {
    const on = (state.value >> (7 - i)) & 1;
    sw.classList.toggle("on", on === 1);
    sw.setAttribute("aria-pressed", String(on === 1));
    sw.disabled = state.source === "device";
  });
  readoutEl.textContent = bin(state.value);
  sendEl.classList.toggle("active", state.button);
  hintEl.textContent =
    state.source === "device"
      ? "Reading switches from the Eight-Switch."
      : "No device: click the switches or press keys 1–8 to flip them, Enter to send, 0 to clear.";
}

// --- Modes ---

let active: Mode = MODES[0];

function selectMode(mode: Mode) {
  active.unmount?.();
  active = mode;
  modeEl.innerHTML = "";
  modeEl.dataset.mode = mode.id;
  mode.mount(modeEl);
  mode.update(getState());
  [...tabsEl.children].forEach((t) => t.classList.toggle("active", (t as HTMLElement).dataset.mode === mode.id));
  history.replaceState(null, "", `#${mode.id}`);
}

for (const mode of MODES) {
  const tab = el("button", "tab", mode.label);
  tab.dataset.mode = mode.id;
  tab.addEventListener("click", () => selectMode(mode));
  tabsEl.append(tab);
}

subscribe((state) => {
  renderSwitches(state);
  active.update(state);
});
onSendPress(() => active.onSend?.(getState()));

// --- Keyboard fallback ---

window.addEventListener("keydown", (e) => {
  if (e.metaKey || e.ctrlKey || e.altKey || e.target instanceof HTMLInputElement) return;
  if (getState().source === "device") return;
  if (e.key >= "1" && e.key <= "8") toggleBit(8 - Number(e.key));
  else if (e.key === "0") setState({ value: 0, source: "screen" });
  else if (e.key === "Enter") pressSend();
  else return;
  e.preventDefault();
});

// --- Device connection ---

device.onStatus((status, name) => {
  statusEl.className = `status ${status}`;
  if (status === "unsupported") {
    statusEl.textContent = "WebHID not supported in this browser — use Chrome or Edge to connect";
    connectBtn.hidden = true;
  } else if (status === "connected") {
    statusEl.textContent = `Connected${name ? `: ${name}` : ""}`;
    connectBtn.textContent = "Disconnect";
  } else {
    statusEl.textContent = "Not connected";
    connectBtn.textContent = "Connect Eight-Switch";
  }
});

connectBtn.addEventListener("click", async () => {
  try {
    if (connectBtn.textContent === "Disconnect") await device.disconnect();
    else await device.requestDevice();
  } catch (err) {
    statusEl.textContent = `Connection failed: ${(err as Error).message}`;
  }
});

renderSwitches(getState());
selectMode(MODES.find((m) => `#${m.id}` === location.hash) ?? MODES[0]);
void device.initAutoConnect();
