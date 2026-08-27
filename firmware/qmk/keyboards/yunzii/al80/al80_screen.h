// Copyright 2026
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

void al80_screen_init(void);
void al80_screen_task(void);

bool al80_screen_real_battery_percent(uint8_t *percent);


/*
 * AL80_SCREEN_HOST_BRIDGE_V1
 *
 * Factory-derived screen upload transport:
 *
 *   0x40 -> begin/info
 *   0x41 -> continuation/data
 *   0x42 -> finish/release
 *
 * The host-command wrapper is removed before these functions
 * are called. Only the actual LCD UART bytes are passed here.
 */
bool al80_screen_host_begin(
    const uint8_t *data,
    uint8_t length
);

bool al80_screen_host_data(
    const uint8_t *data,
    uint8_t length
);

bool al80_screen_host_end(void);


/*
 * AL80_SCREEN_VOLUME_OSD_V1
 *
 * Temporarily displays the host system volume using the
 * LCD module's native percentage field.
 */
bool al80_screen_show_volume(
    uint8_t percent,
    bool muted
);


/*
 * AL80_LCD_UART_RX_BRIDGE_V1
 *
 * Non-blocking read of bytes returned by the LCD module
 * on USART3 RX / PC11.
 */
uint8_t al80_screen_host_read(
    uint8_t *data,
    uint8_t max_length
);
