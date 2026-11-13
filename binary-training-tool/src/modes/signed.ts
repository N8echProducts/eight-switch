import { el, signed8, type Mode } from "./mode";

let cells: HTMLElement[] = [];
let unsignedOut: HTMLElement;
let signedOut: HTMLElement;
let explain: HTMLElement;

export const signed: Mode = {
  id: "signed",
  label: "Two's complement",
  mount(root) {
    const grid = el("div", "place-grid");
    cells = [];
    for (let bit = 7; bit >= 0; bit--) {
      const weight = bit === 7 ? -128 : 1 << bit;
      const cell = el("div", bit === 7 ? "place-cell sign" : "place-cell");
      cell.append(el("div", "place-weight", String(weight)), el("div", "place-power", bit === 7 ? "sign" : `2^${bit}`), el("div", "place-bit"));
      grid.append(cell);
      cells.push(cell);
    }
    const compare = el("div", "compare");
    const u = el("div", "compare-box");
    unsignedOut = el("div", "compare-value");
    u.append(el("div", "compare-label", "Unsigned (uint8)"), unsignedOut);
    const s = el("div", "compare-box");
    signedOut = el("div", "compare-value");
    s.append(el("div", "compare-label", "Signed (int8)"), signedOut);
    compare.append(u, s);
    explain = el("p", "explain");
    root.append(grid, compare, explain);
  },
  update({ value }) {
    cells.forEach((cell, i) => {
      const on = (value >> (7 - i)) & 1;
      cell.classList.toggle("on", on === 1);
      cell.querySelector(".place-bit")!.textContent = String(on);
    });
    const s = signed8(value);
    unsignedOut.textContent = String(value);
    signedOut.textContent = String(s);
    if (s >= 0) {
      explain.textContent = "Left-most bit is 0, so the number is positive and both readings agree.";
    } else {
      const magnitude = ((~value + 1) & 0xff) || 128;
      explain.textContent = `Left-most bit is 1, so the number is negative: −128 + ${value - 128} = ${s}. To negate it, flip every bit and add 1 → ${magnitude}.`;
    }
  },
};
