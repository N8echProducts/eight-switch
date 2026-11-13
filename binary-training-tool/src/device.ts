import { USB_VENDOR_ID, VENDOR_USAGE_PAGE, parseReport, requestStateReport } from "./protocol";
import { setState } from "./store";

export type Status = "unsupported" | "disconnected" | "connected";

const filters: HIDDeviceFilter[] = [{ vendorId: USB_VENDOR_ID, usagePage: VENDOR_USAGE_PAGE }];

let current: HIDDevice | null = null;
let statusListener: (s: Status, name?: string) => void = () => {};

export const hidSupported = typeof navigator !== "undefined" && "hid" in navigator;

function isTrainingInterface(device: HIDDevice): boolean {
  return (
    device.vendorId === USB_VENDOR_ID &&
    device.collections.some((c) => c.usagePage === VENDOR_USAGE_PAGE)
  );
}

function onInputReport(event: HIDInputReportEvent): void {
  const report = parseReport(event.data);
  if (report) setState({ value: report.switches, button: report.button, source: "device" });
}

async function open(device: HIDDevice): Promise<void> {
  if (!device.opened) await device.open();
  current = device;
  device.addEventListener("inputreport", onInputReport);
  statusListener("connected", device.productName);
  try {
    await device.sendReport(0, requestStateReport(device));
  } catch {
    // Profile may not implement the state request; the next input report will catch us up.
  }
}

export function onStatus(listener: (s: Status, name?: string) => void): void {
  statusListener = listener;
  listener(hidSupported ? "disconnected" : "unsupported");
}

/** Must be called from a user gesture (click). */
export async function requestDevice(): Promise<void> {
  const devices = await navigator.hid.requestDevice({ filters });
  const device = devices.find(isTrainingInterface);
  if (device) await open(device);
}

export async function disconnect(): Promise<void> {
  if (!current) return;
  current.removeEventListener("inputreport", onInputReport);
  await current.close();
  current = null;
  setState({ source: "screen", button: false });
  statusListener("disconnected");
}

/** Reconnects to previously-granted devices and follows plug/unplug events. */
export async function initAutoConnect(): Promise<void> {
  if (!hidSupported) return;
  navigator.hid.addEventListener("connect", (e) => {
    if (!current && isTrainingInterface(e.device)) void open(e.device);
  });
  navigator.hid.addEventListener("disconnect", (e) => {
    if (e.device === current) {
      current = null;
      setState({ source: "screen", button: false });
      statusListener("disconnected");
    }
  });
  const granted = (await navigator.hid.getDevices()).find(isTrainingInterface);
  if (granted) await open(granted);
}
