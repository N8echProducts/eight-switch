# Eight-Switch User Manual
Thank you for buying an eight-switch, the most powerful usb input device known to man (citation needed) 
Your new device is capable of representing any unicode keypoint with nothing but binary inputs!

## Basic usage (ascii)
The eight-switch will work on nearly any device that supports a usb keyboard (Give it a shot on your cell-phone if you want)
To start typing like an absolute computer master, you just plug it in and start flipping bits
1. Plug in the usb connector to your device and your computer
1. select the bits you'd like to be on and off by pressing the keys (keyboard version) or flipping the switches (toggle version)
## Compatibility
The eight-switch works on Macos, Windows, and many Linux computers.

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
Windows requires a small change to the registry before unicode values can be entered directly from a USB device.
1. Open the Registry Editor. Press Windows Key + R, type regedit, and press Enter.
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
When your Eight-Switch starts up, you will need to have the keys/toggles in a certain state to activate the specific mode for your operating system to properly handle direct unicode input
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
