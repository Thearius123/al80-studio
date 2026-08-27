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

#include "al80_smartble.h"

#include "host.h"
#include "gpio.h"
#include "action.h"
#include "host_driver.h"
#include "keyboard.h"
#include "report.h"
#include "timer.h"
#include "uart.h"
#include "wait.h"

#include <string.h>

#define AL80_SMARTBLE_BAUD 460800
#define AL80_SMARTBLE_SYNC 0x55

#define AL80_SMARTBLE_MODE_USB 0
#define AL80_SMARTBLE_MODE_BT1 1
#define AL80_SMARTBLE_MODE_BT2 2
#define AL80_SMARTBLE_MODE_BT3 3
#define AL80_SMARTBLE_MODE_24G 4

#define AL80_SMARTBLE_WAKE_BYTES 60

/*
 * Physical AL80 tri-mode selector recovered directly from hardware:
 *
 *   PC14 PC15
 *     1    1  = USB
 *     0    1  = Bluetooth
 *     1    0  = 2.4 GHz
 *     0    0  = invalid/transitional
 *
 * The mechanical selector briefly passes through USB while moving
 * between Bluetooth and 2.4G, so a stable-state debounce is required.
 */
#define AL80_SELECTOR_USB      0
#define AL80_SELECTOR_BT       1
#define AL80_SELECTOR_24G      2
#define AL80_SELECTOR_INVALID  0xFF

#define AL80_SELECTOR_DEBOUNCE_MS 200

/*
 * Non-blocking automatic transport transition states.
 */
#define AL80_TRANSITION_IDLE            0
#define AL80_TRANSITION_WIRELESS_WAKE   1
#define AL80_TRANSITION_WIRELESS_SECOND 2
#define AL80_TRANSITION_USB_STOP        3
#define AL80_TRANSITION_USB_RESTORE     4

#define AL80_WIRELESS_WAKE_MS   350
#define AL80_WIRELESS_SECOND_MS 10
#define AL80_USB_STOP_WAKE_MS   100
#define AL80_USB_RESTORE_MS     20

/*
 * Keep this intentionally conservative.
 *
 * The existing secondary MCU already owns Bluetooth/2.4G pairing and RF.
 * We are only restoring the STM32 -> wireless MCU transport.
 *
 * Factory/B75 protocol uses a 20-byte command payload followed by a
 * fixed 22-byte frame including sync and length.
 *
 * The AL80 factory binary contains "SmartBLE".  We avoid changing the
 * advertised keyboard name in V1; the name field is zeroed.
 */
static uint8_t smartble_command[22] = {
    0x55, 0x14, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00
};

static bool     smartble_uart_ready = false;
static bool     smartble_connected  = false;
static uint8_t  smartble_mode       = AL80_SMARTBLE_MODE_USB;
static uint8_t  smartble_led_state  = 0;

/*
 * AL80_BATTERY_RX_PROBE_V1
 *
 * Passive diagnostic only.
 * Records frames that the SmartBLE MCU ALREADY sends.
 * Does not transmit anything and does not change transport behavior.
 */
#define AL80_RX_CAPTURE_COUNT 16
#define AL80_RX_CAPTURE_SIZE  32

typedef struct {
    uint8_t length;
    uint8_t payload[AL80_RX_CAPTURE_SIZE];
} al80_rx_capture_t;

static al80_rx_capture_t al80_rx_capture[AL80_RX_CAPTURE_COUNT];
static uint8_t al80_rx_capture_count = 0;

void al80_smartble_rx_capture_clear(void) {
    al80_rx_capture_count = 0;
    memset(al80_rx_capture, 0, sizeof(al80_rx_capture));
}

uint8_t al80_smartble_rx_capture_count(void) {
    return al80_rx_capture_count;
}

uint8_t al80_smartble_rx_capture_length(uint8_t index) {
    if (index >= al80_rx_capture_count) {
        return 0;
    }

    return al80_rx_capture[index].length;
}

uint8_t al80_smartble_rx_capture_byte(uint8_t index, uint8_t offset) {
    if (
        index >= al80_rx_capture_count ||
        offset >= al80_rx_capture[index].length ||
        offset >= AL80_RX_CAPTURE_SIZE
    ) {
        return 0xFF;
    }

    return al80_rx_capture[index].payload[offset];
}

