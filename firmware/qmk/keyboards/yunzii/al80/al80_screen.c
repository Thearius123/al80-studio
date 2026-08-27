// Copyright 2026
// SPDX-License-Identifier: GPL-2.0-or-later

#include "quantum.h"
#include "hal.h"

#include "al80_screen.h"
#include "al80_smartble.h"
#include "al80_battery.h"

/*
 * YUNZII AL80 factory-derived screen transport.
 *
 * Factory firmware evidence:
 *   USART3
 *   921600 baud
 *
 * Related YUNZII screen protocol corroboration:
 *   Header:     A5 5A
 *   Type:       1 byte
 *   Length:     2 bytes, big-endian
 *   CRC:        CRC16/MODBUS
 *   CRC init:   FFFF
 *   CRC layout: PK_CHECK_INFO
 *
 * Phase 1 deliberately implements ONLY Caps Lock status.
 *
 * Caps type:
 *   0x03
 *
 * Exact frames:
 *
 * OFF:
 *   A5 5A 03 00 01 00 40 00
 *
 * ON:
 *   A5 5A 03 00 01 00 40 01
 */

static const SerialConfig al80_screen_serial_config = {
    921600,
    0,
    USART_CR2_STOP1_BITS,
    0
};

static bool    al80_screen_ready = false;

/*
 * AL80_SCREEN_HOST_BRIDGE_V1
 *
 * Mirrors the factory PK_MOD_SEM behavior.
 *
 * While a PC -> LCD transfer is active, normal screen packets
 * (Caps, battery, connection, etc.) must not be interleaved
 * with the streamed image/data.
 */
static bool al80_screen_host_transfer_active = false;


/*
 * AL80_SCREEN_VOLUME_OSD_V1
 *
 * Host volume is displayed temporarily through the LCD's
 * existing percentage field. No picture/GIF storage is used.
 */
#define AL80_SCREEN_VOLUME_OSD_MS 1600

static bool     al80_screen_volume_active = false;
static uint32_t al80_screen_volume_timer  = 0;

/*
 * Real AL80 battery display state.
 *
 * Percentage is measured only while operating from battery.
 * PB9 is the factory-derived VBUS/charge-status input.
 */
static uint8_t  al80_screen_last_battery_percent = 0xFF;
static uint8_t  al80_screen_last_battery_status  = 0xFF;

static uint8_t  al80_screen_battery_percent = 100;
static bool     al80_screen_have_battery_percent = false;

/*
 * Becomes true only after an actual battery-side ADC measurement
 * during this boot. EEPROM restoration does NOT set this flag.
 */
static bool     al80_screen_real_battery_seen = false;

static uint32_t al80_screen_battery_timer = 0;
static uint8_t al80_screen_last_caps = 0xFF;
static uint8_t al80_screen_last_conn = 0xFF;

/*
 * STM32F103 USART3 partial remap:
 *
 * USART3_TX -> PC10
 * USART3_RX -> PC11
 *
 * We only actively configure/use TX in this first test.
 *
 * AFIO_MAPR USART3_REMAP:
 *   00 = PB10/PB11
 *   01 = partial remap PC10/PC11
 */
static void al80_screen_enable_usart3_remap(void) {
    AFIO->MAPR =
        (AFIO->MAPR & ~(3U << 4)) |
        (1U << 4);

    palSetPadMode(
        GPIOC,
        10,
        PAL_MODE_STM32_ALTERNATE_PUSHPULL
    );
}

static void al80_screen_send_connection(uint8_t mode) {
    uint8_t frame[8] = {
        0xA5,
        0x5A,
        0x01,
        0x00,
        0x01,
        0xC0,
        0xE1,
        mode
    };

    sdWrite(
        &SD3,
        frame,
        sizeof(frame)
    );
}

/*
 * CRC16/MODBUS used by the AL80 display protocol.
 *
 * Factory packet:
 *
 *   A5 5A TYPE LEN_H LEN_L CRC_H CRC_L PAYLOAD...
 *
 * CRC covers TYPE + LENGTH only.
 */
static uint16_t al80_screen_crc16_byte(
    uint16_t crc,
    uint8_t value
) {
    crc ^= value;

    for (uint8_t i = 0; i < 8; i++) {
        if (crc & 1U) {
            crc =
                (uint16_t)((crc >> 1) ^ 0xA001U);
        } else {
            crc >>= 1;
        }
    }

    return crc;
}

