import { bin, el, hex, oct, signed8, type Mode } from "./mode";
import { getState, type State } from "../store";

type Kind = "dec" | "hex" | "oct" | "char" | "signed";

const KINDS: { kind: Kind; label: string }[] = [
  { kind: "dec", label: "Decimal" },
  { kind: "hex", label: "Hex" },
  { kind: "oct", label: "Octal" },
  { kind: "char", label: "ASCII" },
  { kind: "signed", label: "Negative (int8)" },
];

interface Question {
  kind: Kind;
  answer: number;
  prompt: string;
  hint: string;
}

const STORAGE_KEY = "eight-switch-binary-trainer";
const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") as { kinds?: Kind[]; auto?: boolean; best?: number };

let enabled = new Set<Kind>(saved.kinds?.length ? saved.kinds : ["dec", "hex"]);
let autoCheck = saved.auto ?? true;
let best = saved.best ?? 0;
let score = 0;
let attempts = 0;
let streak = 0;
let question: Question | null = null;
let locked = false;
let lastValue = 0;

let promptEl: HTMLElement;
let hintEl: HTMLElement;
let feedbackEl: HTMLElement;
let statsEl: HTMLElement;

function persist() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({ kinds: [...enabled], auto: autoCheck, best }));
}

function randomInt(lo: number, hi: number) {
  return lo + Math.floor(Math.random() * (hi - lo + 1));
}

function makeQuestion(): Question {
  const kinds = [...enabled];
  const kind = kinds[randomInt(0, kinds.length - 1)];
  let answer = randomInt(1, 255);
  // Don't ask for what's already on the switches.
  while (answer === lastValue) answer = randomInt(1, 255);
  switch (kind) {
    case "dec":
      return { kind, answer, prompt: String(answer), hint: "decimal" };
    case "hex":
      return { kind, answer, prompt: `0x${hex(answer)}`, hint: "hexadecimal" };
    case "oct":
      return { kind, answer, prompt: `0o${oct(answer)}`, hint: "octal" };
    case "char": {
      answer = randomInt(33, 126);
      return { kind, answer, prompt: String.fromCharCode(answer), hint: "ASCII character" };
    }
    case "signed": {
      answer = randomInt(128, 255);
      return { kind, answer, prompt: String(signed8(answer)), hint: "signed 8-bit (two's complement)" };
    }
  }
}

function next() {
  question = makeQuestion();
  locked = false;
  promptEl.textContent = question.prompt;
  hintEl.textContent = `Set the switches to this ${question.hint} value`;
  feedbackEl.className = "feedback";
  feedbackEl.textContent = "";
}

function renderStats() {
  statsEl.textContent = `Score ${score}/${attempts} · Streak ${streak} · Best ${best}`;
}

function check(value: number, explicit: boolean) {
  if (!question || locked) return;
  const correct = value === question.answer;
  if (!correct && !explicit) return;
  attempts++;
  if (correct) {
    score++;
    streak++;
    if (streak > best) {
      best = streak;
      persist();
    }
    feedbackEl.className = "feedback good";
    feedbackEl.textContent = `Correct! ${bin(value)}`;
  } else {
    streak = 0;
    feedbackEl.className = "feedback bad";
    feedbackEl.textContent = `Not quite — you have ${bin(value)}, the answer is ${bin(question.answer)}`;
  }
  renderStats();
  locked = true;
  setTimeout(next, correct ? 900 : 2500);
}

export const challenge: Mode = {
  id: "challenge",
  label: "Challenge",
  mount(root) {
    const options = el("div", "options");
    for (const { kind, label } of KINDS) {
      const lab = el("label", "chip");
      const box = el("input");
      box.type = "checkbox";
      box.checked = enabled.has(kind);
      box.addEventListener("change", () => {
        if (box.checked) enabled.add(kind);
        else if (enabled.size > 1) enabled.delete(kind);
        else box.checked = true;
        persist();
        next();
      });
      lab.append(box, document.createTextNode(label));
      options.append(lab);
    }
    const autoLab = el("label", "chip auto");
    const autoBox = el("input");
    autoBox.type = "checkbox";
    autoBox.checked = autoCheck;
    autoBox.addEventListener("change", () => {
      autoCheck = autoBox.checked;
      persist();
    });
    autoLab.append(autoBox, document.createTextNode("Auto-check (otherwise press send)"));
    options.append(autoLab);

    promptEl = el("div", "challenge-prompt");
    hintEl = el("div", "challenge-hint");
    feedbackEl = el("div", "feedback");
    statsEl = el("div", "stats");
    const skip = el("button", "ghost", "Skip");
    skip.addEventListener("click", () => {
      streak = 0;
      renderStats();
      next();
    });
    root.append(options, hintEl, promptEl, feedbackEl, statsEl, skip);
    renderStats();
    lastValue = getState().value;
    next();
  },
  update(state: State) {
    lastValue = state.value;
    if (autoCheck) check(state.value, false);
  },
  onSend(state: State) {
    check(state.value, true);
  },
};
