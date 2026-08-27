/* Copyright 2026 Thearius
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

#include <stdbool.h>
#include <stdint.h>

/*
 * AL80 proprietary wireless MCU interface.
 *
 * Recovered from factory firmware:
 *
 *   STM32F103 USART1
 *   TX = PA9
 *   RX = PA10
 *   baud = 460800
 *
 * SmartBLE framing:
 *
 *   0x55 LEN COMMAND ...
 *
 * Modes:
 *   0 = USB/stop
 *   1 = Bluetooth slot 1
 *   2 = Bluetooth slot 2
 *   3 = Bluetooth slot 3
 *   4 = 2.4 GHz
 */

void al80_smartble_init(void);
void al80_smartble_task(void);

void al80_smartble_start(uint8_t mode);
void al80_smartble_pair(uint8_t mode);
void al80_smartble_stop(void);

bool    al80_smartble_connected(void);
uint8_t al80_smartble_mode(void);
uint8_t al80_smartble_leds(void);

/*
 * Physical tri-mode selector.
 *
 * Values:
 *   0 = USB
 *   1 = Bluetooth
 *   2 = 2.4 GHz
 *   0xFF = invalid/transitional state
 *
 * Phase 1A is monitor-only:
 * selector state does NOT automatically change transport yet.
 */
uint8_t al80_smartble_selector_raw(void);
uint8_t al80_smartble_selector_stable(void);
uint8_t al80_smartble_selector_candidate(void);
uint8_t al80_smartble_last_bt_mode(void);
uint16_t al80_smartble_selector_candidate_ms(void);

/*
 * Automatic transport transition diagnostics.
 *
 * transition_state:
 *   0 = idle
 *   1 = wireless wake delay
 *   2 = wireless second-command delay
 *   3 = USB stop delay
 *   4 = USB restore delay
 */
uint8_t al80_smartble_requested_mode(void);
uint8_t al80_smartble_transition_state(void);
uint16_t al80_smartble_transition_ms(void);


/*
 * Passive SmartBLE RX diagnostic capture.
 * No transport writes are performed by these functions.
 */
void al80_smartble_rx_capture_clear(void);
uint8_t al80_smartble_rx_capture_count(void);
uint8_t al80_smartble_rx_capture_length(uint8_t index);
uint8_t al80_smartble_rx_capture_byte(uint8_t index, uint8_t offset);