static void al80_smartble_rx_capture_store(
    uint8_t length,
    const uint8_t *payload
) {
    if (length > AL80_RX_CAPTURE_SIZE) {
        return;
    }

    /*
     * Battery discovery probe:
     *
     * Keep UNIQUE frames rather than the most recent frames.
     * SmartBLE repeatedly emits connection/LED status packets;
     * allowing duplicates would evict rare packet types that are
     * more interesting for reverse engineering.
     */
    for (uint8_t i = 0; i < al80_rx_capture_count; i++) {
        if (
            al80_rx_capture[i].length == length &&
            memcmp(
                al80_rx_capture[i].payload,
                payload,
                length
            ) == 0
        ) {
            return;
        }
    }

    if (al80_rx_capture_count >= AL80_RX_CAPTURE_COUNT) {
        return;
    }

    al80_rx_capture_t *dst =
        &al80_rx_capture[al80_rx_capture_count++];

    dst->length = length;
    memcpy(dst->payload, payload, length);
}

/*
 * Remember the most recently requested Bluetooth slot.
 *
 * This is RAM-only in Phase 1A. EEPROM persistence comes later.
 */
static uint8_t smartble_last_bt_mode = AL80_SMARTBLE_MODE_BT1;

/*
 * Physical selector state.
 *
 * Phase 1A intentionally observes and debounces the selector without
 * allowing it to change transport. This lets us validate the state
 * machine before giving it authority over USB/BT/2.4G.
 */
static uint8_t  selector_raw_state       = AL80_SELECTOR_INVALID;
static uint8_t  selector_candidate_state = AL80_SELECTOR_INVALID;
static uint8_t  selector_stable_state    = AL80_SELECTOR_INVALID;
static uint32_t selector_candidate_timer = 0;

/*
 * Automatic transport state.
 *
 * requested_mode uses the same values as SmartBLE:
 *   0 = USB
 *   1..3 = BT slots
 *   4 = 2.4G
 */
static uint8_t  smartble_requested_mode  = AL80_SMARTBLE_MODE_USB;
static uint8_t  smartble_transition_state = AL80_TRANSITION_IDLE;
static uint32_t smartble_transition_timer = 0;

static host_driver_t *usb_host_driver = NULL;

/* ---------- Physical mode selector ---------- */

static uint8_t smartble_read_selector(void) {
    bool pc14 = gpio_read_pin(C14);
    bool pc15 = gpio_read_pin(C15);

    if (pc14 && pc15) {
        return AL80_SELECTOR_USB;
    }

    if (!pc14 && pc15) {
        return AL80_SELECTOR_BT;
    }

    if (pc14 && !pc15) {
        return AL80_SELECTOR_24G;
    }

    return AL80_SELECTOR_INVALID;
}

static void smartble_selector_init(void) {
    uint8_t raw = smartble_read_selector();

    selector_raw_state       = raw;
    selector_candidate_state = raw;

    /*
     * Do not trust the first electrical sample at boot.
     * The selector must remain valid and unchanged for the full
     * debounce interval before becoming stable.
     */
    selector_stable_state    = AL80_SELECTOR_INVALID;
    selector_candidate_timer = timer_read32();
}

static void smartble_selector_task(void) {
    uint8_t  raw = smartble_read_selector();
    uint32_t now = timer_read32();

    selector_raw_state = raw;

    /*
     * 00 is not a valid settled AL80 selector position.
     * Never promote it to the stable state.
     */
    if (raw == AL80_SELECTOR_INVALID) {
        selector_candidate_state = AL80_SELECTOR_INVALID;
        selector_candidate_timer = now;
        return;
    }

    /*
     * A newly observed state must remain unchanged for the full
     * debounce interval before it becomes authoritative.
     */
    if (raw != selector_candidate_state) {
        selector_candidate_state = raw;
        selector_candidate_timer = now;
        return;
    }

    if (raw != selector_stable_state &&
        timer_elapsed32(selector_candidate_timer) >= AL80_SELECTOR_DEBOUNCE_MS) {
        selector_stable_state = raw;
    }
}

/* ---------- Forward declarations ---------- */

static void smartble_wake_module(void);
static void smartble_send_mode_command(uint8_t mode);
static void smartble_select_host_driver(void);
static void smartble_restore_usb_driver(void);

/* ---------- Automatic transport state machine ---------- */

static uint8_t smartble_selector_to_requested_mode(uint8_t selector) {
    switch (selector) {
        case AL80_SELECTOR_USB:
            return AL80_SMARTBLE_MODE_USB;

        case AL80_SELECTOR_BT:
            return smartble_last_bt_mode;

        case AL80_SELECTOR_24G:
            return AL80_SMARTBLE_MODE_24G;

        default:
            return smartble_requested_mode;
    }
}