static uint16_t al80_screen_packet_crc(
    uint8_t type,
    uint16_t length
) {
    uint16_t crc = 0xFFFFU;

    crc = al80_screen_crc16_byte(crc, type);
    crc = al80_screen_crc16_byte(
        crc,
        (uint8_t)(length >> 8)
    );
    crc = al80_screen_crc16_byte(
        crc,
        (uint8_t)(length & 0xFF)
    );

    return crc;
}

static void al80_screen_send_byte_packet(
    uint8_t type,
    uint8_t value
) {
    uint16_t crc =
        al80_screen_packet_crc(type, 1);

    uint8_t frame[8] = {
        0xA5,
        0x5A,
        type,
        0x00,
        0x01,
        (uint8_t)(crc >> 8),
        (uint8_t)(crc & 0xFF),
        value
    };

    sdWrite(
        &SD3,
        frame,
        sizeof(frame)
    );
}

static void al80_screen_send_battery_percent(
    uint8_t percent
) {
    /*
     * Factory enum:
     * PK_BATT_QUANTITY = 0x06
     */
    al80_screen_send_byte_packet(
        0x06,
        percent
    );
}

static void al80_screen_send_battery_status(
    bool usb_present
) {
    /*
     * AL80 factory behavior:
     *
     * PK_BATT_STATUS = 0x07
     * payload = PB9 / VBUS state directly.
     */
    /*
     * Physical AL80 LCD validation:
     *
     * payload 0 -> charging icon ON
     * payload 1 -> charging icon OFF
     *
     * Therefore:
     *   USB present -> 0
     *   USB absent  -> 1
     */
    al80_screen_send_byte_packet(
        0x07,
        usb_present ? 0 : 1
    );
}

static void al80_screen_send_caps(bool enabled) {
    uint8_t frame[8] = {
        0xA5,
        0x5A,
        0x03,
        0x00,
        0x01,
        0x00,
        0x40,
        enabled ? 0x01 : 0x00
    };

    sdWrite(
        &SD3,
        frame,
        sizeof(frame)
    );
}


static bool al80_screen_host_write(
    const uint8_t *data,
    uint8_t length
) {
    if (!al80_screen_ready) {
        return false;
    }

    if (length == 0) {
        return true;
    }

    if (!data) {
        return false;
    }

    sdWrite(
        &SD3,
        data,
        length
    );

    return true;
}

bool al80_screen_host_begin(
    const uint8_t *data,
    uint8_t length
) {
    /*
     * Factory behavior:
     * 0x40 succeeds only when the screen upload semaphore
     * is currently free.
     */
    if (
        !al80_screen_ready ||
        al80_screen_host_transfer_active
    ) {
        return false;
    }

    /*
     * Acquire before transmitting so housekeeping cannot
     * interleave another screen packet.
     */
    al80_screen_host_transfer_active = true;

    if (
        !al80_screen_host_write(
            data,
            length
        )
    ) {
        al80_screen_host_transfer_active = false;
        return false;
    }

    return true;
}

bool al80_screen_host_data(
    const uint8_t *data,
    uint8_t length
) {
    /*
     * Factory behavior:
     * 0x41 is accepted only while the 0x40 transfer owns
     * the screen semaphore.
     */
    if (!al80_screen_host_transfer_active) {
        return false;
    }

    return al80_screen_host_write(
        data,
        length
    );
}

bool al80_screen_host_end(void) {
    /*
     * Factory behavior:
     * 0x42 releases the screen semaphore.
     */
    if (!al80_screen_host_transfer_active) {
        return false;
    }

    al80_screen_host_transfer_active = false;
    return true;
}


static uint8_t al80_volume_render_phase = 0;
static uint16_t al80_volume_render_row = 0;
static uint32_t al80_volume_render_timer = 0;

static bool al80_volume_render_pending = false;

static uint8_t al80_volume_target_percent = 0;
static bool al80_volume_target_muted = false;

static uint8_t al80_volume_frame_percent = 0;
static bool al80_volume_frame_muted = false;


