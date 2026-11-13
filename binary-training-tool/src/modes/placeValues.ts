import { el, type Mode } from "./mode";

let cells: HTMLElement[] = [];
let sum: HTMLElement;

export const placeValues: Mode = {
  id: "place",
  label: "Place values",
  mount(root) {
    const grid = el("div", "place-grid");
    cells = [];
    for (let bit = 7; bit >= 0; bit--) {
      const cell = el("div", "place-cell");
      cell.append(el("div", "place-weight", String(1 << bit)), el("div", "place-power", `2^${bit}`), el("div", "place-bit"));
      grid.append(cell);
      cells.push(cell);
    }
    sum = el("div", "sum-line");
    root.append(grid, sum);
  },
  update({ value }) {
    const terms: string[] = [];
    cells.forEach((cell, i) => {
      const bit = 7 - i;
      const on = (value >> bit) & 1;
      cell.classList.toggle("on", on === 1);
      cell.querySelector(".place-bit")!.textContent = String(on);
      if (on) terms.push(String(1 << bit));
    });
    sum.innerHTML = "";
    sum.append(
      el("span", "terms", terms.length ? terms.join(" + ") : "0"),
      el("span", "equals", " = "),
      el("span", "total", String(value)),
    );
  },
};
