import type { State } from "../store";

export interface Mode {
  id: string;
  label: string;
  /** Builds the mode's DOM inside root. Called each time the mode is selected. */
  mount(root: HTMLElement): void;
  /** Called on every switch change while mounted (and once after mount). */
  update(state: State): void;
  /** Called on the send button / Enter while mounted. */
  onSend?(state: State): void;
  unmount?(): void;
}

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className = "",
  text = "",
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

export const bin = (v: number, width = 8) => v.toString(2).padStart(width, "0");
export const hex = (v: number, width = 2) => v.toString(16).toUpperCase().padStart(width, "0");
export const oct = (v: number) => v.toString(8).padStart(3, "0");
export const signed8 = (v: number) => (v & 0x80 ? v - 256 : v);

const CONTROL_NAMES = [
  "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI",
  "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC", "FS", "GS", "RS", "US",
];

/** Human-readable glyph for a 7-bit ASCII code (0–127). */
export function charLabel(v: number): { glyph: string; name: string } {
  if (v < 32) return { glyph: CONTROL_NAMES[v], name: "control character" };
  if (v === 32) return { glyph: "␠", name: "space" };
  if (v === 127) return { glyph: "DEL", name: "control character" };
  return { glyph: String.fromCharCode(v), name: "printable character" };
}

/** Text to append to an output area for a decoded code point. Control codes become their visible symbols. */
export function printable(cp: number): string {
  if (cp === 10 || cp === 13) return "\n";
  if (cp < 32) return String.fromCodePoint(0x2400 + cp);
  if (cp === 127) return "\u2421";
  if (cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) return "\ufffd";
  return String.fromCodePoint(cp);
}

export const codePoint = (cp: number) => `U+${hex(cp, 4)}`;