static uint8_t al80_volume_glyph_column(
    char c,
    uint8_t col
) {
    if (col >= 5) {
        return 0;
    }

    static const uint8_t g_0[5] = {
        0x3E, 0x51, 0x49, 0x45, 0x3E
    };

    static const uint8_t g_1[5] = {
        0x00, 0x42, 0x7F, 0x40, 0x00
    };

    static const uint8_t g_2[5] = {
        0x42, 0x61, 0x51, 0x49, 0x46
    };

    static const uint8_t g_3[5] = {
        0x21, 0x41, 0x45, 0x4B, 0x31
    };

    static const uint8_t g_4[5] = {
        0x18, 0x14, 0x12, 0x7F, 0x10
    };

    static const uint8_t g_5[5] = {
        0x27, 0x45, 0x45, 0x45, 0x39
    };

    static const uint8_t g_6[5] = {
        0x3C, 0x4A, 0x49, 0x49, 0x30
    };

    static const uint8_t g_7[5] = {
        0x01, 0x71, 0x09, 0x05, 0x03
    };

    static const uint8_t g_8[5] = {
        0x36, 0x49, 0x49, 0x49, 0x36
    };

    static const uint8_t g_9[5] = {
        0x06, 0x49, 0x49, 0x29, 0x1E
    };

    static const uint8_t g_pct[5] = {
        0x63, 0x13, 0x08, 0x64, 0x63
    };

    static const uint8_t g_V[5] = {
        0x1F, 0x20, 0x40, 0x20, 0x1F
    };

    static const uint8_t g_O[5] = {
        0x3E, 0x41, 0x41, 0x41, 0x3E
    };

    static const uint8_t g_L[5] = {
        0x7F, 0x40, 0x40, 0x40, 0x40
    };

    static const uint8_t g_U[5] = {
        0x3F, 0x40, 0x40, 0x40, 0x3F
    };

    static const uint8_t g_M[5] = {
        0x7F, 0x02, 0x0C, 0x02, 0x7F
    };

    static const uint8_t g_E[5] = {
        0x7F, 0x49, 0x49, 0x49, 0x41
    };

    static const uint8_t g_T[5] = {
        0x01, 0x01, 0x7F, 0x01, 0x01
    };

    const uint8_t *glyph = NULL;

    switch (c) {
        case '0':
            glyph = g_0;
            break;

        case '1':
            glyph = g_1;
            break;

        case '2':
            glyph = g_2;
            break;

        case '3':
            glyph = g_3;
            break;

        case '4':
            glyph = g_4;
            break;

        case '5':
            glyph = g_5;
            break;

        case '6':
            glyph = g_6;
            break;

        case '7':
            glyph = g_7;
            break;

        case '8':
            glyph = g_8;
            break;

        case '9':
            glyph = g_9;
            break;

        case '%':
            glyph = g_pct;
            break;

        case 'V':
            glyph = g_V;
            break;

        case 'O':
            glyph = g_O;
            break;

        case 'L':
            glyph = g_L;
            break;

        case 'U':
            glyph = g_U;
            break;

        case 'M':
            glyph = g_M;
            break;

        case 'E':
            glyph = g_E;
            break;

        case 'T':
            glyph = g_T;
            break;

        default:
            return 0;
    }

    return glyph[col];
}


static uint8_t al80_volume_text_length(
    const char *text
) {
    uint8_t length = 0;

    while (text[length] != '\0') {
        length++;
    }

    return length;
}


static uint8_t al80_volume_text_width(
    const char *text,
    uint8_t scale
) {
    uint8_t length =
        al80_volume_text_length(text);

    if (length == 0) {
        return 0;
    }

    return (uint8_t)(
        length * 6U * scale - scale
    );
}


static bool al80_volume_text_pixel(
    const char *text,
    uint8_t x0,
    uint8_t y0,
    uint8_t scale,
    uint8_t x,
    uint8_t y
) {
    if (
        y < y0 ||
        y >= (uint8_t)(y0 + 7U * scale)
    ) {
        return false;
    }

    uint8_t cursor = x0;

    for (
        uint8_t i = 0;
        text[i] != '\0';
        i++
    ) {
        if (
            x >= cursor &&
            x < (uint8_t)(
                cursor + 5U * scale
            )
        ) {
            uint8_t glyph_col =
                (uint8_t)(
                    (x - cursor) / scale
                );

            uint8_t glyph_row =
                (uint8_t)(
                    (y - y0) / scale
                );

            uint8_t bits =
                al80_volume_glyph_column(
                    text[i],
                    glyph_col
                );

            return (
                bits &
                (uint8_t)(1U << glyph_row)
            ) != 0;
        }

        cursor =
            (uint8_t)(
                cursor + 6U * scale
            );
    }

    return false;
}


