# Eight-Switch (QMK)

QMK macropad firmware for the KB DIY edition of the Eight-Switch: eight keyswitches and a "send" button, with one LED above each key.

* Maintainer: npmaile
* Hardware: RP2040, direct-wired keys
* Source: https://github.com/npmaile/eight-switch

Build:

    qmk compile -kb n8ech/eight_switch -km default

## Entering the bootloader

* Hold the "flash firmware" (BOOTSEL) button while plugging in, **or**
* Hold the leftmost key while plugging in (Bootmagic)

## Pinout

| Function       | Pins                           |
|----------------|--------------------------------|
| Keys, L → R    | GP0 … GP7 (active low)         |
| Send button    | GP8 (active low)               |
| LEDs, L → R    | GP9 … GP16 (active high)       |
| Edition strap  | GP17 (low = KB DIY edition)    |
