#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use embedded_hal::{digital::InputPin, pwm::SetDutyCycle};
use libm;
// The macro for our start-up function
use rp_pico::{
    entry,
    hal::{
        self, Clock,
        gpio::{DynPinId, FunctionSio, Pin, PullUp, SioInput},
        multicore::{Multicore, Stack},
        pac,
        sio::Sio,
    },
    pac::{generic::RawReg, timer},
};

use panic_halt as _;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::{
    descriptor::{
        KeyboardReport,
        KeyboardUsage::{self, *},
        generator_prelude::*,
    },
    hid_class::HIDClass,
};

mod debounce;

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Human Interface Device Driver (shared with the interrupt).
static mut USB_HID: Option<HIDClass<hal::usb::UsbBus>> = None;

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn core1_key_tracker_main(sys_freq: u32, timer: &hal::Timer, keys: &mut CherryBoard) -> ! {
    let pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq);

    loop {
        // look at keys all the time and send them back to core 0 if they are asked for
        match sio.fifo.read() {
            // core 0 requested state. Send it back
            Some(req_type) => match req_type {
                0 => {
                    let mut val: u8 = 0;
                    for x in 0..8 {
                        if keys.states[x] {
                            val |= 1 << x;
                        }
                    }
                    sio.fifo.write(val as u32);
                }
                1 => {
                    let mut val: u8 = 0;
                    for x in 0..8 {
                        if keys.keys[x].is_pressed_raw() {
                            val |= 1 << x;
                        }
                    }
                    sio.fifo.write(val as u32);
                }
                _ => {}
            },
            // core 0 didn't request state. Ignore
            None => {}
        }

        let now_ms = timer.get_counter().ticks() / 1000;
        for x in 0..8 {
            keys.keys[x].update(now_ms);
            if keys.keys[x].just_pressed() {
                keys.states[x] = !keys.states[x];
            }
        }
        // just to not spin the second core like crazy all day long
        delay.delay_ms(2);
    }
}

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sys_freq = clocks.system_clock.freq().to_Hz();

    let mut sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let usb_clock = clocks.usb_clock;
    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        usb_clock,
        true,
        &mut pac.RESETS,
    ));
    unsafe {
        // Note (safety): This is safe as interrupts haven't been started yet
        USB_BUS = Some(usb_bus);
    }

    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    let usb_hid = HIDClass::new(bus_ref, KeyboardReport::desc(), 60);
    unsafe {
        USB_HID = Some(usb_hid);
    }

    let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27dc))
        .strings(&[StringDescriptors::default()
            .manufacturer("N8ech")
            .product("8switch")
            .serial_number("TEST")])
        .unwrap()
        .device_class(0)
        .max_packet_size_0(64)
        .unwrap()
        .build();
    unsafe {
        USB_DEVICE = Some(usb_dev);
    }

    let mut switch_or_keys_pin = pins.gpio17.into_pull_up_input();

    let mut keys: AnySwitchBoard = match switch_or_keys_pin.is_low().unwrap() {
        // switch edition
        false => {
            let switches = [
                pins.gpio0.into_pull_up_input().into_dyn_pin(),
                pins.gpio1.into_pull_up_input().into_dyn_pin(),
                pins.gpio2.into_pull_up_input().into_dyn_pin(),
                pins.gpio3.into_pull_up_input().into_dyn_pin(),
                pins.gpio4.into_pull_up_input().into_dyn_pin(),
                pins.gpio5.into_pull_up_input().into_dyn_pin(),
                pins.gpio6.into_pull_up_input().into_dyn_pin(),
                pins.gpio7.into_pull_up_input().into_dyn_pin(),
            ];
            AnySwitchBoard::Toggle(ToggleBoard::new(switches))
        }
        // keyboard edition
        true => {
            // set up input pins
            let keyswitches = [
                debounce::DebouncedButton::new(pins.gpio7.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio6.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio5.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio4.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio3.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio2.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio1.into_pull_up_input().into_dyn_pin()),
                debounce::DebouncedButton::new(pins.gpio0.into_pull_up_input().into_dyn_pin()),
            ];

            let mut keys = CherryBoard {
                keys: keyswitches,
                states: [false; 8],
            };
            let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
            let cores = mc.cores();
            let core1 = &mut cores[1];
            let _ = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
                core1_key_tracker_main(sys_freq, &timer, &mut keys)
            });
            AnySwitchBoard::Cherry(CherryBoardStateGetter::new(sio.fifo))
        }
    };

    // set up necessary pwm slices for lights
    let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let mut pwm4 = pwm_slices.pwm4;
    pwm4.set_ph_correct();
    pwm4.enable();
    let mut pwm5 = pwm_slices.pwm5;
    pwm5.set_ph_correct();
    pwm5.enable();
    let mut pwm6 = pwm_slices.pwm6;
    pwm6.set_ph_correct();
    pwm6.enable();
    let mut pwm7 = pwm_slices.pwm7;
    pwm7.set_ph_correct();
    pwm7.enable();
    let mut pwm0 = pwm_slices.pwm0;
    pwm0.set_ph_correct();
    pwm0.enable();

    // pwm for gpio 9
    let led1 = &mut pwm4.channel_b;
    led1.output_to(pins.gpio9);
    led1.set_duty_cycle_percent(0);
    // pwm for gpio 10
    let led2 = &mut pwm5.channel_a;
    led2.output_to(pins.gpio10);
    led2.set_duty_cycle_percent(0);
    // pwm for gpio 11
    let led3 = &mut pwm5.channel_b;
    led3.output_to(pins.gpio11);
    led3.set_duty_cycle_percent(0);
    // pwm for gpio 12
    let led4 = &mut pwm6.channel_a;
    led4.output_to(pins.gpio12);
    led4.set_duty_cycle_percent(0);
    // pwm for gpio 13
    let led5 = &mut pwm6.channel_b;
    led5.output_to(pins.gpio13);
    led5.set_duty_cycle_percent(0);
    // pwm for gpio 14
    let led6 = &mut pwm7.channel_a;
    led6.output_to(pins.gpio14);
    led6.set_duty_cycle_percent(0);
    // pwm for gpio 15
    let led7 = &mut pwm7.channel_b;
    led7.output_to(pins.gpio15);
    led7.set_duty_cycle_percent(0);
    // pwm for gpio 16
    let led8 = &mut pwm0.channel_a;
    led8.output_to(pins.gpio16);
    led8.set_duty_cycle_percent(0);

    let mut lights: [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8] =
        [led8, led7, led6, led5, led4, led3, led2, led1];
    let button = pins.gpio8.into_pull_up_input().into_dyn_pin();

    let os = match keys.get_state_raw() {
        1 => OsInputMode::Win,
        2 => OsInputMode::Lin,
        _ => OsInputMode::Mac,
    };

    let mut keyprocessor = Processor {
        bbuf: [0, 0, 0, 0],
        bbufp: 0,
        expected_chars: 0,
    };

    let mut btn = debounce::DebouncedButton::new(button);

    let now_ms = timer.get_counter().ticks() / 1000;
    let mut lm = LightManager {
        mode: LightMode::Startup(now_ms),
        default_brightness: 50,
    };

    let mut keybuf = KeyBuffer::default();

    loop {
        critical_section::with(|_| unsafe {
            let usb_dev = USB_DEVICE.as_mut().unwrap();
            let usb_hid = USB_HID.as_mut().unwrap();
            usb_dev.poll(&mut [usb_hid]);
        });

        let now_ms = timer.get_counter().ticks() / 1000;
        let switch_state: u8 = keys.get_state();
        lm.tick(switch_state, &mut lights, now_ms);
        btn.update(now_ms);
        if btn.just_pressed() {
            match keyprocessor.check_for_utf32val(switch_state) {
                Ok(Some(utf32_val)) => {
                    lm.mode = LightMode::Normal;
                    keybuf.reset();
                    if utf32_val <= 127 {
                        match ascii_to_key_usage(utf32_val as u8) {
                            None => {
                                lm.mode = LightMode::ErrBlink(now_ms);
                            }
                            Some(val) => {
                                keybuf.push(KbvalWithModKey {
                                    keycode: 0,
                                    modkey: val.modkey,
                                });
                                keybuf.push(val);
                            }
                        }
                    } else {
                        let (sequence, len) = os.u32_to_nibbles(utf32_val);
                        match len {
                            4 => {
                                for prefix_key in os.pre() {
                                    match prefix_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                                for x in 4..=7 {
                                    let usb_code =
                                        lookup_keycode_for_usb_hex_input(os, sequence[x]);
                                    // the key
                                    keybuf.push(KbvalWithModKey {
                                        keycode: usb_code as u8,
                                        modkey: os.mod_key(),
                                    });
                                    // release while holding mod
                                    keybuf.push(KbvalWithModKey {
                                        keycode: 0,
                                        modkey: os.mod_key(),
                                    });
                                }
                                for after_key in os.post() {
                                    match after_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                            }
                            8 => {
                                for prefix_key in os.pre() {
                                    match prefix_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                                for x in 0..=3 {
                                    let usb_code =
                                        lookup_keycode_for_usb_hex_input(os, sequence[x]);
                                    if usb_code as u8 != 0 {
                                        // the key
                                        keybuf.push(KbvalWithModKey {
                                            keycode: usb_code as u8,
                                            modkey: os.mod_key(),
                                        });
                                        // release while holding mod
                                        keybuf.push(KbvalWithModKey {
                                            keycode: 0,
                                            modkey: os.mod_key(),
                                        });
                                    }
                                }

                                for mid_key in os.mid() {
                                    match mid_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }

                                for x in 4..=7 {
                                    let usb_code =
                                        lookup_keycode_for_usb_hex_input(os, sequence[x]);
                                    if usb_code as u8 != 0 {
                                        // the key
                                        keybuf.push(KbvalWithModKey {
                                            keycode: usb_code as u8,
                                            modkey: os.mod_key(),
                                        });
                                        // release while holding mod
                                        keybuf.push(KbvalWithModKey {
                                            keycode: 0,
                                            modkey: os.mod_key(),
                                        });
                                    }
                                }
                                for after_key in os.post() {
                                    match after_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                            }
                            _ => {
                                for prefix_key in os.pre() {
                                    match prefix_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                                for x in 8 - len..=7 {
                                    let usb_code =
                                        lookup_keycode_for_usb_hex_input(os, sequence[x]);
                                    // the key
                                    keybuf.push(KbvalWithModKey {
                                        keycode: usb_code as u8,
                                        modkey: os.mod_key(),
                                    });
                                    // release while holding mod
                                    keybuf.push(KbvalWithModKey {
                                        keycode: 0,
                                        modkey: os.mod_key(),
                                    });
                                }
                                for after_key in os.post() {
                                    match after_key {
                                        Some(key) => keybuf.push(key),
                                        None => {}
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    lm.mode = LightMode::Pulsing(now_ms);
                }
                Err(_) => {
                    lm.mode = LightMode::ErrBlink(now_ms);
                }
            }
        }

        if keybuf.send_read < keybuf.send_write {
            let key = keybuf.send_buf[keybuf.send_read];
            match push_keys(KeyboardReport {
                modifier: key.modkey,
                reserved: 0,
                leds: 0,
                keycodes: [key.keycode, 0, 0, 0, 0, 0],
            }) {
                Ok(_) => {
                    keybuf.send_read += 1;
                }
                Err(UsbError::WouldBlock) => {}
                Err(_) => keybuf.send_read += 1,
            }
        } else {
            let _ = push_keys(KeyboardReport {
                modifier: 0,
                reserved: 0,
                leds: 0,
                keycodes: [0, 0, 0, 0, 0, 0],
            });
        }
    }
}

struct KeyBuffer {
    send_buf: [KbvalWithModKey; 32],
    send_read: usize,
    send_write: usize,
}

impl KeyBuffer {
    fn push(&mut self, val: KbvalWithModKey) {
        self.send_buf[self.send_write] = val;
        self.send_write += 1;
    }
    fn reset(&mut self) {
        self.send_read = 0;
        self.send_write = 0;
    }
}

impl Default for KeyBuffer {
    fn default() -> Self {
        return KeyBuffer {
            send_buf: core::array::from_fn(|_| KbvalWithModKey::default()),
            send_read: 0,
            send_write: 0,
        };
    }
}

#[derive(Copy, Clone)]
enum OsInputMode {
    Mac,
    Win,
    Lin,
}

impl OsInputMode {
    fn mod_key(&self) -> u8 {
        match self {
            OsInputMode::Mac => 0x04,
            OsInputMode::Win => 0x40,
            OsInputMode::Lin => 0x00,
        }
    }
    fn pre(&self) -> [Option<KbvalWithModKey>; 4] {
        return match &self {
            OsInputMode::Mac => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                None,
                None,
                None,
            ],
            OsInputMode::Win => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                Some(KbvalWithModKey {
                    keycode: KeyboardUsage::KeypadPlus as u8,
                    modkey: self.mod_key(),
                }),
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                None,
            ],
            OsInputMode::Lin => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: 0x03,
                }),
                Some(KbvalWithModKey {
                    keycode: 0x18,
                    modkey: 0x03,
                }),
                None,
                None,
            ],
        };
    }
    fn mid(&self) -> [Option<KbvalWithModKey>; 4] {
        return match &self {
            OsInputMode::Win => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: 0,
                }),
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                Some(KbvalWithModKey {
                    keycode: KeyboardUsage::KeypadPlus as u8,
                    modkey: self.mod_key(),
                }),
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
            ],
            _ => [None, None, None, None],
        };
    }
    fn post(&self) -> [Option<KbvalWithModKey>; 4] {
        return match &self {
            OsInputMode::Mac => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                None,
                None,
                None,
            ],
            OsInputMode::Win => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                None,
                None,
                None,
            ],
            OsInputMode::Lin => [
                Some(KbvalWithModKey {
                    keycode: 0,
                    modkey: self.mod_key(),
                }),
                Some(KbvalWithModKey {
                    keycode: 0x28,
                    modkey: self.mod_key(),
                }),
                None,
                None,
            ],
        };
    }
    fn u32_to_nibbles(self, val: u32) -> ([u8; 8], usize) {
        match &self {
            OsInputMode::Lin => u32_to_unicode_codepoint_bytes(val),
            _ => u32_to_utf16_bytes(val),
        }
    }
}

