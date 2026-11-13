import { bin, el, hex, oct, type Mode } from "./mode";

let rows: Record<string, HTMLElement> = {};
let nibbles: HTMLElement;

function row(label: string, prefix: string): [HTMLElement, HTMLElement] {
  const r = el("div", "base-row");
  const value = el("span", "base-value");
  r.append(el("span", "base-label", label), el("span", "base-prefix", prefix), value);
  return [r, value];
}

export const bases: Mode = {
  id: "bases",
  label: "Bases",
  mount(root) {
    const list = el("div", "base-list");
    rows = {};
    for (const [key, label, prefix] of [
      ["bin", "Binary", "0b"],
      ["oct", "Octal", "0o"],
      ["dec", "Decimal", ""],
      ["hex", "Hexadecimal", "0x"],
    ]) {
      const [r, v] = row(label, prefix);
      rows[key] = v;
      list.append(r);
    }
    nibbles = el("div", "nibbles");
    root.append(list, el("h3", "", "Each hex digit is one nibble (4 bits)"), nibbles);
  },
  update({ value }) {
    rows.bin.textContent = `${bin(value >> 4, 4)} ${bin(value & 15, 4)}`;
    rows.oct.textContent = oct(value);
    rows.dec.textContent = String(value);
    rows.hex.textContent = hex(value);
    nibbles.innerHTML = "";
    for (const n of [value >> 4, value & 15]) {
      const box = el("div", "nibble");
      box.append(el("div", "nibble-bits", bin(n, 4)), el("div", "nibble-arrow", "↓"), el("div", "nibble-hex", hex(n, 1)), el("div", "nibble-dec", `(${n})`));
      nibbles.append(box);
    }
  },
};
