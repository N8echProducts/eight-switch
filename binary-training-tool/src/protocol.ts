// USB protocol for the Eight-Switch binary training profile. See PROTOCOL.md.

export const USB_VENDOR_ID = 0x16c0;
export const VENDOR_USAGE_PAGE = 0xff00;

export const MSG_STATE = 0x01;
export const MSG_REQUEST_STATE = 0x02;

export const BUTTON_SEND = 0x01;

export interface SwitchReport {
  /** Bit 7 = left-most switch, bit 0 = right-most. */
  switches: number;
  /** Send button currently held. */
  button: boolean;
}

export function parseReport(data: DataView): SwitchReport | null {
  if (data.byteLength < 2 || data.getUint8(0) !== MSG_STATE) return null;
  const flags = data.byteLength > 2 ? data.getUint8(2) : 0;
  return { switches: data.getUint8(1), button: (flags & BUTTON_SEND) !== 0 };
}

/** Builds an output report padded to the size the device declares. */
export function requestStateReport(device: HIDDevice): Uint8Array<ArrayBuffer> {
  const out = device.collections
    .flatMap((c) => c.outputReports ?? [])
    .find((r) => (r.reportId ?? 0) === 0);
  const bytes =
    out?.items?.reduce((n, item) => n + ((item.reportSize ?? 0) * (item.reportCount ?? 0)) / 8, 0) ?? 1;
  const report = new Uint8Array(Math.max(1, bytes));
  report[0] = MSG_REQUEST_STATE;
  return report;
}