impl Default for OsInputMode {
    fn default() -> Self {
        return OsInputMode::Mac;
    }
}

fn u32_to_unicode_codepoint_bytes(val: u32) -> ([u8; 8], usize) {
    let mut buf: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut bytes = 8;
    for i in 0..8 {
        let nibble = (val >> ((7 - i) * 4)) & 0xf;
        buf[i] = nibble as u8;
    }
    for b in 0..8 {
        if buf[b] == 0 { bytes -= 1 } else { break }
    }

    (buf, bytes)
}

fn u32_to_utf16_bytes(mut val: u32) -> ([u8; 8], usize) {
    let bytes;
    match val {
        0x10_000.. => {
            let h = ((val - 0x10_000) / 0x400) + 0xD800;
            let l = ((val - 0x10_000) % 0x400) + 0xDC00;
            val = h << 16 | l;
            bytes = 8;
        }
        0..128 => bytes = 1,
        _ => {
            bytes = 4;
        }
    };

    let mut buf = [0u8; 8];
    for i in 0..8 {
        let nibble = (val >> ((7 - i) * 4)) & 0xf;
        buf[i] = nibble as u8;
    }
    (buf, bytes)
}

fn match_led_switch_states(
    switch_state: u8,
    lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8],
    duty_cycle: u8,
) {
    for i in 0..8 {
        match (switch_state >> i) & 1 {
            1 => {
                lights[i].set_duty_cycle_percent(duty_cycle);
            }
            0 => {
                lights[i].set_duty_cycle_fully_off();
            }
            // not possible
            _ => {}
        }
    }
}

