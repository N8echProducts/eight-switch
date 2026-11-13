# Binary training tool USB protocol

The web app talks to the Eight-Switch over **WebHID**. The training-mode
device profile must expose a **vendor-defined HID interface** in addition to
(or instead of) the normal keyboard interface. Browsers refuse to let web pages
open keyboard/mouse collections, so the switch state cannot be read from the
regular keyboard reports.

All the constants below live in [`src/protocol.ts`](src/protocol.ts). If you
change anything in the firmware, change it there too.

## Device matching

| Field         | Value                                  |
|---------------|----------------------------------------|
| Vendor ID     | `0x16C0` (same as the normal firmware) |
| Product ID    | any (the app does not filter on it)    |
| Usage page    | `0xFF00` (vendor defined)              |
| Usage         | `0x01`                                 |

## Device → host: state report (input report)

Sent whenever the switch byte or the send button changes, and in response to a
state request (below). Sending it periodically as well (e.g. every 250 ms) is
harmless.

| Byte | Meaning                                                       |
|------|---------------------------------------------------------------|
| 0    | Message type, `0x01` = state                                  |
| 1    | Switch byte. Bit 7 = left-most switch, bit 0 = right-most     |
| 2    | Button flags. Bit 0 = send button held, other bits reserved 0 |

The report has no report ID. Pad to the report size declared in the descriptor
(8 bytes is suggested); trailing bytes are ignored.

The "switch byte" should be the same value the normal firmware gets from
`keys.get_state()`, so the KB editions report their latched/toggled state, not
raw key presses.

## Host → device: state request (output report)

| Byte | Meaning                            |
|------|------------------------------------|
| 0    | Message type, `0x02` = request state |

The app sends this right after opening the device so the display is correct
before anything is flipped. Supporting it is optional; without it the app
shows the state from the first report it receives.

## Example report descriptor

Vendor-defined, 8 bytes in, 8 bytes out, no report IDs:

```
0x06, 0x00, 0xFF,  // Usage Page (Vendor Defined 0xFF00)
0x09, 0x01,        // Usage (0x01)
0xA1, 0x01,        // Collection (Application)
0x09, 0x02,        //   Usage (0x02)
0x15, 0x00,        //   Logical Minimum (0)
0x26, 0xFF, 0x00,  //   Logical Maximum (255)
0x75, 0x08,        //   Report Size (8)
0x95, 0x08,        //   Report Count (8)
0x81, 0x02,        //   Input (Data, Var, Abs)
0x09, 0x03,        //   Usage (0x03)
0x95, 0x08,        //   Report Count (8)
0x91, 0x02,        //   Output (Data, Var, Abs)
0xC0               // End Collection
```

With `usbd-hid` this can be passed to `HIDClass::new` as a raw `&[u8]`
descriptor.

## Browser support

WebHID works in Chrome, Edge, Opera and other Chromium browsers on desktop.
Firefox and Safari do not support it; the app still works there with the
on-screen switches. Pages must be served from `https://` or `localhost`.

On Linux, the user needs read/write access to the hidraw node, e.g. a udev
rule:

```
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="16c0", MODE="0660", TAG+="uaccess"
```
