# eight-switch
Open Source Utf-8 direct keyboard

## Using
ASCII inputs work on more-or-less any device that takes a usb keyboard out of the box.
1. Plug in the eight-switch
1. Select a printable ascii code using the toggle switches
1. Press the button

## UTF-8
Utf-8 inputs are currently supported on the following systems, but some only after enabling unicode direct hex input on the computer.

### MacOS
1. Enable unicode direct input (only needed once)
    1. Open "System Settings"
    2. Open the "Keyboard" menu
    3. Open the "Input Sources" menu
    4. Add "Unicode Hex Input" from the "Others" menu
2. Flip all keys on your eight-switch off
3. Plug in the eight-switch

### Windows
1. Enable Unicode direct input (only needed once)
    1. Press Win + R to open "RUN"
    2. Type regedit and hit Enter to open the Registry Editor.
    3. Navigate to HKEY_CURRENT_USER\Control Panel\Input Method.
    4. Right-click the empty space in the right pane,
    5. select New > String Value, and name it EnableHexNumpad.Double-click EnableHexNumpad,
    6. set the Value data to 1,
    7. click OK.
    8. Restart your computer
1. Flip all keys on your eight-switch off except for the right-most switch (00000001)
2. Plug in the eight-switch

## Firmware Updates
Firmware can be found on the ["Releases" page](https://github.com/npmaile/eight-switch/releases) in this repository
### Flashing new updates
the eight-switch is based on the Raspberry pi rp2040 chip, which makes flashing the firmware trivial.
1. Hold down the "flash firmware" button (the button that isn't the push-button to send sequences to the computer) while plugging the device into your computer.
2. drag the .uf2 file from your computer to the drive the device is presenting itself as
3. wait for the device to restart

## AI disclosure
The product and firmware was all hand-designed/coded with only chatbot style glorified Google Search patterns to/from Claude code.
The utf-8 booklet was nearly entirely vibecoded beyond the initial implementation.
The CI setup was painfully ai assisted and I regret not doing it myself due to the hassle the AI brought
All documentation is written by hand with AI used as glorified Google search.

My general policy: I strive to put out work at the highest quality, with an emphasis on the things people will care about and interact with the most. I allow AI in my workflows only in places where it would not be feasible to do it by hand, and where I can carefully monitor and guage the output to be to my standards.