static void al80_volume_make_value_text(
    char *out,
    uint8_t percent,
    bool muted
) {
    if (muted) {
        out[0] = 'M';
        out[1] = 'U';
        out[2] = 'T';
        out[3] = 'E';
        out[4] = '\0';
        return;
    }

    if (percent >= 100) {
        out[0] = '1';
        out[1] = '0';
        out[2] = '0';
        out[3] = '%';
        out[4] = '\0';
        return;
    }

    if (percent >= 10) {
        out[0] =
            (char)(
                '0' + percent / 10
            );

        out[1] =
            (char)(
                '0' + percent % 10
            );

        out[2] = '%';
        out[3] = '\0';
        out[4] = '\0';
        return;
    }

    out[0] =
        (char)(
            '0' + percent
        );

    out[1] = '%';
    out[2] = '\0';
    out[3] = '\0';
    out[4] = '\0';
}


static bool al80_volume_pixel_on(
    uint8_t x,
    uint8_t y,
    uint8_t percent,
    bool muted
) {
    static const char title[] = "VOLUME";

    uint8_t title_scale = 2;

    uint8_t title_width =
        al80_volume_text_width(
            title,
            title_scale
        );

    uint8_t title_x =
        (uint8_t)(
            (96U - title_width) / 2U
        );

    if (
        al80_volume_text_pixel(
            title,
            title_x,
            14,
            title_scale,
            x,
            y
        )
    ) {
        return true;
    }

    char value[5];

    al80_volume_make_value_text(
        value,
        percent,
        muted
    );

    uint8_t value_scale = 3;

    uint8_t value_width =
        al80_volume_text_width(
            value,
            value_scale
        );

    uint8_t value_x =
        (uint8_t)(
            (96U - value_width) / 2U
        );

    if (
        al80_volume_text_pixel(
            value,
            value_x,
            55,
            value_scale,
            x,
            y
        )
    ) {
        return true;
    }

    const uint8_t left = 10;
    const uint8_t right = 85;
    const uint8_t top = 108;
    const uint8_t bottom = 127;

    bool border =
        (
            x >= left &&
            x <= right &&
            (
                y == top ||
                y == (uint8_t)(top + 1) ||
                y == (uint8_t)(bottom - 1) ||
                y == bottom
            )
        ) ||
        (
            y >= top &&
            y <= bottom &&
            (
                x == left ||
                x == (uint8_t)(left + 1) ||
                x == (uint8_t)(right - 1) ||
                x == right
            )
        );

    if (border) {
        return true;
    }

    if (
        !muted &&
        y >= (uint8_t)(top + 4) &&
        y <= (uint8_t)(bottom - 4) &&
        x >= (uint8_t)(left + 4)
    ) {
        uint8_t inner_width =
            (uint8_t)(
                right - left - 7U
            );

        uint8_t fill =
            (uint8_t)(
                (
                    (uint16_t)inner_width *
                    percent
                ) / 100U
            );

        if (
            x <
            (uint8_t)(
                left + 4U + fill
            )
        ) {
            return true;
        }
    }

    return false;
}


static uint8_t al80_volume_render_row_buffer[192];
static uint16_t al80_volume_render_row_offset = 0;


static bool al80_volume_send_row(
    uint8_t row_index,
    uint8_t percent,
    bool muted
) {
    if (
        al80_volume_render_row_offset == 0
    ) {
        for (
            uint8_t x = 0;
            x < 96;
            x++
        ) {
            bool on =
                al80_volume_pixel_on(
                    x,
                    row_index,
                    percent,
                    muted
                );

            uint16_t rgb565 =
                on ? 0xFFFFU : 0x0000U;

            al80_volume_render_row_buffer[
                (uint16_t)x * 2U
            ] =
                (uint8_t)(
                    rgb565 >> 8
                );

            al80_volume_render_row_buffer[
                (uint16_t)x * 2U + 1U
            ] =
                (uint8_t)(
                    rgb565 & 0xFFU
                );
        }
    }

    size_t written =
        sdAsynchronousWrite(
            &SD3,
            &al80_volume_render_row_buffer[
                al80_volume_render_row_offset
            ],
            sizeof(
                al80_volume_render_row_buffer
            ) -
            al80_volume_render_row_offset
        );

    al80_volume_render_row_offset +=
        (uint16_t)written;

    if (
        al80_volume_render_row_offset <
        sizeof(
            al80_volume_render_row_buffer
        )
    ) {
        return false;
    }

    al80_volume_render_row_offset = 0;

    return true;
}