fn lookup_keycode_for_usb_hex_input(os: OsInputMode, val: u8) -> KeyboardUsage {
    match os {
        OsInputMode::Mac | OsInputMode::Lin => {
            return match val {
                0x00 => KeyboardUsage::Keyboard0CloseParens,
                0x01 => KeyboardUsage::Keyboard1Exclamation,
                0x02 => KeyboardUsage::Keyboard2At,
                0x03 => KeyboardUsage::Keyboard3Hash,
                0x04 => KeyboardUsage::Keyboard4Dollar,
                0x05 => KeyboardUsage::Keyboard5Percent,
                0x06 => KeyboardUsage::Keyboard6Caret,
                0x07 => KeyboardUsage::Keyboard7Ampersand,
                0x08 => KeyboardUsage::Keyboard8Asterisk,
                0x09 => KeyboardUsage::Keyboard9OpenParens,
                0x0A => KeyboardUsage::KeyboardAa,
                0x0B => KeyboardUsage::KeyboardBb,
                0x0C => KeyboardUsage::KeyboardCc,
                0x0D => KeyboardUsage::KeyboardDd,
                0x0E => KeyboardUsage::KeyboardEe,
                0x0F => KeyboardUsage::KeyboardFf,
                _ => KeyboardUsage::KeyboardQq, // shouldn't happen
            };
        }
        OsInputMode::Win => {
            return match val {
                0x00 => KeyboardUsage::Keypad0Insert,
                0x01 => KeyboardUsage::Keypad1End,
                0x02 => KeyboardUsage::Keypad2DownArrow,
                0x03 => KeyboardUsage::Keypad3PageDown,
                0x04 => KeyboardUsage::Keypad4LeftArrow,
                0x05 => KeyboardUsage::Keypad5,
                0x06 => KeyboardUsage::Keypad6RightArrow,
                0x07 => KeyboardUsage::Keypad7Home,
                0x08 => KeyboardUsage::Keypad8UpArrow,
                0x09 => KeyboardUsage::Keypad9PageUp,
                0x0A => KeyboardUsage::KeyboardAa,
                0x0B => KeyboardUsage::KeyboardBb,
                0x0C => KeyboardUsage::KeyboardCc,
                0x0D => KeyboardUsage::KeyboardDd,
                0x0E => KeyboardUsage::KeyboardEe,
                0x0F => KeyboardUsage::KeyboardFf,
                _ => KeyboardUsage::KeyboardQq, // shouldn't happen
            };
        }
    }
}

