use embedded_hal::digital::InputPin;
use rp_pico::hal::gpio::{DynPinId, FunctionSio, Pin, PullUp, SioInput};

pub struct DebouncedButton {
    pin: Pin<DynPinId, FunctionSio<SioInput>, PullUp>,
    last_raw: bool,
    stable_state: bool,
    stable_since: u64,
    just_pressed_flag: bool,
}

impl DebouncedButton {
    pub fn new(pin: Pin<DynPinId, FunctionSio<SioInput>, PullUp>) -> Self {
        Self {
            pin,
            last_raw: false,
            stable_state: false,
            stable_since: 0,
            just_pressed_flag: false,
        }
    }

    pub fn update(&mut self, now_ms: u64) {
        // always clear the flag at the start of update
        self.just_pressed_flag = false;

        let current = self.pin.is_low().unwrap();
        if current != self.last_raw {
            self.last_raw = current;
            self.stable_since = now_ms;
        }
        if now_ms.wrapping_sub(self.stable_since) >= 10 {
            if current && !self.stable_state {
                // transition from released to pressed
                self.just_pressed_flag = true;
            }
            self.stable_state = current;
        }
    }

    pub fn just_pressed(&self) -> bool {
        self.just_pressed_flag
    }

    pub fn is_pressed_raw(&mut self) -> bool {
        self.pin.is_low().unwrap()
    }
}