static void smartble_begin_wireless_transition(uint8_t mode) {
    if (mode < AL80_SMARTBLE_MODE_BT1 ||
        mode > AL80_SMARTBLE_MODE_24G) {
        return;
    }

    /*
     * Release all keys through the CURRENT transport before changing
     * mode/connection state. This prevents stuck keys or modifiers on
     * the host we are leaving.
     */
    clear_keyboard();

    smartble_requested_mode = mode;

    if (mode >= AL80_SMARTBLE_MODE_BT1 &&
        mode <= AL80_SMARTBLE_MODE_BT3) {
        smartble_last_bt_mode = mode;
    }

    /*
     * Accept RX status frames for the requested wireless mode from the
     * beginning of the transition. The host driver itself remains
     * unchanged until both START commands have been sent.
     *
     * This also rejects delayed status frames from the previous mode.
     */
    smartble_mode      = mode;
    smartble_connected = false;

    /*
     * Keep the existing host driver during RF wake.
     */
    smartble_wake_module();

    smartble_transition_state = AL80_TRANSITION_WIRELESS_WAKE;
    smartble_transition_timer = timer_read32();
}

static void smartble_begin_usb_transition(void) {
    /*
     * Release through the current transport while its connection state
     * and protocol mode are still valid.
     */
    clear_keyboard();

    smartble_requested_mode = AL80_SMARTBLE_MODE_USB;
    smartble_connected      = false;

    smartble_wake_module();

    smartble_transition_state = AL80_TRANSITION_USB_STOP;
    smartble_transition_timer = timer_read32();
}

static void smartble_auto_request(uint8_t requested) {
    if (requested == smartble_requested_mode &&
        smartble_transition_state == AL80_TRANSITION_IDLE) {
        return;
    }

    if (requested == AL80_SMARTBLE_MODE_USB) {
        smartble_begin_usb_transition();
    } else {
        smartble_begin_wireless_transition(requested);
    }
}

static void smartble_transition_task(void) {
    uint32_t elapsed = timer_elapsed32(smartble_transition_timer);

    switch (smartble_transition_state) {
        case AL80_TRANSITION_IDLE:
            return;

        case AL80_TRANSITION_WIRELESS_WAKE:
            if (elapsed < AL80_WIRELESS_WAKE_MS) {
                return;
            }

            smartble_send_mode_command(smartble_requested_mode);

            smartble_transition_state = AL80_TRANSITION_WIRELESS_SECOND;
            smartble_transition_timer = timer_read32();
            return;

        case AL80_TRANSITION_WIRELESS_SECOND:
            if (elapsed < AL80_WIRELESS_SECOND_MS) {
                return;
            }

            smartble_send_mode_command(smartble_requested_mode);

            /*
             * smartble_mode was already selected at transition start so
             * the RX parser could accept an early connection/status frame.
             */
            smartble_select_host_driver();

            smartble_transition_state = AL80_TRANSITION_IDLE;
            return;

        case AL80_TRANSITION_USB_STOP:
            if (elapsed < AL80_USB_STOP_WAKE_MS) {
                return;
            }

            uart_write(0x55);
            uart_write(0x02);
            uart_write(0x00);
            uart_write(0x00);

            smartble_mode = AL80_SMARTBLE_MODE_USB;

            smartble_transition_state = AL80_TRANSITION_USB_RESTORE;
            smartble_transition_timer = timer_read32();
            return;

        case AL80_TRANSITION_USB_RESTORE:
            if (elapsed < AL80_USB_RESTORE_MS) {
                return;
            }

            smartble_restore_usb_driver();

            smartble_transition_state = AL80_TRANSITION_IDLE;
            return;

        default:
            smartble_transition_state = AL80_TRANSITION_IDLE;
            return;
    }
}

/* ---------- UART helpers ---------- */

static void smartble_wake_module(void) {
    for (uint8_t i = 0; i < AL80_SMARTBLE_WAKE_BYTES; i++) {
        uart_write(0x00);
    }
}

static void smartble_send_mode_command(uint8_t mode) {
    smartble_command[0] = 0x55;
    smartble_command[1] = 20;
    smartble_command[2] = 0;
    smartble_command[3] = mode;

    uart_transmit(smartble_command, sizeof(smartble_command));
}

/* ---------- Wireless host driver ---------- */

static uint8_t smartble_keyboard_leds(void) {
    return smartble_led_state;
}

static void smartble_send_keyboard(report_keyboard_t *report) {
    /*
     * Factory/B75 protocol:
     *
     * 55 09 01 + 8-byte boot keyboard report
     */
    if (!smartble_connected) {
        return;
    }

    uart_write(0x55);
    uart_write(0x09);
    uart_write(0x01);

    uart_transmit((const uint8_t *)report, KEYBOARD_REPORT_SIZE);

    /*
     * Factory code spaces Bluetooth reports by roughly 8 ms.
     * 2.4G uses a shorter interval.
     */
    if (smartble_mode == AL80_SMARTBLE_MODE_24G) {
        wait_ms(2);
    } else {
        wait_ms(8);
    }
}