fn get_raw_byte_from_switches(
    switches: &mut [Pin<DynPinId, FunctionSio<SioInput>, PullUp>; 8],
) -> u8 {
    let mut val: u8 = 0;

    for switch in switches {
        val = val << 1;
        val |= match switch.is_high().unwrap() {
            true => 1,
            false => 0,
        }
    }
    val
}

fn push_keys(report: KeyboardReport) -> Result<usize, usb_device::UsbError> {
    critical_section::with(|_| unsafe {
        // Now interrupts are disabled, grab the global variable and, if
        // available, send it a HID report
        USB_HID.as_mut().map(|hid| hid.push_input(&report))
    })
    .unwrap()
}

#[derive(Default, Copy, Clone)]
struct KbvalWithModKey {
    keycode: u8,
    modkey: u8,
}

fn ascii_to_key_usage(input: u8) -> Option<KbvalWithModKey> {
    let mut k: u8 = 0;
    let mut modkey: u8 = 0;
    match input {
        0 => (),
        1..=26 => {
            k = input - 1 + 0x04;
            modkey = 0x01;
        }
        27 => k = KeyboardEscape as u8,
        28..=32 => (),
        33 => {
            k = Keyboard1Exclamation as u8;
            modkey = 0x02;
        }
        34 => {
            k = KeyboardSingleDoubleQuote as u8;
            modkey = 0x02;
        }
        35 => {
            k = Keyboard3Hash as u8;
            modkey = 0x02;
        }
        36 => {
            k = Keyboard4Dollar as u8;
            modkey = 0x02;
        }
        37 => {
            k = Keyboard5Percent as u8;
            modkey = 0x02;
        }
        38 => {
            k = Keyboard7Ampersand as u8;
            modkey = 0x02;
        }
        39 => {
            k = KeyboardSingleDoubleQuote as u8;
        }
        40 => {
            k = Keyboard9OpenParens as u8;
            modkey = 0x02;
        }
        41 => {
            k = Keyboard0CloseParens as u8;
            modkey = 0x02;
        }
        42 => {
            k = Keyboard8Asterisk as u8;
            modkey = 0x02;
        }
        43 => {
            k = KeyboardEqualPlus as u8;
            modkey = 0x02;
        }
        44 => k = KeyboardCommaLess as u8,
        45 => k = KeyboardDashUnderscore as u8,
        46 => k = KeyboardPeriodGreater as u8,
        47 => k = KeyboardSlashQuestion as u8,
        48 => k = Keyboard0CloseParens as u8,
        49..=57 => k = input - 49 + 0x1e,
        58 => {
            k = KeyboardSemiColon as u8;
            modkey = 0x02;
        }
        59 => k = KeyboardSemiColon as u8,
        60 => {
            k = KeyboardCommaLess as u8;
            modkey = 0x02;
        }
        61 => k = KeyboardEqualPlus as u8,
        62 => {
            k = KeyboardPeriodGreater as u8;
            modkey = 0x02;
        }
        63 => {
            k = KeyboardSlashQuestion as u8;
            modkey = 0x02;
        }
        64 => {
            k = Keyboard2At as u8;
            modkey = 0x02;
        }
        65..=90 => {
            modkey = 0x02;
            k = input - 65 + 0x04;
        }
        91 => k = KeyboardOpenBracketBrace as u8,
        92 => k = KeyboardBackslashBar as u8,
        93 => k = KeyboardCloseBracketBrace as u8,
        94 => {
            k = Keyboard6Caret as u8;
            modkey = 0x02;
        }
        95 => {
            k = KeyboardDashUnderscore as u8;
            modkey = 0x02;
        }
        96 => k = KeyboardBacktickTilde as u8,
        97..=122 => k = input - 97 + 0x04,
        123 => {
            k = KeyboardOpenBracketBrace as u8;
            modkey = 0x02;
        }
        124 => {
            k = KeyboardBackslashBar as u8;
            modkey = 0x02;
        }
        125 => {
            k = KeyboardCloseBracketBrace as u8;
            modkey = 0x02;
        }
        126 => {
            k = KeyboardBacktickTilde as u8;
            modkey = 0x02;
        }
        127 => k = KeyboardBackspace as u8,

        _ => (),
    };
    return Some(KbvalWithModKey {
        keycode: k,
        modkey: modkey,
    });
}

