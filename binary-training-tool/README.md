# Eight-Switch Binary Trainer

A web app that turns the Eight-Switch into a binary training tool. With the
training device profile flashed, the app reads the switch byte over USB and
shows it in different representations.

Note: like `qmk-firmware/`, this directory is vibecoded.

## Modes

- **Place values** – each switch with its weight (128 … 1) and the sum.
- **Bases** – binary, octal, decimal and hex, with the hex nibble split.
- **Two's complement** – unsigned vs. signed (int8) reading of the byte.
- **ASCII** – the character for the byte; the send button appends it to a line of text.
- **Challenge** – a decimal / hex / octal / ASCII / negative value is shown and
  you set the switches to match. Checks automatically, or on the send button.

## Running

```sh
npm install
npm run dev      # http://localhost:5173
npm run build    # static site in dist/
```

Connecting to the device needs WebHID (Chrome, Edge or another Chromium
browser) and a page served from `localhost` or `https://`. Without a device the
on-screen switches are clickable, and keys `1`–`8` flip them (`0` clears,
`Enter` is the send button).

## Device profile

The USB messages the training profile must send are described in
[PROTOCOL.md](PROTOCOL.md). The constants are in
[`src/protocol.ts`](src/protocol.ts).

## Layout

```
src/
  protocol.ts   USB report format (keep in sync with the firmware)
  device.ts     WebHID connect / auto-reconnect / report handling
  store.ts      current switch byte + send button, from device or screen
  main.ts       switch bank, tabs, keyboard fallback
  modes/        one file per display mode
```

Adding a mode: create a `Mode` in `src/modes/` and add it to `MODES` in
`src/main.ts`.
