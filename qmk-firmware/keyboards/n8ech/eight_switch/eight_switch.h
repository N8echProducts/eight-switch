// Copyright 2026 Nathaniel Maile
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include "quantum.h"

#define EIGHT_SWITCH_LED_COUNT 8

// Turn LEDs on/off. Bit 0 is the leftmost LED, bit 7 the rightmost.
void eight_switch_set_leds(uint8_t mask);

// Which of the eight top keys are currently held (bit 0 = leftmost).
uint8_t eight_switch_pressed_mask(void);

// Weak hook a keymap may override. Called whenever keys or layers change.
// Return true to let the board light up whichever keys are held,
// or false if the keymap has already set the LEDs itself.
bool eight_switch_leds_user(uint8_t pressed);
