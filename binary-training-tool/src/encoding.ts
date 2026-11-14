// Byte-at-a-time UTF-8 and UTF-16 decoders. Written by hand (not TextDecoder) so the
// pages can show partial sequences and say exactly why a byte is invalid.

export type Decoded =
  | { kind: "char"; bytes: number[]; cp: number; warning?: string }
  | { kind: "error"; bytes: number[]; reason: string };

// ---------------------------------------------------------------- UTF-8
//
// This mirrors the Eight-Switch firmware's decoder (Processor::check_for_utf32val in
// firmware/src/main.rs) so the page stays in step with what the keyboard types:
//   - C0–DF start 2-byte, E0–EF 3-byte, F0–F7 4-byte sequences (no stricter lead checks)
//   - 80–BF with no lead, and F8–FF, are errors
//   - a non-continuation byte mid-sequence is an error and is discarded along with the sequence
//   - overlong, surrogate and > U+10FFFF results are still produced (the keyboard types them),
//     but are flagged with a warning because they are not valid UTF-8.

export interface Utf8State {
  /** Bytes of the sequence in progress (lead byte first). Empty when idle. */
  pending: number[];
  /** Total length of the sequence in progress, 0 when idle. */
  length: number;
}

export const UTF8_IDLE: Utf8State = { pending: [], length: 0 };

export type Utf8Role = "ascii" | "lead" | "continuation" | "invalid";

/** What a byte is from its bit pattern alone, and how many leading marker bits it has. */
export function utf8Role(b: number): { role: Utf8Role; markerBits: number; length: number } {
  if (b < 0x80) return { role: "ascii", markerBits: 1, length: 1 };
  if (b < 0xc0) return { role: "continuation", markerBits: 2, length: 0 };
  if (b < 0xe0) return { role: "lead", markerBits: 3, length: 2 };
  if (b < 0xf0) return { role: "lead", markerBits: 4, length: 3 };
  if (b < 0xf8) return { role: "lead", markerBits: 5, length: 4 };
  return { role: "invalid", markerBits: 8, length: 0 };
}

/** Why a lead byte can never start valid UTF-8, if it can't (the keyboard still accepts it). */
export function utf8LeadWarning(b: number): string | undefined {
  if (b === 0xc0 || b === 0xc1) return "C0/C1 can only produce overlong encodings";
  if (b >= 0xf5 && b < 0xf8) return "F5–F7 can only produce code points above U+10FFFF";
  return undefined;
}

const MIN_FOR_LENGTH = [0, 0, 0x80, 0x800, 0x10000];

/** Why a decoded code point is not valid UTF-8, if it isn't. */
export function utf8Warning(cp: number, length: number): string | undefined {
  if (cp < MIN_FOR_LENGTH[length]) return `overlong: U+${cp.toString(16).toUpperCase().padStart(4, "0")} fits in fewer bytes; not valid UTF-8`;
  if (cp >= 0xd800 && cp <= 0xdfff) return "surrogate code point (U+D800–DFFF); not valid UTF-8";
  if (cp > 0x10ffff) return "above U+10FFFF, the highest code point; not valid UTF-8";
  return undefined;
}

export function utf8Decode(bytes: number[]): number {
  const { markerBits } = utf8Role(bytes[0]);
  let cp = bytes[0] & (0xff >> markerBits);
  for (const b of bytes.slice(1)) cp = (cp << 6) | (b & 0x3f);
  return cp;
}

/** Feeds one byte. Returns the new state and any characters/errors it produced. */
export function utf8Step(state: Utf8State, b: number): { state: Utf8State; out: Decoded[] } {
  if (state.length === 0) {
    const { role, length } = utf8Role(b);
    if (role === "ascii") return { state: UTF8_IDLE, out: [{ kind: "char", bytes: [b], cp: b }] };
    if (role === "lead") return { state: { pending: [b], length }, out: [] };
    const reason = role === "continuation" ? "continuation byte (10xxxxxx) with no lead byte before it" : "11111xxx is never valid in UTF-8";
    return { state: UTF8_IDLE, out: [{ kind: "error", bytes: [b], reason }] };
  }

  const bytes = [...state.pending, b];
  if ((b & 0xc0) !== 0x80) {
    return {
      state: UTF8_IDLE,
      out: [{ kind: "error", bytes, reason: "expected a continuation byte (10xxxxxx); sequence discarded" }],
    };
  }
  if (bytes.length < state.length) return { state: { pending: bytes, length: state.length }, out: [] };
  const cp = utf8Decode(bytes);
  return { state: UTF8_IDLE, out: [{ kind: "char", bytes, cp, warning: utf8Warning(cp, bytes.length) }] };
}

// ---------------------------------------------------------------- UTF-16 (big-endian)

export interface Utf16State {
  /** First (high) byte of the code unit in progress. */
  highByte: number | null;
  /** A high surrogate waiting for its low surrogate. */
  highSurrogate: number | null;
  /** Bytes making up the pending surrogate, for display/error reporting. */
  surrogateBytes: number[];
}

export const UTF16_IDLE: Utf16State = { highByte: null, highSurrogate: null, surrogateBytes: [] };

export const isHighSurrogate = (u: number) => u >= 0xd800 && u <= 0xdbff;
export const isLowSurrogate = (u: number) => u >= 0xdc00 && u <= 0xdfff;

function unitStep(state: Utf16State, unit: number, bytes: number[]): { state: Utf16State; out: Decoded[] } {
  if (state.highSurrogate !== null) {
    if (isLowSurrogate(unit)) {
      const cp = 0x10000 + ((state.highSurrogate - 0xd800) << 10) + (unit - 0xdc00);
      return { state: UTF16_IDLE, out: [{ kind: "char", bytes: [...state.surrogateBytes, ...bytes], cp }] };
    }
    const err: Decoded = {
      kind: "error",
      bytes: state.surrogateBytes,
      reason: "high surrogate not followed by a low surrogate (DC00–DFFF)",
    };
    const next = unitStep(UTF16_IDLE, unit, bytes);
    return { state: next.state, out: [err, ...next.out] };
  }
  if (isHighSurrogate(unit)) return { state: { highByte: null, highSurrogate: unit, surrogateBytes: bytes }, out: [] };
  if (isLowSurrogate(unit)) {
    return { state: UTF16_IDLE, out: [{ kind: "error", bytes, reason: "low surrogate with no high surrogate before it" }] };
  }
  return { state: UTF16_IDLE, out: [{ kind: "char", bytes, cp: unit }] };
}

export function utf16Step(state: Utf16State, b: number): { state: Utf16State; out: Decoded[] } {
  if (state.highByte === null) return { state: { ...state, highByte: b }, out: [] };
  const unit = (state.highByte << 8) | b;
  return unitStep({ ...state, highByte: null }, unit, [state.highByte, b]);
}
