import { UTF8_IDLE, utf8Decode, utf8LeadWarning, utf8Role, utf8Step, utf8Warning, type Utf8State } from "../encoding";
import { getState } from "../store";
import { bin, codePoint, el, hex, printable, type Mode } from "./mode";
import { bitCells, describe, section, Transcript } from "./seqview";

const ROLE_LABEL = {
  ascii: "ASCII (0xxxxxxx)",
  lead: "lead byte",
  continuation: "continuation (10xxxxxx)",
  invalid: "never valid",
};

let state: Utf8State = UTF8_IDLE;
const transcript = new Transcript();

let currentEl: HTMLElement;
let previewEl: HTMLElement;
let slotsEl: HTMLElement;
let cpEl: HTMLElement;
let textEl: HTMLElement;
let historyEl: HTMLElement;

type Message = { text: string; bad: boolean; warn: boolean };

/** Byte most recently sent, while the switches still show it; null once they change. */
let justSent: number | null = null;
/** What that send did, shown instead of a preview until the switches change. */
let lastResult: Message | null = null;

/** Describes the result of sending `value` from `from`, for the preview and the "sent" message. */
function outcome(from: Utf8State, value: number): Message {
  const { state: next, out } = utf8Step(from, value);
  if (out.length) return describe(out);
  if (from.length === 0) {
    const warning = utf8LeadWarning(value);
    return {
      text: `starts a ${next.length}-byte sequence, ${next.length - 1} continuation byte(s) to follow${warning ? ` (${warning})` : ""}`,
      bad: false,
      warn: !!warning,
    };
  }
  return { text: `continuation byte ${next.pending.length} of ${next.length}`, bad: false, warn: false };
}

function template(index: number, length: number): string {
  if (index > 0) return "10xxxxxx";
  return ["0xxxxxxx", "110xxxxx", "1110xxxx", "11110xxx"][length - 1];
}

function render(value: number) {
  const { role, markerBits, length } = utf8Role(value);

  currentEl.innerHTML = "";
  currentEl.append(
    bitCells(value, 8, role === "invalid" && markerBits === 8 ? 5 : markerBits, role),
    el("div", `role-badge ${role}`, role === "lead" ? `lead byte: ${length}-byte sequence` : ROLE_LABEL[role]),
  );

  // Right after a send the switches still hold the byte just sent. Previewing a second send of it
  // would read like an error (e.g. F0 then F0), so report what the send did until the switches change.
  const showingSent = justSent === value && lastResult !== null;
  const message = showingSent ? lastResult! : outcome(state, value);
  const next = state.length ? "set the next byte (10xxxxxx)" : "set the next byte";
  previewEl.className = message.bad ? "preview bad" : message.warn ? "preview warn" : "preview";
  previewEl.textContent = showingSent
    ? `Sent 0x${hex(value)}: ${message.text}. Now ${next} and press send.`
    : `Press send → ${message.text}`;

  // Sequence slots: entered bytes, then where the current switches would go, then empty templates.
  // The switches "fit" if sending them would not produce an error.
  const { out } = utf8Step(state, value);
  const accepted = !showingSent && !out.some((d) => d.kind === "error");
  const seqLength = state.length || (accepted ? length : 0);
  const bytes = accepted ? [...state.pending, value] : [...state.pending];
  slotsEl.innerHTML = "";
  if (seqLength === 0) {
    slotsEl.append(el("div", "slot-empty", "Set a valid ASCII or lead byte to start a sequence"));
  }
  for (let i = 0; i < seqLength; i++) {
    const slot = el("div", "slot");
    const isPreview = i === bytes.length - 1 && accepted;
    if (i < bytes.length) {
      slot.classList.add(isPreview ? "preview-slot" : "filled");
      slot.append(bitCells(bytes[i], 8, utf8Role(bytes[i]).markerBits, `byte-${i}`));
      slot.append(el("div", "slot-label", isPreview ? "switches (not sent)" : `byte ${i + 1} sent`));
    } else {
      slot.append(el("div", "bits template", template(i, seqLength)), el("div", "slot-label", `byte ${i + 1}`));
    }
    slotsEl.append(slot);
  }

  // The code point assembled from the payload bits of each byte.
  cpEl.innerHTML = "";
  if (bytes.length && seqLength) {
    const assembled = el("div", "assembled");
    bytes.forEach((b, i) => {
      const m = utf8Role(b).markerBits;
      assembled.append(el("span", `segment byte-${i}`, bin(b & (0xff >> m), 8 - m)));
    });
    for (let i = bytes.length; i < seqLength; i++) assembled.append(el("span", "segment missing", "xxxxxx"));
    cpEl.append(assembled);
    if (bytes.length === seqLength) {
      const cp = utf8Decode(bytes);
      const warning = utf8Warning(cp, bytes.length);
      cpEl.append(el("div", warning ? "assembled-result warn" : "assembled-result", `= ${codePoint(cp)}  “${printable(cp)}”`));
      if (warning) cpEl.append(el("div", "preview warn", `${warning}, but the Eight-Switch still types it`));
    }
  }

  transcript.render(textEl, historyEl);
}

export const utf8: Mode = {
  id: "utf8",
  label: "UTF-8",
  mount(root) {
    currentEl = el("div", "current-byte");
    previewEl = el("div", "preview");
    slotsEl = el("div", "slots");
    cpEl = el("div", "codepoint");
    textEl = el("div", "typed");
    historyEl = el("div", "history");
    // Only clears the page's text and history. The decoder state is left alone so it stays in step
    // with the keyboard's own decoder, which has no reset. To abandon a sequence, send a byte that
    // isn't a continuation byte (e.g. 00000000); both sides discard it.
    const clear = el("button", "ghost", "Clear text");
    clear.addEventListener("click", () => {
      transcript.clear();
      render(getState().value);
    });
    const buttons = el("div", "button-row");
    buttons.append(clear);
    root.append(
      el("p", "intro", "Set a byte on the switches and press send to add it to the UTF-8 sequence. Marker bits are dimmed; payload bits make up the code point. This page follows the same rules as the keyboard's own UTF-8 decoder. To abandon a sequence, send a byte that isn't 10xxxxxx."),
      section("Switches", currentEl, previewEl),
      section("Sequence", slotsEl, cpEl),
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
    const r = utf8Step(state, value);
    state = r.state;
    transcript.add(r.out);
    render(value);
  },
};