static void smartble_send_mouse(report_mouse_t *report) {
    if (!smartble_connected) {
        return;
    }

    /*
     * Do not assume the AL80's wireless MCU accepts the current
     * QMK mouse report ABI until keyboard transport is proven.
     *
     * V1 therefore intentionally suppresses wireless mouse reports.
     */
    (void)report;
}

static void smartble_send_extra(report_extra_t *report) {
    if (!smartble_connected) {
        return;
    }

    /*
     * B75 factory-derived protocol sends:
     *
     * 55 LEN + report_extra_t
     *
     * Keep this available for media/system keys.
     */
    uart_write(0x55);
    uart_write(sizeof(report_extra_t));
    uart_transmit((const uint8_t *)report, sizeof(report_extra_t));

    if (smartble_mode == AL80_SMARTBLE_MODE_24G) {
        wait_ms(2);
    } else {
        wait_ms(8);
    }
}

static host_driver_t smartble_host_driver = {
    .keyboard_leds = smartble_keyboard_leds,
    .send_keyboard = smartble_send_keyboard,

    /*
     * NKRO format differs between QMK generations.
     * Leave NULL in V1 so we do not send an incorrect packet.
     */
    .send_nkro = NULL,

    .send_mouse = smartble_send_mouse,
    .send_extra = smartble_send_extra,
};

/* ---------- Driver switching ---------- */

static void smartble_select_host_driver(void) {
    if (host_get_driver() == &smartble_host_driver) {
        return;
    }

    clear_keyboard();

    usb_host_driver = host_get_driver();

    host_set_driver(&smartble_host_driver);
}

static void smartble_restore_usb_driver(void) {
    if (host_get_driver() != &smartble_host_driver) {
        return;
    }

    clear_keyboard();

    if (usb_host_driver != NULL) {
        host_set_driver(usb_host_driver);
    }
}

/* ---------- Public API ---------- */

void al80_smartble_init(void) {
    if (smartble_uart_ready) {
        return;
    }

    /*
     * QMK ChibiOS UART defaults for STM32F1:
     *
     * UART_DRIVER = SD1
     * TX = A9
     * RX = A10
     *
     * Those are the exact factory USART1 pins recovered for AL80.
     */
    uart_init(AL80_SMARTBLE_BAUD);

    smartble_selector_init();

    smartble_uart_ready = true;
}

void al80_smartble_start(uint8_t mode) {
    if (mode < AL80_SMARTBLE_MODE_BT1 ||
        mode > AL80_SMARTBLE_MODE_24G) {
        return;
    }

    al80_smartble_init();

    if (mode >= AL80_SMARTBLE_MODE_BT1 &&
        mode <= AL80_SMARTBLE_MODE_BT3) {
        smartble_last_bt_mode = mode;
    }

    smartble_requested_mode = mode;
    smartble_mode           = mode;
    smartble_connected      = false;
    smartble_transition_state = AL80_TRANSITION_IDLE;

    smartble_select_host_driver();

    smartble_wake_module();

    /*
     * Factory-derived code waits ~350 ms after wake bytes.
     */
    wait_ms(350);

    smartble_send_mode_command(mode);
    wait_ms(10);

    /*
     * Factory sends the startup command twice.
     */
    smartble_send_mode_command(mode);
}

void al80_smartble_pair(uint8_t mode) {
    if (mode < AL80_SMARTBLE_MODE_BT1 ||
        mode > AL80_SMARTBLE_MODE_24G) {
        return;
    }

    al80_smartble_init();

    smartble_mode      = mode;
    smartble_connected = false;

    smartble_select_host_driver();

    smartble_wake_module();
    wait_ms(350);

    /*
     * Pair command recovered from YUNZII SmartBLE:
     *
     * 55 03 00 MODE 01
     *
     * sent twice.
     */
    for (uint8_t attempt = 0; attempt < 2; attempt++) {
        uart_write(0x55);
        uart_write(0x03);
        uart_write(0x00);
        uart_write(mode);
        uart_write(0x01);

        wait_ms(10);
    }
}

