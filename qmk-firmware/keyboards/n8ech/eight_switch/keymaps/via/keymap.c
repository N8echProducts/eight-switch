// Copyright 2026 Nathaniel Maile
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H

enum layers {
    _BASE,
    _FN,
};

// Layout: eight keys left to right, then the "send" button.
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [_BASE] = LAYOUT(
        KC_1,    KC_2,    KC_3,    KC_4,    KC_5,    KC_6,    KC_7,    KC_8,
        MO(_FN)
    ),
    // Hold the send button.
    [_FN] = LAYOUT(
        KC_Q,    KC_W,    KC_E,    KC_R,    KC_T,    KC_Y,    KC_U,    KC_I,
        _______
    ),
};

// While the send button is held, invert the LEDs so it's obvious the layer changed.
bool eight_switch_leds_user(uint8_t pressed) {
    if (layer_state_is(_FN)) {
        eight_switch_set_leds(~pressed);
        return false;
    }
    return true;
}