static bool al80_screen_volume_render_task(
    void
) {
    if (!al80_screen_ready) {
        return false;
    }

    if (
        al80_screen_host_transfer_active
    ) {
        return false;
    }

    if (
        al80_volume_render_phase == 0
    ) {
        if (
            !al80_volume_render_pending
        ) {
            return false;
        }

        static const uint8_t gui_event[] = {
            0xA5,
            0x5A,
            0x10,
            0x00,
            0x01,
            0xC5,
            0xB1,
            0x01
        };

        al80_volume_frame_percent =
            al80_volume_target_percent;

        al80_volume_frame_muted =
            al80_volume_target_muted;

        al80_volume_render_pending =
            false;

        sdWrite(
            &SD3,
            gui_event,
            sizeof(gui_event)
        );

        al80_volume_render_timer =
            timer_read32();

        al80_volume_render_phase = 1;

        return true;
    }

    if (
        al80_volume_render_phase == 1
    ) {
        if (
            timer_elapsed32(
                al80_volume_render_timer
            ) < 150
        ) {
            return true;
        }

        static const uint8_t add_pic[] = {
            0xA5,
            0x5A,
            0x0C,
            0x78,
            0x00,
            0xC3,
            0x93
        };

        al80_volume_frame_percent =
            al80_volume_target_percent;

        al80_volume_frame_muted =
            al80_volume_target_muted;

        al80_volume_render_pending =
            false;

        sdWrite(
            &SD3,
            add_pic,
            sizeof(add_pic)
        );

        al80_volume_render_row = 0;
        al80_volume_render_row_offset = 0;
        al80_volume_render_phase = 2;

        return true;
    }

    if (
        al80_volume_render_phase == 2
    ) {
        if (
            al80_volume_send_row(
                (uint8_t)
                    al80_volume_render_row,
                al80_volume_frame_percent,
                al80_volume_frame_muted
            )
        ) {
            al80_volume_render_row++;

            if (
                al80_volume_render_row >= 160
            ) {
                al80_volume_render_phase = 0;
            }
        }

        return true;
    }

    al80_volume_render_phase = 0;

    return false;
}


bool al80_screen_show_volume(
    uint8_t percent,
    bool muted
) {
    if (!al80_screen_ready) {
        return false;
    }

    if (percent > 100) {
        percent = 100;
    }

    al80_volume_target_percent =
        percent;

    al80_volume_target_muted =
        muted;

    al80_volume_render_pending =
        true;

    return true;
}


uint8_t al80_screen_host_read(
    uint8_t *data,
    uint8_t max_length
) {
    uint8_t count = 0;

    if (
        !al80_screen_ready ||
        data == NULL ||
        max_length == 0
    ) {
        return 0;
    }

    while (
        count < max_length &&
        !sdGetWouldBlock(&SD3)
    ) {
        msg_t value = sdGet(&SD3);

        if (value < 0) {
            break;
        }

        data[count++] = (uint8_t)value;
    }

    return count;
}

void al80_screen_init(void) {
    /*
     * A8 and C9 are already configured by keyboard_post_init_kb().
     * Do not alter them here.
     */

    al80_screen_enable_usart3_remap();

    sdStart(
        &SD3,
        &al80_screen_serial_config
    );

    /*
     * Do NOT transmit during initialization.
     *
     * First packet is sent only after an actual host Caps state
     * transition. This keeps the first hardware test conservative.
     */
    al80_screen_last_caps =
        host_keyboard_led_state().caps_lock ? 1 : 0;

    /*
     * Restore the last REAL battery-side percentage.
     *
     * This allows the LCD to show battery quantity even when the
     * keyboard boots directly in USB mode, where ADC9 itself is
     * not a valid battery measurement.
     */
    uint8_t saved_percent = 0;

    if (
        al80_battery_saved_percent_load(
            &saved_percent
        )
    ) {
        al80_screen_battery_percent =
            saved_percent;

        al80_screen_have_battery_percent =
            true;
    }

    /*
     * Do not transmit connection state during boot either.
     * Cache current active SmartBLE transport and wait for a
     * real selector/transport transition.
     */
    al80_screen_last_conn = al80_smartble_mode();

    al80_screen_ready = true;
}

