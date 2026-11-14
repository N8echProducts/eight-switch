import { isHighSurrogate, isLowSurrogate, UTF16_IDLE, utf16Step, type Utf16State } from "../encoding";
import { getState } from "../store";
import { bin, codePoint, el, hex, printable, type Mode } from "./mode";
import { bitCells, describe, section, Transcript } from "./seqview";

let state: Utf16State = UTF16_IDLE;
const transcript = new Transcript();

let unitEl: HTMLElement;
let previewEl: HTMLElement;
let pairEl: HTMLElement;
let textEl: HTMLElement;
let historyEl: HTMLElement;

type Message = { text: string; bad: boolean };

/** Byte most recently sent, while the switches still show it; null once they change. */
let justSent: number | null = null;
/** What that send did, shown instead of a preview until the switches change. */
let lastResult: Message | null = null;

function outcome(from: Utf16State, value: number): Message {
  const { out } = utf16Step(from, value);
  if (out.length) return describe(out);
  return {
    text: from.highByte === null ? "high byte of a 16-bit code unit" : "starts a surrogate pair, a low surrogate must follow",
    bad: false,
  };
}

function unitKind(unit: number): string {
  if (isHighSurrogate(unit)) return "high surrogate (D800–DBFF): first half of a pair";
  if (isLowSurrogate(unit)) return "low surrogate (DC00–DFFF): second half of a pair";
  return `${codePoint(unit)} “${printable(unit)}”`;
}

function render(value: number) {
  // Right after a send the switches still hold the byte just sent; don't preview sending it again.
  const showingSent = justSent === value && lastResult !== null;
  const previewing = !showingSent;

  // Code unit being built: high byte (sent or on the switches), then low byte.
  unitEl.innerHTML = "";
  const hiSent = state.highByte !== null;
  const empty = (label: string) => {
    const box = el("div", "slot");
    box.append(el("div", "bits template", "xxxxxxxx"), el("div", "slot-label", label));
    return box;
  };
  let hiBox: HTMLElement;
  if (hiSent) {
    hiBox = el("div", "slot filled");
    hiBox.append(bitCells(state.highByte!, 8, 0, "byte-0"), el("div", "slot-label", `high byte 0x${hex(state.highByte!)} sent`));
  } else if (previewing) {
    hiBox = el("div", "slot preview-slot");
    hiBox.append(bitCells(value, 8, 0, "byte-0"), el("div", "slot-label", "high byte (switches, not sent)"));
  } else hiBox = empty("high byte");
  let loBox: HTMLElement;
  if (hiSent && previewing) {
    loBox = el("div", "slot preview-slot");
    loBox.append(bitCells(value, 8, 0, "byte-1"), el("div", "slot-label", "low byte (switches, not sent)"));
  } else loBox = empty("low byte");
  const slots = el("div", "slots");
  slots.append(hiBox, loBox);
  unitEl.append(slots);
  if (hiSent && previewing) {
    const unit = (state.highByte! << 8) | value;
    unitEl.append(el("div", "assembled-result", `0x${hex(unit, 4)} = ${unitKind(unit)}`));
  } else if (!hiSent && previewing) {
    unitEl.append(el("div", "assembled-result muted", `high byte 0x${hex(value)}${value >= 0xd8 && value <= 0xdf ? " starts a surrogate" : ""}`));
  }

  const message = showingSent ? lastResult! : outcome(state, value);
  previewEl.className = message.bad ? "preview bad" : "preview";
  previewEl.textContent = showingSent
    ? `Sent 0x${hex(value)}: ${message.text}. Now set the ${hiSent ? "low" : "next high"} byte and press send.`
    : `Press send → ${message.text}`;

  // Surrogate pair breakdown.
  pairEl.innerHTML = "";
  if (state.highSurrogate === null) {
    pairEl.append(el("div", "slot-empty", "Code points above U+FFFF need two code units: a high surrogate (D800–DBFF) then a low surrogate (DC00–DFFF). Each carries 10 bits of (code point − 0x10000)."));
  } else {
    const high = state.highSurrogate;
    const row = el("div", "pair");
    const h = el("div", "pair-part");
    h.append(el("div", "slot-label", `high 0x${hex(high, 4)}`), bitCells(high, 16, 6));
    row.append(h);
    const lowUnit = hiSent && previewing ? ((state.highByte! << 8) | value) : null;
    const l = el("div", "pair-part");
    if (lowUnit !== null && isLowSurrogate(lowUnit)) {
      l.append(el("div", "slot-label", `low 0x${hex(lowUnit, 4)} (switches)`), bitCells(lowUnit, 16, 6, "preview-bits"));
    } else {
      l.append(el("div", "slot-label", "low surrogate"), el("div", "bits template", "110111xxxxxxxxxx"));
    }
    row.append(l);
    pairEl.append(row);
    const assembled = el("div", "assembled");
    assembled.append(el("span", "segment byte-0", bin(high - 0xd800, 10)));
    if (lowUnit !== null && isLowSurrogate(lowUnit)) {
      assembled.append(el("span", "segment byte-1", bin(lowUnit - 0xdc00, 10)));
      const cp = 0x10000 + ((high - 0xd800) << 10) + (lowUnit - 0xdc00);
      pairEl.append(assembled, el("div", "assembled-result", `+ 0x10000 = ${codePoint(cp)}  “${printable(cp)}”`));
    } else {
      assembled.append(el("span", "segment missing", "xxxxxxxxxx"));
      pairEl.append(assembled);
    }
  }

  transcript.render(textEl, historyEl);
}

export const utf16: Mode = {
  id: "utf16",
  label: "UTF-16",
  mount(root) {
    unitEl = el("div");
    previewEl = el("div", "preview");
    pairEl = el("div");
    textEl = el("div", "typed");
    historyEl = el("div", "history");
    const reset = el("button", "ghost", "Abandon code unit");
    reset.addEventListener("click", () => {
      state = UTF16_IDLE;
      render(getState().value);
    });
    const clear = el("button", "ghost", "Clear all");
    clear.addEventListener("click", () => {
      state = UTF16_IDLE;
      transcript.clear();
      render(getState().value);
    });
    const buttons = el("div", "button-row");
    buttons.append(reset, clear);
    root.append(
      el("p", "intro", "UTF-16 code units are 16 bits, so each one takes two sends: the high byte first, then the low byte (big-endian, UTF-16BE)."),
      section("Code unit", unitEl, previewEl),
      section("Surrogate pair", pairEl),
      section("Decoded text", textEl, buttons),
      section("History", historyEl),
    );
  },
  update({ value }) {
    if (value !== justSent) justSent = null;
    render(value);
  },
  onSend({ value }) {
    lastResult = outcome(state, value);
    justSent = value;
    const r = utf16Step(state, value);
    state = r.state;
    transcript.add(r.out);
    render(value);
  },
};
