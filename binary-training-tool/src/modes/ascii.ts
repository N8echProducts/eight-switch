import { charLabel, el, hex, type Mode } from "./mode";

let glyph: HTMLElement;
let name: HTMLElement;
let strip: HTMLElement;
let typed: HTMLElement;
let buffer = "";

export const ascii: Mode = {
  id: "ascii",
  label: "ASCII",
  mount(root) {
    glyph = el("div", "ascii-glyph");
    name = el("div", "ascii-name");
    strip = el("div", "ascii-strip");
    typed = el("div", "typed");
    const clear = el("button", "ghost", "Clear");
    clear.addEventListener("click", () => {
      buffer = "";
      typed.textContent = "";
    });
    const typedWrap = el("div", "typed-wrap");
    typedWrap.append(el("div", "compare-label", "Press send to append the character"), typed, clear);
    root.append(glyph, name, strip, typedWrap);
    typed.textContent = buffer;
  },
  update({ value }) {
    strip.innerHTML = "";
    if (value > 127) {
      glyph.textContent = "✕";
      glyph.classList.remove("small");
      glyph.classList.add("invalid");
      name.textContent = `${value} · 0x${hex(value)} · not ASCII. ASCII is 7 bits (0–127), so the left-most switch must be off.`;
      return;
    }
    const c = charLabel(value);
    glyph.textContent = c.glyph;
    glyph.classList.remove("invalid");
    glyph.classList.toggle("small", c.glyph.length > 1);
    name.textContent = `${value} · 0x${hex(value)} · ${c.name}`;
    for (let v = Math.max(0, value - 4); v <= Math.min(127, value + 4); v++) {
      const cell = el("div", v === value ? "strip-cell current" : "strip-cell");
      cell.append(el("div", "strip-glyph", charLabel(v).glyph), el("div", "strip-code", String(v)));
      strip.append(cell);
    }
  },
  onSend({ value }) {
    if (value > 127) return;
    if (value === 8) buffer = buffer.slice(0, -1);
    else if (value === 10 || value === 13) buffer += "\n";
    else if (value >= 32 && value !== 127) buffer += String.fromCharCode(value);
    typed.textContent = buffer;
  },
};