#[derive(Default)]
struct Processor {
    bbuf: [u8; 4],
    bbufp: usize,
    expected_chars: usize,
}

struct ProcessError;

impl Processor {
    fn check_for_utf32val(&mut self, switch_state: u8) -> Result<Option<u32>, ProcessError> {
        match self.expected_chars {
            0 => match switch_state {
                0..=127 => return Ok(Some(switch_state as u32)),
                128..=191 => {
                    self.resetbuf();
                    Err(ProcessError)
                }
                192..=223 => {
                    self.bbuf[self.bbufp] = switch_state;
                    self.bbufp += 1;
                    self.expected_chars = 2;
                    Ok(None)
                }
                224..=239 => {
                    self.bbuf[self.bbufp] = switch_state;
                    self.bbufp += 1;
                    self.expected_chars = 3;
                    Ok(None)
                }
                240..=247 => {
                    self.bbuf[self.bbufp] = switch_state;
                    self.bbufp += 1;
                    self.expected_chars = 4;
                    Ok(None)
                }
                248..=u8::MAX => {
                    self.resetbuf();
                    Err(ProcessError)
                }
            },
            _ => {
                self.bbuf[self.bbufp] = switch_state;
                self.bbufp += 1;
                return match self.expected_chars {
                    1 => {
                        self.resetbuf();
                        Err(ProcessError)
                    }
                    2 => {
                        if switch_state >> 6 != 2 {
                            self.resetbuf();
                            Err(ProcessError)
                        } else {
                            let mut ret: u32 = (self.bbuf[0] as u32 & 0x1F) << 6;
                            ret |= self.bbuf[1] as u32 & 0x3F;
                            self.resetbuf();
                            Ok(Some(ret))
                        }
                    }
                    3 => {
                        match self.bbufp {
                            // second switch_state gotten. Not ready to finish processing
                            2 => {
                                if switch_state >> 6 != 2 {
                                    self.resetbuf();
                                    return Err(ProcessError);
                                }
                                Ok(None)
                            }
                            3 => {
                                if switch_state >> 6 != 2 {
                                    self.resetbuf();
                                    return Err(ProcessError);
                                }
                                let mut ret: u32 = (self.bbuf[0] as u32 & 0x0F) << 12;
                                ret |= (self.bbuf[1] as u32 & 0x3F) << 6;
                                ret |= self.bbuf[2] as u32 & 0x3F;
                                self.resetbuf();
                                Ok(Some(ret))
                            }
                            _ => {
                                self.resetbuf();
                                Err(ProcessError) // shouldn't happen
                            }
                        }
                    }
                    4 => {
                        match self.bbufp {
                            2 => {
                                if switch_state >> 6 != 2 {
                                    self.resetbuf();
                                    return Err(ProcessError);
                                }
                                Ok(None)
                            }
                            3 => {
                                if switch_state >> 6 != 2 {
                                    self.resetbuf();
                                    return Err(ProcessError);
                                }
                                Ok(None)
                            }
                            4 => {
                                if switch_state >> 6 != 2 {
                                    self.resetbuf();
                                    return Err(ProcessError);
                                }
                                let mut ret: u32 = (self.bbuf[0] as u32 & 0x07) << 18;
                                ret |= (self.bbuf[1] as u32 & 0x3F) << 12;
                                ret |= (self.bbuf[2] as u32 & 0x3F) << 6;
                                ret |= self.bbuf[3] as u32 & 0x3F;
                                self.resetbuf();
                                Ok(Some(ret))
                            }
                            _ => {
                                self.resetbuf();
                                Err(ProcessError)
                            } // shouldn't happen
                        }
                    }
                    _ => {
                        self.resetbuf();
                        Err(ProcessError) // shouldn't happen
                    }
                };
            }
        }
    }

