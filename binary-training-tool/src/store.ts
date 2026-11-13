// Single source of truth for the switch byte, fed by the device or the on-screen switches.

export type Source = "device" | "screen";

export interface State {
  value: number;
  button: boolean;
  source: Source;
}

type Listener = (state: State, prev: State) => void;

let state: State = { value: 0, button: false, source: "screen" };
const listeners = new Set<Listener>();
const pressListeners = new Set<() => void>();

export function getState(): State {
  return state;
}

export function setState(patch: Partial<State>): void {
  const prev = state;
  state = { ...state, ...patch, value: (patch.value ?? state.value) & 0xff };
  if (prev.value === state.value && prev.button === state.button && prev.source === state.source) return;
  listeners.forEach((l) => l(state, prev));
  if (state.button && !prev.button) pressListeners.forEach((l) => l());
}

export function subscribe(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Fires on the rising edge of the send button (device) or Enter (keyboard). */
export function onSendPress(listener: () => void): () => void {
  pressListeners.add(listener);
  return () => pressListeners.delete(listener);
}

export function toggleBit(bit: number): void {
  setState({ value: state.value ^ (1 << bit), source: "screen" });
}

export function pressSend(): void {
  setState({ button: true });
  setState({ button: false });
}
