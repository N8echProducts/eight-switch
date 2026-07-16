use embedded_hal::pwm::SetDutyCycle;
use rp_pico::hal::rom_data::float_funcs::uint64_to_float;

pub struct LightManager {
    mode: LightMode,
    default_brightness: u8,
}

impl LightManager {
    pub fn new(lightmode: LightMode, default_brightness: u8) -> Self {
        Self {
            mode: lightmode,
            default_brightness,
        }
    }

    pub fn set_mode(&mut self, mode: LightMode) {
        self.mode = mode;
    }

    pub fn tick(
        &mut self,
        switch_state: u8,
        lights: &mut [&mut dyn SetDutyCycle<Error = core::convert::Infallible>; 8],
        now: u64,
    ) {
        match &self.mode {
            LightMode::Startup(x) => match now - x {
                0..200 => set_lights(lights, 0b10000000, self.default_brightness),
                200..300 => set_lights(lights, 0b11000000, self.default_brightness),
                300..400 => set_lights(lights, 0b11100000, self.default_brightness),
                400..500 => set_lights(lights, 0b11110000, self.default_brightness),
                500..600 => set_lights(lights, 0b11111000, self.default_brightness),
                600..700 => set_lights(lights, 0b11111100, self.default_brightness),
                700..800 => set_lights(lights, 0b11111110, self.default_brightness),
                800..1000 => set_lights(lights, 0b11111111, self.default_brightness),
                _ => {
                    self.mode = LightMode::Normal;
                }
            },
            LightMode::Normal => {
                match_led_switch_states(switch_state, lights, self.default_brightness);
            }
            LightMode::Pulsing(x) => {
                let brightness = pulse_brightness(
                    now,
                    *x,
                    self.default_brightness,
                    self.default_brightness / 10,
                    1500,
                );
                let duty = brightness as u8;
                match_led_switch_states(switch_state, lights, duty);
            }
            LightMode::ErrBlink(x) => match now - x {
                0..125 => {
                    all_on(lights, self.default_brightness);
                }
                125..250 => {
                    all_off(lights);
                }
                250..375 => {
                    all_on(lights, self.default_brightness);
                }
                375..500 => {
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

pub enum LightMode {
    Pulsing(u64),
    ErrBlink(u64),
    Startup(u64),
    Normal,
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

fn pulse_brightness(
    now_ms: u64,
    start_ms: u64,
    max_duty_pct: u8,
    min_duty_pct: u8,
    period_ms: u64,
) -> u8 {
    let elapsed = now_ms.wrapping_sub(start_ms);
    let t = (elapsed % period_ms) as f32 / period_ms as f32;
    let envelope = libm::sinf(t * core::f32::consts::TAU) * 0.5 + 0.5; // 0.0..1.0
    let range = (max_duty_pct - min_duty_pct) as f32;
    let brightness = min_duty_pct as f32 + envelope * range;
    brightness as u8
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