    fn resetbuf(&mut self) {
        self.bbuf = [0, 0, 0, 0];
        self.bbufp = 0;
        self.expected_chars = 0;
    }
}

enum LightMode {
    Pulsing(u64),
    ErrBlink(u64),
    Startup(u64),
    Normal,
}

struct LightManager {
    mode: LightMode,
    default_brightness: u8,
}

impl LightManager {
    fn tick(
        &mut self,
        switch_state: u8,
        lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8],
        now: u64,
    ) {
        match &self.mode {
            LightMode::Startup(x) => match now - x {
                0..125 => set_lights(lights, 0b10000000, self.default_brightness),
                125..250 => set_lights(lights, 0b11000000, self.default_brightness),
                250..375 => set_lights(lights, 0b11100000, self.default_brightness),
                375..500 => set_lights(lights, 0b11110000, self.default_brightness),
                500..625 => set_lights(lights, 0b11111000, self.default_brightness),
                625..750 => set_lights(lights, 0b11111100, self.default_brightness),
                750..875 => set_lights(lights, 0b11111110, self.default_brightness),
                875..1000 => set_lights(lights, 0b11111111, self.default_brightness),
                _ => {
                    self.mode = LightMode::Normal;
                }
            },
            LightMode::Normal => {
                match_led_switch_states(switch_state, lights, self.default_brightness);
            }
            LightMode::Pulsing(x) => {
                let brightness = pulse_brightness(now, *x, self.default_brightness, 1500);
                let duty = brightness as u8;
                match_led_switch_states(switch_state, lights, duty);
            }
            LightMode::ErrBlink(x) => match now - x {
                0..250 => {
                    all_on(lights, self.default_brightness);
                }
                250..500 => {
                    all_off(lights);
                }
                500..750 => {
                    all_on(lights, self.default_brightness);
                }
                750..1000 => {
                    all_off(lights);
                }
                _ => {
                    self.mode = LightMode::Normal;
                }
            },
        }
    }
}

