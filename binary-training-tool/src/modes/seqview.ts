// Shared pieces for the byte-sequence pages (UTF-8, UTF-16).

import type { Decoded } from "../encoding";
import { codePoint, el, hex, printable } from "./mode";

/** 8 (or more) bit cells; the first `markerBits` are drawn as format/marker bits. */
export function bitCells(value: number, width: number, markerBits = 0, extra = ""): HTMLElement {
  const row = el("div", `bits ${extra}`);
  for (let i = width - 1; i >= 0; i--) {
    const isMarker = width - 1 - i < markerBits;
    row.append(el("span", isMarker ? "bit marker" : "bit payload", String((value >> i) & 1)));
  }
  return row;
}

export const hexBytes = (bytes: number[]) => bytes.map((b) => hex(b)).join(" ");

export function describe(out: Decoded[]): { text: string; bad: boolean; warn: boolean } {
  const parts = out.map((d) =>
    d.kind === "char"
      ? `${codePoint(d.cp)} “${printable(d.cp)}”${d.warning ? ` (${d.warning})` : ""}`
      : `error: ${d.reason}`,
  );
  return {
    text: parts.join(", then "),
    bad: out.some((d) => d.kind === "error"),
    warn: out.some((d) => d.kind === "char" && !!d.warning),
  };
}

/** Running text plus a history of decoded sequences, kept across tab switches by the caller. */
export class Transcript {
  text = "";
  history: Decoded[] = [];

  add(out: Decoded[]) {
    for (const d of out) {
      if (d.kind === "char") this.text += printable(d.cp);
      this.history.unshift(d);
    }
    this.history.length = Math.min(this.history.length, 12);
  }

  clear() {
    this.text = "";
    this.history = [];
  }

  render(textEl: HTMLElement, historyEl: HTMLElement) {
    textEl.textContent = this.text;
    historyEl.innerHTML = "";
    if (!this.history.length) {
      historyEl.append(el("div", "history-empty", "Nothing sent yet"));
      return;
    }
    for (const d of this.history) {
      const row = el("div", d.kind === "error" ? "history-row bad" : d.warning ? "history-row warn" : "history-row");
      row.append(el("span", "history-bytes", hexBytes(d.bytes)));
      if (d.kind === "char") {
        row.append(el("span", "history-cp", codePoint(d.cp)), el("span", "history-glyph", printable(d.cp)));
        if (d.warning) row.append(el("span", "history-reason", d.warning));
      } else {
        row.append(el("span", "history-reason", d.reason));
      }
      historyEl.append(row);
    }
  }
}

export function section(title: string, ...children: HTMLElement[]): HTMLElement {
  const s = el("section", "seq-section");
  s.append(el("h3", "", title), ...children);
  return s;
}