void al80_screen_task(void) {

    if (
        al80_screen_volume_render_task()
    ) {
        return;
    }

    if (!al80_screen_ready) {
        return;
    }

    /*
     * Do not interleave normal status packets with a host
     * image/data stream.
     *
     * This mirrors the factory PK_MOD_SEM gate.
     */
    if (al80_screen_host_transfer_active) {
        return;
    }

    /*
     * Keep normal Caps/battery/connection packets from
     * overwriting the temporary volume percentage.
     */
    if (al80_screen_volume_active) {
        if (
            timer_elapsed32(
                al80_screen_volume_timer
            ) < AL80_SCREEN_VOLUME_OSD_MS
        ) {
            return;
        }

        /*
         * Timeout expired. Force normal battery/status values
         * to be emitted again by the existing housekeeping path.
         */
        al80_screen_volume_active = false;

        al80_screen_last_battery_percent = 0xFF;
        al80_screen_last_battery_status  = 0xFF;
    }

    /*
     * Existing Caps display behavior.
     */
    uint8_t caps =
        host_keyboard_led_state().caps_lock ? 1 : 0;

    if (caps != al80_screen_last_caps) {
        al80_screen_last_caps = caps;
        al80_screen_send_caps(caps != 0);
    }

    /*
     * Existing connection-mode display behavior remains handled
     * below by the previously recovered SmartBLE mode state.
     */
    uint8_t conn =
        al80_smartble_mode();

    if (conn != al80_screen_last_conn) {
        al80_screen_last_conn = conn;
        al80_screen_send_connection(conn);
    }

    /*
     * AL80 factory-derived battery status.
     *
     * PB9:
     *   LOW  = USB/VBUS absent
     *   HIGH = USB/VBUS present
     *
     * PK_BATT_STATUS 0x07 receives this exact one-byte value
     * in the factory firmware.
     */
    uint8_t battery_status =
        gpio_read_pin(B9) ? 1 : 0;

    if (
        battery_status !=
        al80_screen_last_battery_status
    ) {
        al80_screen_last_battery_status =
            battery_status;

        al80_screen_send_battery_status(
            battery_status != 0
        );
    }

    /*
     * Percentage measurement.
     *
     * PB1/ADC9 does NOT contain a meaningful battery voltage
     * while USB is present, as physically validated on this AL80.
     *
     * Therefore measure only when VBUS is absent.
     *
     * Factory firmware uses a roughly 30-second periodic refresh.
     */
    if (!battery_status) {
        bool measure = false;

        if (!al80_screen_have_battery_percent) {
            measure = true;
        } else if (
            timer_elapsed32(
                al80_screen_battery_timer
            ) >= 30000
        ) {
            measure = true;
        }

        if (measure) {
            al80_battery_sample_t sample = {0};

            if (
                al80_battery_measure(&sample) &&
                sample.valid
            ) {
                al80_screen_battery_percent =
                    sample.percent;

                al80_screen_have_battery_percent =
                    true;

                al80_screen_real_battery_seen =
                    true;

                /*
                 * This measurement was taken only while VBUS was
                 * absent, so it is a genuine battery reading.
                 */
                al80_battery_saved_percent_store(
                    sample.percent
                );

                al80_screen_battery_timer =
                    timer_read32();
            }
        }
    }

    /*
     * Send quantity only after a real battery-side reading exists.
     *
     * When USB is subsequently connected, retain that last valid
     * percentage instead of replacing it with the invalid USB ADC9
     * reading.
     */
    if (
        al80_screen_have_battery_percent &&
        al80_screen_battery_percent !=
            al80_screen_last_battery_percent
    ) {
        al80_screen_last_battery_percent =
            al80_screen_battery_percent;

        al80_screen_send_battery_percent(
            al80_screen_battery_percent
        );
    }
}


bool al80_screen_real_battery_percent(
    uint8_t *percent
) {
    if (
        !percent ||
        !al80_screen_real_battery_seen
    ) {
        return false;
    }

    *percent = al80_screen_battery_percent;
    return true;
}