fn set_lights(
    lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8],
    pattern: u8,
    duty: u8,
) {
    for x in 0..8 {
        if pattern >> x & 0x01 == 1 {
            lights[x].set_duty_cycle_percent(duty);
        } else {
            lights[x].set_duty_cycle_percent(0);
        }
    }
}

fn all_off(lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8]) {
    for x in lights {
        x.set_duty_cycle_percent(0).unwrap();
    }
}

fn all_on(lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8], duty: u8) {
    for x in lights {
        x.set_duty_cycle_percent(duty).unwrap();
    }
}

fn pulse_brightness(now_ms: u64, start_ms: u64, max_duty_pct: u8, period_ms: u64) -> u8 {
    let elapsed = now_ms.wrapping_sub(start_ms);
    let t = (elapsed % period_ms) as f32 / period_ms as f32;
    let brightness = (libm::sinf(t * core::f32::consts::TAU) * 0.5 + 0.5) * max_duty_pct as f32;
    brightness as u8
}

trait SwitchBoard {
    fn get_state(&mut self) -> u8;
}

enum AnySwitchBoard {
    Toggle(ToggleBoard),
    Cherry(CherryBoardStateGetter),
}

impl AnySwitchBoard {
    fn get_state(&mut self) -> u8 {
        match self {
            AnySwitchBoard::Toggle(b) => b.get_state(),
            AnySwitchBoard::Cherry(b) => b.get_state(),
        }
    }
    fn get_state_raw(&mut self) -> u8 {
        match self {
            AnySwitchBoard::Toggle(b) => b.get_state(),
            AnySwitchBoard::Cherry(b) => b.get_raw_keys(),
        }
    }
}

struct ToggleBoard {
    switches: [Pin<DynPinId, FunctionSio<SioInput>, PullUp>; 8],
}

impl ToggleBoard {
    fn new(switches: [Pin<DynPinId, FunctionSio<SioInput>, PullUp>; 8]) -> Self {
        Self { switches }
    }
}

impl SwitchBoard for ToggleBoard {
    fn get_state(&mut self) -> u8 {
        return get_raw_byte_from_switches(&mut self.switches);
    }
}

struct CherryBoard {
    keys: [debounce::DebouncedButton; 8],
    states: [bool; 8],
}

// this doesn't really hold on to the keys as much as it requests the key states from the second thread
struct CherryBoardStateGetter {
    fifo: hal::sio::SioFifo,
}

impl CherryBoardStateGetter {
    fn new(fifo: hal::sio::SioFifo) -> Self {
        Self { fifo }
    }
    fn get_raw_keys(&mut self) -> u8 {
        self.fifo.write(1);
        self.fifo.read_blocking() as u8
    }
}

impl SwitchBoard for CherryBoardStateGetter {
    fn get_state(&mut self) -> u8 {
        self.fifo.write(0);
        self.fifo.read_blocking() as u8
    }
}
