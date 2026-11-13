# Eight-Switch QMK firmware (macropad mode)

Alternate firmware that turns the **KB DIY edition** of the Eight-Switch into a regular [QMK](https://qmk.fm) macropad: nine fully remappable keys (the eight keyswitches plus the send button) with layers, macros, media keys and so on. It does **not** do the binary/UTF-8 input that the stock firmware does. To go back, flash the stock `eight-switch.uf2` again; the RP2040 can always be recovered with the BOOTSEL button.

> Toggle edition: this firmware isn't meant for it. A toggle left "on" acts like a key held down forever.

## Prebuilt files

Every release on the [Releases page](https://github.com/npmaile/eight-switch/releases) includes:

| File | What it is |
|---|---|
| `eight-switch-qmk-default.uf2` | Fixed keymap (1–8, and Q–I while holding Send). Change it by editing the source and rebuilding. |
| `eight-switch-qmk-via.uf2` | Same keymap, but remappable live from a browser with [VIA](https://usevia.app). |
| `eight_switch.json` | VIA definition file (see below). |

Flash it the same way as the stock firmware: hold the "flash firmware" button while plugging in, then drag the `.uf2` onto the drive that appears.

### Remapping with VIA

1. Flash `eight-switch-qmk-via.uf2`.
2. Open https://usevia.app in a Chromium-based browser.
3. Settings → turn on **Show Design tab**.
4. Design tab → **Load** → pick `eight_switch.json`.
5. Configure tab → authorize the device → click a key and choose its new function. Changes save on the device.

## Default keymap

```
Base:            [1] [2] [3] [4] [5] [6] [7] [8]
                                 [ Send = hold for Fn ]

Fn (hold Send):   [Q] [W] [E] [R] [T] [Y] [U] [I]
```

The LED above a key lights while you hold it. While Send is held, every LED lights except the ones above keys you're pressing.

## Ways into the bootloader

* Hold the "flash firmware" button while plugging in
* Hold the leftmost key while plugging in

## Building it yourself

You need `git`, `make`, Python 3 with the `qmk` package (`pip install qmk`), and an ARM toolchain (`brew install arm-none-eabi-gcc` on macOS, or use the `ghcr.io/qmk/qmk_cli` Docker image).

```bash
./build.sh            # builds default and via keymaps into build/
./build.sh mykeymap   # builds keyboards/n8ech/eight_switch/keymaps/mykeymap
```

On its first run, `build.sh` clones a pinned QMK release into `.qmk_firmware/`. That's about 1 GB with submodules, and it's gitignored. To use an existing checkout, set `QMK_HOME=/path/to/qmk_firmware`.

To make your own layout, copy `keyboards/n8ech/eight_switch/keymaps/default` to a new folder, edit `keymap.c`, and run `./build.sh <folder name>`.

### Layout of this folder

```
qmk-firmware/
├── build.sh                         # copies the board into QMK and compiles
├── via/eight_switch.json            # VIA definition
└── keyboards/n8ech/eight_switch/
    ├── keyboard.json                # MCU, USB IDs, pin map, layout
    ├── eight_switch.c/.h            # LED handling + startup animation
    └── keymaps/{default,via}/
```

### Hardware notes

| Function | Pins |
|---|---|
| Keys, left → right | GP0 … GP7 (active low, internal pull-up) |
| Send button | GP8 (active low) |
| LEDs, left → right | GP9 … GP16 (active high) |
| Edition strap | GP17 (low = KB DIY edition) |

Keymaps can take over the LEDs by defining `bool eight_switch_leds_user(uint8_t pressed)` and calling `eight_switch_set_leds(mask)` (bit 0 = leftmost LED). Return `false` to stop the default "light what's held" behaviour. The default keymap uses this for its Fn indicator.

The USB ID is currently the QMK placeholder `FEED:0008`. A free open-hardware PID from [pid.codes](https://pid.codes) would be better.
