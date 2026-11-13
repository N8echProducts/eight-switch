// Copyright 2026 Nathaniel Maile
// SPDX-License-Identifier: GPL-2.0-or-later

#include "eight_switch.h"

// LED above key column N is on GP(9 + N): the leftmost key (GP0) pairs with GP9,
// the rightmost key (GP7) with GP16.
static const pin_t led_pins[EIGHT_SWITCH_LED_COUNT] = {GP9, GP10, GP11, GP12, GP13, GP14, GP15, GP16};

#define STARTUP_STEP_MS 60

static uint8_t  pressed_mask = 0;
static bool     starting_up  = true;
static uint16_t startup_timer;

void eight_switch_set_leds(uint8_t mask) {
    for (uint8_t i = 0; i < EIGHT_SWITCH_LED_COUNT; i++) {
        gpio_write_pin(led_pins[i], (mask >> i) & 1);
    }
}

uint8_t eight_switch_pressed_mask(void) {
    return pressed_mask;
}

// Keymaps can override this to drive the LEDs themselves (layer indicators etc).
// Return false to stop the default "light the key you're holding" behaviour.
__attribute__((weak)) bool eight_switch_leds_user(uint8_t pressed) {
    return true;
}

static void update_leds(void) {
    if (starting_up) {
        return;
    }
    if (eight_switch_leds_user(pressed_mask)) {
        eight_switch_set_leds(pressed_mask);
    }
}

void keyboard_post_init_kb(void) {
    for (uint8_t i = 0; i < EIGHT_SWITCH_LED_COUNT; i++) {
        gpio_set_pin_output(led_pins[i]);
        gpio_write_pin_low(led_pins[i]);
    }
    startup_timer = timer_read();
    keyboard_post_init_user();
}

// Non-blocking left-to-right sweep on power-up, mirroring the stock firmware.
void housekeeping_task_kb(void) {
    if (starting_up) {
        uint16_t step = timer_elapsed(startup_timer) / STARTUP_STEP_MS;
        if (step < EIGHT_SWITCH_LED_COUNT) {
            eight_switch_set_leds((uint8_t)((1u << (step + 1)) - 1));
        } else if (step < EIGHT_SWITCH_LED_COUNT + 3) {
            eight_switch_set_leds(0xFF);
        } else {
            starting_up = false;
            update_leds();
        }
    }
    housekeeping_task_user();
}

bool process_record_kb(uint16_t keycode, keyrecord_t *record) {
    uint8_t col = record->event.key.col;
    if (IS_KEYEVENT(record->event) && record->event.key.row == 0 && col < EIGHT_SWITCH_LED_COUNT) {
        if (record->event.pressed) {
            pressed_mask |= (1 << col);
        } else {
            pressed_mask &= ~(1 << col);
        }
        update_leds();
    }
    return process_record_user(keycode, record);
}

layer_state_t layer_state_set_kb(layer_state_t state) {
    state = layer_state_set_user(state);
    // Give keymaps a chance to redraw layer indicators.
    update_leds();
    return state;
}