void al80_smartble_stop(void) {
    if (!smartble_uart_ready) {
        smartble_restore_usb_driver();
        return;
    }

    smartble_connected = false;

    smartble_wake_module();
    wait_ms(100);

    /*
     * Tell wireless MCU to enter USB/off mode.
     *
     * 55 02 00 00
     */
    uart_write(0x55);
    uart_write(0x02);
    uart_write(0x00);
    uart_write(0x00);

    smartble_requested_mode  = AL80_SMARTBLE_MODE_USB;
    smartble_mode            = AL80_SMARTBLE_MODE_USB;
    smartble_transition_state = AL80_TRANSITION_IDLE;

    wait_ms(20);

    smartble_restore_usb_driver();
}

bool al80_smartble_connected(void) {
    return smartble_connected;
}

uint8_t al80_smartble_mode(void) {
    return smartble_mode;
}

uint8_t al80_smartble_leds(void) {
    return smartble_led_state;
}

uint8_t al80_smartble_selector_raw(void) {
    return selector_raw_state;
}

uint8_t al80_smartble_selector_stable(void) {
    return selector_stable_state;
}

uint8_t al80_smartble_selector_candidate(void) {
    return selector_candidate_state;
}

uint8_t al80_smartble_last_bt_mode(void) {
    return smartble_last_bt_mode;
}

uint16_t al80_smartble_selector_candidate_ms(void) {
    uint32_t elapsed = timer_elapsed32(selector_candidate_timer);

    if (elapsed > 0xFFFF) {
        return 0xFFFF;
    }

    return (uint16_t)elapsed;
}

uint8_t al80_smartble_requested_mode(void) {
    return smartble_requested_mode;
}

uint8_t al80_smartble_transition_state(void) {
    return smartble_transition_state;
}

uint16_t al80_smartble_transition_ms(void) {
    uint32_t elapsed = timer_elapsed32(smartble_transition_timer);

    if (elapsed > 0xFFFF) {
        return 0xFFFF;
    }

    return (uint16_t)elapsed;
}

/* ---------- RX parser ---------- */



void al80_smartble_task(void) {
    /*
     * Expected status frame:
     *
     * 55 03 COMMAND MODE DATA
     *
     * COMMAND 0 = connection state
     * COMMAND 1 = keyboard LED state
     *
     * Factory-derived semantics:
     * connection DATA == 0 => connected
     */

    enum {
        RX_SYNC,
        RX_LENGTH,
        RX_PAYLOAD
    };

    static uint8_t state   = RX_SYNC;
    static uint8_t length  = 0;
    static uint8_t index   = 0;
    static uint8_t payload[32];

    if (!smartble_uart_ready) {
        return;
    }

    /*
     * Update and debounce the physical selector.
     */
    uint8_t previous_stable = selector_stable_state;

    smartble_selector_task();

    /*
     * Only a newly debounced physical position may request a transport
     * transition. Raw/candidate changes never control transport.
     */
    if (selector_stable_state != AL80_SELECTOR_INVALID &&
        selector_stable_state != previous_stable) {
        uint8_t requested =
            smartble_selector_to_requested_mode(selector_stable_state);

        smartble_auto_request(requested);
    }

    /*
     * Run the non-blocking automatic transport sequence.
     */
    smartble_transition_task();

    while (uart_available()) {
        uint8_t c = uart_read();

        switch (state) {
            case RX_SYNC:
                if (c == AL80_SMARTBLE_SYNC) {
                    state = RX_LENGTH;
                }
                break;

            case RX_LENGTH:
                if (c >= 2 && c <= sizeof(payload)) {
                    length = c;
                    index  = 0;
                    state  = RX_PAYLOAD;
                } else if (c != AL80_SMARTBLE_SYNC) {
                    state = RX_SYNC;
                }
                break;

            case RX_PAYLOAD:
                payload[index++] = c;

                if (index >= length) {
                    /*
                     * Passive diagnostic capture.
                     * Save exactly what the wireless MCU sent before
                     * applying any semantic interpretation.
                     */
                    al80_smartble_rx_capture_store(
                        length,
                        payload
                    );

                    /*
                     * Status packets relevant to us are length 3:
                     *
                     * payload[0] = command
                     * payload[1] = wireless mode
                     * payload[2] = data
                     */
                    if (length == 3) {
                        uint8_t command = payload[0];
                        uint8_t mode    = payload[1];
                        uint8_t data    = payload[2];

                        bool mode_matches =
                            (smartble_mode <= 3 && mode == smartble_mode) ||
                            (smartble_mode == 4 && mode == 4);

                        if (mode_matches) {
                            if (command == 0) {
                                smartble_connected = (data == 0);
                            } else if (command == 1) {
                                smartble_led_state = data;
                            }
                        }
                    }

                    state = RX_SYNC;
                    index = 0;
                }
                break;

            default:
                state = RX_SYNC;
                index = 0;
                break;
        }
    }
}
