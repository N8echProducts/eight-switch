# Eight-Switch User Manual
Thank you for buying an eight-switch, the most powerful USB input device known to man (citation needed) 
Your new device is capable of representing any Unicode keypoint with nothing but binary inputs!

## Basic usage (ASCII)
The eight-switch will work on nearly any device that supports a USB keyboard (Give it a shot on your cell-phone if you want)
To start typing like an absolute computer master, you just plug it in and start flipping bits
1. Plug in the USB connector to your device and your computer
1. select the bits you'd like to be on and off by pressing the keys (keyboard version) or flipping the switches (toggle version)
## Compatibility
The eight-switch works on MacOS, Windows, and many Linux computers.

## OS configuration (setup)
Most operating systems do not allow for native input of unicode/utf-8 values right out of the box, and require a little set up prior to use.
### MacOS
You must enable Unicode Hex Input mode before you can input utf-8 values directly
1. Open the Apple Menu in the top-left corner of your screen and select System Settings.
1. Click on Keyboard in the sidebar.
1. Under the Text Input section, click the Edit button next to Input Sources.
1. Click the + (add) button at the bottom of the list.
1. Scroll down to the bottom of the list, select Others, and choose Unicode Hex Input.
1. Click Add and exit Settings.
1. In your top bar, you should now have an icon that when clicked allows you to change between your normal input mode and the "U+" mode. The utf-8 inputs will work when your computer is in U+ mode.
### Windows
Windows requires a small change to the registry before Unicode values can be entered directly from a USB device.
1. Open the Registry Editor. Press Windows Key + R, type `regedit`, and press Enter.
2. Navigate to the following registry path:text `HKEY_CURRENT_USER\Control Panel\Input Method`
3. Create a new string value:
	• Right-click an empty space in the right pane (or go to Edit > New > String Value).
	• Name the new value: `EnableHexNumpad`.
4. Set the value data:
	• Double-click `EnableHexNumpad` and change its Value data to 1.
	• Click OK and close the Registry Editor.
5. Apply the changes by logging off and log back in to your Windows account (or restart your computer) for the setting to take effect.
### Linux
The settings differ between different desktop environments. Please consult your user manual.

## OS configuration (usage)
When your Eight-Switch starts up, you will need to have the keys/toggles in a certain state to activate the specific mode for your operating system to properly handle direct Unicode input
### MacOS
When plugging in the device, do not have any of the keys pressed (Keyboard version) or any of the toggles in the "On" position (Toggle version)
### Windows
When plugging in the device, have the right-most of the eight bits set to on.
- Keyboard Version: press down the eighth key as you plug in the device
- Toggle Version: Have the eighth switch turned on as you plug in the device
### Linux (works in many desktop environments)
When plugging in the device, have the right-most of the eight bits set to on.
- Keyboard Version: press down the eighth key and the seventh key as you plug in the device
- Toggle Version: Have the eighth switch and the seventh switch turned on as you plug in the device

# How to use the eight-switch (once you've set up your computer)
## Orienting your device
The eight-switch is meant to be oriented with the submit button on the right side of the device, and the eight inputs (keys or switches) to the left.
## Inputting a simple ASCII character
The capital letter `A` can be input by setting your eight-switch bits to `off-on-off-off-off-off-off-on`
- Toggle edition: The first switch is down, the second is up, the next 5 are down, and the last is up
- Keyboard edition: The first key has it's light off, the second key's light is on, the next 5 are off, and the last is on
I will represent this pattern as `01000001` for the remainder of this document.

There are other notable ASCII values that can be input. Interesting ones are noted below
- lowercase letters begin at `01100001`
- numbers begin at `00110000`
- backspace is `01111111`
The remainder of the ASCII values can be found by doing a google search for "ASCII table" or by experimenting with your device
## Inputting a UTF-8 value
UTF-8 is an encoding standard that can represent nearly any form of written human communication and currently has over 170,000 "glyphs" including: all of ASCII, Chinese, Japanese characters, Emoji, dice, various symbols, and many, many more. 
The UTF-8 values can be input by inputting the binary values eight bits at a time, using the submit button on your device between sets of 8 bits. The device will indicate you've input an illegal value (and thus need to restart your attempt) by flashing twice at you, and indicate that it's waiting for more sets of 8 bits by "pulsing" the light.

Here are a few interesting sequences:
- é `11000011 10101001`
- 龙 `11101001 10111110 10011001`
- 😊 `11110000 10011111 10011000 10001010`

For a full list of sequences, please look at the n8ech website (https://n8ech.com) for the Unicode Codebook, which has a comprehensive list of all possible values. 

# Updating your Eight-Switch
It should work out of the box, but if your eight-switch needs an update (if your operating system disables the current update mode) or if you attempt to flash alternative firmware to your device and want to go back, a set of official firmware can be found on the github "releases" page for n8ech
1. Navigate to https://github.com/N8echProducts/eight-switch
2. Click on the "releases" link
3. Download the most recent ".uf2" file from the page
4. Plug in your eight-switch while holding the small button (on DIY kit versions, it will be the button on the raspberry pi pico module. On pre-assembled models, it will be accessible by pressing a paperclip or sim-ejector tool or similar through a small hole on the bottom of the device)
5. Drag the .UF2 file on-to the RPI-RP2 removable device the eight-switch is presenting itself as.
6. Wait for the update to finish

