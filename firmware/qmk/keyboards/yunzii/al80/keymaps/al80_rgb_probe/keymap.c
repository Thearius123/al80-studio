/* 
Copyright 2021 owlab
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 2 of the License, or
(at your option) any later version.
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/


#include QMK_KEYBOARD_H
#include "al80_smartble.h"
#include "al80_screen.h"


const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {

	[0] = LAYOUT(
		KC_ESC,  KC_F1,    KC_F2,    KC_F3,    KC_F4,    KC_F5,    KC_F6,    KC_F7,    KC_F8,    KC_F9,    KC_F10,  	KC_F11,  	KC_F12,  	KC_DELETE,  KC_MUTE,
		KC_GRV,  KC_1,     KC_2,     KC_3,     KC_4,     KC_5,     KC_6,     KC_7,     KC_8,     KC_9,     KC_0,    	KC_MINS,  	KC_EQL,  	KC_BSPC,    KC_PAGE_UP,
		KC_TAB,  KC_Q,     KC_W,     KC_E,     KC_R,     KC_T,     KC_Y,     KC_U,     KC_I,     KC_O,     KC_P,    	KC_LBRC, 	KC_RBRC, 	KC_BSLS,    KC_PAGE_DOWN,
		KC_CAPS, KC_A,     KC_S,     KC_D,     KC_F,     KC_G,     KC_H,     KC_J,     KC_K,     KC_L,     KC_SCLN, 	KC_QUOT,    KC_ENT,
		KC_LSFT, KC_Z,     KC_X,     KC_C,     KC_V,     KC_B,     KC_N,     KC_M,     KC_COMM,  KC_DOT,      KC_SLSH, 	KC_RSFT,    KC_UP,      
		KC_LCTL, KC_LGUI,  KC_LALT,                      KC_SPC,             MO(1),   KC_RCTL,   KC_LEFT,  KC_DOWN,	    KC_RIGHT 
	),
	[1] = LAYOUT(
		QK_BOOT,  KC_BRID,  KC_BRIU, LGUI(KC_TAB), 	KC_MYCM, 	KC_MAIL,   KC_WHOM,    KC_MPRV,  KC_MPLY,  KC_MNXT,     KC_MUTE, 	KC_VOLD, 	KC_VOLU,  	_______,   KC_MUTE,
		_______,  _______,  _______,  _______,  _______,   _______,     _______,     _______,     _______,     _______,     _______,    	_______,  	_______,  	_______,    _______,
		_______,  _______,  _______,     _______,     _______,      _______,     _______,     _______,     _______,     _______,     _______,    	_______, 	_______, 	RM_NEXT,    _______,
		_______,  _______,     _______,     _______,     _______,      _______,     _______,     _______,     _______,     _______,     _______, 	_______,    RM_HUEU,
		_______,  _______,     _______,     _______,     _______,      _______,     _______,     _______,     _______,  _______,      _______, 	_______,    RM_VALU,      
		_______,  _______,      _______,                  _______,     _______,      _______,    RM_SPDD,  RM_VALD,	    RM_SPDU 
	),
	[2] = LAYOUT(
		KC_ESC,  KC_F1,    KC_F2,    KC_F3,    KC_F4,    KC_F5,    KC_F6,    KC_F7,    KC_F8,    KC_F9,    KC_F10,  	KC_F11,  	KC_F12,  	KC_DELETE,  KC_MUTE,
		KC_GRV,  KC_1,     KC_2,     KC_3,     KC_4,     KC_5,     KC_6,     KC_7,     KC_8,     KC_9,     KC_0,    	KC_MINS,  	KC_EQL,  	KC_BSPC,    KC_PAGE_UP,
		KC_TAB,  KC_Q,     KC_W,     KC_E,     KC_R,     KC_T,     KC_Y,     KC_U,     KC_I,     KC_O,     KC_P,    	KC_LBRC, 	KC_RBRC, 	KC_BSLS,    KC_PAGE_DOWN,
		KC_CAPS, KC_A,     KC_S,     KC_D,     KC_F,     KC_G,     KC_H,     KC_J,     KC_K,     KC_L,     KC_SCLN, 	KC_QUOT,    KC_ENT,
		KC_LSFT, KC_Z,     KC_X,     KC_C,     KC_V,     KC_B,     KC_N,     KC_M,     KC_COMM,  KC_DOT,      KC_SLSH, 	KC_RSFT,    KC_UP,      
		KC_LCTL, KC_LALT,  KC_LGUI,                      KC_SPC,              MO(3),   KC_RCTL,  KC_LEFT,  KC_DOWN,	    KC_RIGHT 
	),
	[3] = LAYOUT(
		QK_BOOT,  KC_BRID,  KC_BRIU,  LCTL(KC_UP),   KC_LPAD, 	 _______,     _______,     KC_MPRV, 	KC_MPLY,    KC_MNXT,     KC_MUTE, 	   KC_VOLD, 	 KC_VOLU,  	_______,   KC_MUTE,
		_______,  _______,  _______,  _______,       _______,     _______,     _______,     _______,     _______,     _______,     _______,    	_______,  	_______,  	_______,    _______,
		_______,  _______,     _______,   _______,     _______,      _______,     _______,     _______,     _______,     _______,     _______,    	_______, 	_______, 	RM_NEXT,    _______,
		_______,  _______,     _______,     _______,     _______,      _______,     _______,     _______,     _______,     _______,     _______, 	_______,    RM_HUEU,
		_______,  _______,     _______,     _______,     _______,      _______,     _______,     _______,     _______,  _______,      _______, 	_______,    RM_VALU,      
		_______,  _______,     _______,                  _______,     _______,      _______,    RM_SPDD,  RM_VALD,	    RM_SPDU 
	)
};
						



/*
 * AL80 RGB probe.
 *
 * Recovery:
 *   Fn+Esc -> QK_BOOT (already in layers 1 and 3)
 *   VIA Raw HID 0x0B -> bootloader_jump()
 *
 * RGB probe:
 *   all LEDs OFF
 *   LED index 0 = dim red
 */

#ifdef VIA_ENABLE
#    include "via.h"
#    include "raw_hid.h"
#    include "bootloader.h"

static uint32_t al80_matrix_scan_counter = 0;
static uint32_t al80_matrix_scan_rate_hz = 0;
static uint32_t al80_matrix_scan_window_timer = 0;

void matrix_scan_user(void) {
    al80_matrix_scan_counter++;

    if ((al80_matrix_scan_counter & 0xFFU) != 0U) {
        return;
    }

    uint32_t now = timer_read32();

    if (al80_matrix_scan_window_timer == 0U) {
        al80_matrix_scan_window_timer = now;
        al80_matrix_scan_counter = 0;
        return;
    }

    uint32_t elapsed =
        now - al80_matrix_scan_window_timer;

    if (elapsed >= 1000U) {
        al80_matrix_scan_rate_hz =
            (uint32_t)(
                (al80_matrix_scan_counter * 1000U) /
                elapsed
            );

        al80_matrix_scan_counter = 0;
        al80_matrix_scan_window_timer = now;
    }
}


#ifdef RGB_MATRIX_ENABLE
static bool al80_custom_rgb_overlay_enabled = true;

/*
 * AL80_CREATOR_RGB_SCENE_V1
 *
 * Host uploads a complete frame into staging.
 * COMMIT atomically replaces the visible active frame.
 *
 * RAM only:
 *   no EEPROM
 *   no flash write
 */
#define AL80_CREATOR_LED_COUNT RGB_MATRIX_LED_COUNT
#define AL80_CREATOR_CHUNK_MAX 9

static bool al80_creator_scene_enabled = false;

static uint8_t
    al80_creator_scene_staging[AL80_CREATOR_LED_COUNT][3];

static uint8_t
    al80_creator_scene_active[AL80_CREATOR_LED_COUNT][3];

/*
 * AL80_LIVE_RGB_TELEMETRY_V1
 *
 * Read-only host telemetry for the final RGB values authored by this
 * AL80 indicator callback. No EEPROM, flash, or runtime lighting state
 * is changed by telemetry queries.
 *
 * source:
 *   0 = native/unknown (not frame-readable here)
 *   1 = Snake/Heart overlay
 *   2 = Creator Scene
 *   3 = low-battery safety red
 */
#define AL80_LIVE_RGB_TELEMETRY_VERSION 1
#define AL80_LIVE_RGB_TELEMETRY_CHUNK 8

static uint8_t
    al80_live_rgb_shadow[AL80_CREATOR_LED_COUNT][3];

static bool al80_live_rgb_frame_valid = false;
static uint8_t al80_live_rgb_source = 0;

static void al80_live_rgb_set_color(
    uint8_t index,
    uint8_t red,
    uint8_t green,
    uint8_t blue
) {
    if (index < AL80_CREATOR_LED_COUNT) {
        al80_live_rgb_shadow[index][0] = red;
        al80_live_rgb_shadow[index][1] = green;
        al80_live_rgb_shadow[index][2] = blue;
    }

    rgb_matrix_set_color(index, red, green, blue);
}

#endif

/*
 * AL80_INPUT_ROUTER_V1
 *
 * Volatile, typed, allowlisted knob routing.
 *
 * No EEPROM.
 * No arbitrary QMK keycode injection from host.
 * No bootloader action.
 * No firmware write.
 *
 * Configuration is lost when the keyboard reboots and must be
 * reapplied by AL80 Studio / al80d.
 */

#define AL80_INPUT_BINDING_MAX 12

enum al80_input_event {
    AL80_INPUT_EVENT_NONE = 0,
    AL80_INPUT_EVENT_KNOB_CCW = 1,
    AL80_INPUT_EVENT_KNOB_CW = 2,
    AL80_INPUT_EVENT_KNOB_PRESS = 3,
};

enum al80_input_trigger {
    AL80_INPUT_TRIGGER_NONE = 0,
    AL80_INPUT_TRIGGER_LAYER = 1,
    AL80_INPUT_TRIGGER_MATRIX = 2,
    AL80_INPUT_TRIGGER_MODS = 3,
};

enum al80_input_action {
    AL80_INPUT_ACTION_NONE = 0,

    AL80_INPUT_ACTION_VOLUME_DOWN = 1,
    AL80_INPUT_ACTION_VOLUME_UP = 2,
    AL80_INPUT_ACTION_MUTE = 3,

    AL80_INPUT_ACTION_MEDIA_PREV = 4,
    AL80_INPUT_ACTION_MEDIA_NEXT = 5,
    AL80_INPUT_ACTION_MEDIA_PLAY_PAUSE = 6,

    AL80_INPUT_ACTION_BRIGHTNESS_DOWN = 7,
    AL80_INPUT_ACTION_BRIGHTNESS_UP = 8,

    AL80_INPUT_ACTION_LEFT = 9,
    AL80_INPUT_ACTION_RIGHT = 10,
    AL80_INPUT_ACTION_UP = 11,
    AL80_INPUT_ACTION_DOWN = 12,
    AL80_INPUT_ACTION_PAGE_UP = 13,
    AL80_INPUT_ACTION_PAGE_DOWN = 14,

    AL80_INPUT_ACTION_RGB_VALUE_DOWN = 15,
    AL80_INPUT_ACTION_RGB_VALUE_UP = 16,
    AL80_INPUT_ACTION_RGB_HUE_DOWN = 17,
    AL80_INPUT_ACTION_RGB_HUE_UP = 18,
    AL80_INPUT_ACTION_RGB_SPEED_DOWN = 19,
    AL80_INPUT_ACTION_RGB_SPEED_UP = 20,

    AL80_INPUT_ACTION_SNAKE_OFF = 21,
    AL80_INPUT_ACTION_SNAKE_ON = 22,
    AL80_INPUT_ACTION_SNAKE_TOGGLE = 23,

    AL80_INPUT_ACTION_CREATOR_SCENE_OFF = 24,

    AL80_INPUT_ACTION_MAX = 24,
};

typedef struct {
    uint8_t event;
    uint8_t trigger;
    uint8_t trigger_a;
    uint8_t trigger_b;
    uint8_t action;
    uint8_t flags;
} al80_input_binding_t;

static bool al80_input_router_enabled = false;

static bool
    al80_input_matrix_held[MATRIX_ROWS][MATRIX_COLS];

static al80_input_binding_t
    al80_input_bindings[AL80_INPUT_BINDING_MAX] = {
        {
            AL80_INPUT_EVENT_KNOB_CCW,
            AL80_INPUT_TRIGGER_NONE,
            0,
            0,
            AL80_INPUT_ACTION_VOLUME_DOWN,
            0,
        },
        {
            AL80_INPUT_EVENT_KNOB_CW,
            AL80_INPUT_TRIGGER_NONE,
            0,
            0,
            AL80_INPUT_ACTION_VOLUME_UP,
            0,
        },
        {
            AL80_INPUT_EVENT_KNOB_PRESS,
            AL80_INPUT_TRIGGER_NONE,
            0,
            0,
            AL80_INPUT_ACTION_MUTE,
            0,
        },
    };


/*
 * AL80_INPUT_ROUTER_EVENT_BRIDGE_V1
 *
 * Unsolicited host event protocol:
 *
 *   [0]     0x4C event namespace
 *   [1]     0xE1 unsolicited marker
 *   [2]     protocol version 1
 *   [3..4]  wrapping u16 sequence, little-endian
 *   [5]     input event: 1 CCW, 2 CW, 3 PRESS
 *   [6]     matched binding slot 0..11
 *   [7]     trigger kind
 *   [8]     trigger A
 *   [9]     trigger B
 *   [10]    action id 0..24
 *   [11]    flags = 0
 *   [12..13] wrapping u16 dropped-event counter
 *   [14]    router enabled snapshot = 1
 *   [15..31] reserved = 0
 *
 * Input execution is authoritative. Event delivery is best effort.
 * The input path only enqueues metadata. Raw HID transmission occurs
 * later from housekeeping_task_user(), at most one event per call.
 */

#define AL80_INPUT_HOST_EVENT_NAMESPACE 0x4CU
#define AL80_INPUT_HOST_EVENT_MARKER 0xE1U
#define AL80_INPUT_HOST_EVENT_VERSION 1U
#define AL80_INPUT_HOST_EVENT_REPORT_BYTES 32U
#define AL80_INPUT_HOST_EVENT_QUEUE_CAPACITY 8U

#if defined(RAW_EPSIZE) && RAW_EPSIZE != 32
#    error "AL80 input event bridge requires 32-byte Raw HID reports"
#endif

typedef struct {
    uint16_t sequence;
    uint16_t dropped_counter;
    uint8_t event;
    uint8_t slot;
    uint8_t trigger;
    uint8_t trigger_a;
    uint8_t trigger_b;
    uint8_t action;
} al80_input_host_event_t;

#ifdef VIA_ENABLE

static al80_input_host_event_t
    al80_input_host_event_queue[
        AL80_INPUT_HOST_EVENT_QUEUE_CAPACITY
    ];

static uint8_t al80_input_host_event_head = 0;
static uint8_t al80_input_host_event_tail = 0;
static uint8_t al80_input_host_event_count = 0;

static uint16_t al80_input_host_event_next_sequence = 0;
static uint16_t al80_input_host_event_dropped = 0;

static void al80_input_host_event_enqueue(
    uint8_t event,
    uint8_t slot,
    const al80_input_binding_t *binding
) {
    if (
        !al80_input_router_enabled ||
        binding == NULL
    ) {
        return;
    }

    if (
        al80_input_host_event_count >=
        AL80_INPUT_HOST_EVENT_QUEUE_CAPACITY
    ) {
        al80_input_host_event_dropped =
            (uint16_t)(
                al80_input_host_event_dropped + 1U
            );

        return;
    }

    al80_input_host_event_t *queued =
        &al80_input_host_event_queue[
            al80_input_host_event_head
        ];

    queued->sequence =
        al80_input_host_event_next_sequence;

    queued->dropped_counter =
        al80_input_host_event_dropped;

    queued->event = event;
    queued->slot = slot;
    queued->trigger = binding->trigger;
    queued->trigger_a = binding->trigger_a;
    queued->trigger_b = binding->trigger_b;
    queued->action = binding->action;

    al80_input_host_event_next_sequence =
        (uint16_t)(
            al80_input_host_event_next_sequence + 1U
        );

    al80_input_host_event_head =
        (uint8_t)(
            (
                al80_input_host_event_head + 1U
            ) %
            AL80_INPUT_HOST_EVENT_QUEUE_CAPACITY
        );

    al80_input_host_event_count++;
}

static void al80_input_host_event_send_one(void) {
    if (al80_input_host_event_count == 0) {
        return;
    }

    const al80_input_host_event_t *queued =
        &al80_input_host_event_queue[
            al80_input_host_event_tail
        ];

    uint8_t report[
        AL80_INPUT_HOST_EVENT_REPORT_BYTES
    ] = {0};

    report[0] = AL80_INPUT_HOST_EVENT_NAMESPACE;
    report[1] = AL80_INPUT_HOST_EVENT_MARKER;
    report[2] = AL80_INPUT_HOST_EVENT_VERSION;

    report[3] =
        (uint8_t)(queued->sequence & 0xFFU);

    report[4] =
        (uint8_t)(
            (queued->sequence >> 8) & 0xFFU
        );

    report[5] = queued->event;
    report[6] = queued->slot;
    report[7] = queued->trigger;
    report[8] = queued->trigger_a;
    report[9] = queued->trigger_b;
    report[10] = queued->action;
    report[11] = 0;

    report[12] =
        (uint8_t)(
            queued->dropped_counter & 0xFFU
        );

    report[13] =
        (uint8_t)(
            (
                queued->dropped_counter >> 8
            ) &
            0xFFU
        );

    report[14] = 1;

    raw_hid_send(
        report,
        AL80_INPUT_HOST_EVENT_REPORT_BYTES
    );

    al80_input_host_event_tail =
        (uint8_t)(
            (
                al80_input_host_event_tail + 1U
            ) %
            AL80_INPUT_HOST_EVENT_QUEUE_CAPACITY
        );

    al80_input_host_event_count--;
}

#else

static void al80_input_host_event_enqueue(
    uint8_t event,
    uint8_t slot,
    const al80_input_binding_t *binding
) {
    (void)event;
    (void)slot;
    (void)binding;
}

#endif

static bool al80_input_binding_valid(
    const al80_input_binding_t *binding
) {
    if (
        binding->event < AL80_INPUT_EVENT_KNOB_CCW ||
        binding->event > AL80_INPUT_EVENT_KNOB_PRESS
    ) {
        return false;
    }

    if (
        binding->trigger > AL80_INPUT_TRIGGER_MODS
    ) {
        return false;
    }

    if (
        binding->action > AL80_INPUT_ACTION_MAX
    ) {
        return false;
    }

    if (
        binding->trigger == AL80_INPUT_TRIGGER_LAYER &&
        binding->trigger_a >= 32
    ) {
        return false;
    }

    if (
        binding->trigger == AL80_INPUT_TRIGGER_MATRIX &&
        (
            binding->trigger_a >= MATRIX_ROWS ||
            binding->trigger_b >= MATRIX_COLS
        )
    ) {
        return false;
    }

    return true;
}

static bool al80_input_trigger_matches(
    const al80_input_binding_t *binding
) {
    switch (binding->trigger) {
        case AL80_INPUT_TRIGGER_NONE:
            return true;

        case AL80_INPUT_TRIGGER_LAYER:
            return layer_state_is(
                binding->trigger_a
            );

        case AL80_INPUT_TRIGGER_MATRIX:
            return al80_input_matrix_held[
                binding->trigger_a
            ][
                binding->trigger_b
            ];

        case AL80_INPUT_TRIGGER_MODS:
            return (
                get_mods() &
                binding->trigger_a
            ) == binding->trigger_a;

        default:
            return false;
    }
}

static void al80_input_execute_action(
    uint8_t action
) {
    switch (action) {
        case AL80_INPUT_ACTION_NONE:
            break;

        case AL80_INPUT_ACTION_VOLUME_DOWN:
            tap_code16(KC_VOLD);
            break;

        case AL80_INPUT_ACTION_VOLUME_UP:
            tap_code16(KC_VOLU);
            break;

        case AL80_INPUT_ACTION_MUTE:
            tap_code16(KC_MUTE);
            break;

        case AL80_INPUT_ACTION_MEDIA_PREV:
            tap_code16(KC_MPRV);
            break;

        case AL80_INPUT_ACTION_MEDIA_NEXT:
            tap_code16(KC_MNXT);
            break;

        case AL80_INPUT_ACTION_MEDIA_PLAY_PAUSE:
            tap_code16(KC_MPLY);
            break;

        case AL80_INPUT_ACTION_BRIGHTNESS_DOWN:
            tap_code16(KC_BRID);
            break;

        case AL80_INPUT_ACTION_BRIGHTNESS_UP:
            tap_code16(KC_BRIU);
            break;

        case AL80_INPUT_ACTION_LEFT:
            tap_code16(KC_LEFT);
            break;

        case AL80_INPUT_ACTION_RIGHT:
            tap_code16(KC_RIGHT);
            break;

        case AL80_INPUT_ACTION_UP:
            tap_code16(KC_UP);
            break;

        case AL80_INPUT_ACTION_DOWN:
            tap_code16(KC_DOWN);
            break;

        case AL80_INPUT_ACTION_PAGE_UP:
            tap_code16(KC_PGUP);
            break;

        case AL80_INPUT_ACTION_PAGE_DOWN:
            tap_code16(KC_PGDN);
            break;

#ifdef RGB_MATRIX_ENABLE
        case AL80_INPUT_ACTION_RGB_VALUE_DOWN:
            rgb_matrix_decrease_val_noeeprom();
            break;

        case AL80_INPUT_ACTION_RGB_VALUE_UP:
            rgb_matrix_increase_val_noeeprom();
            break;

        case AL80_INPUT_ACTION_RGB_HUE_DOWN:
            rgb_matrix_decrease_hue_noeeprom();
            break;

        case AL80_INPUT_ACTION_RGB_HUE_UP:
            rgb_matrix_increase_hue_noeeprom();
            break;

        case AL80_INPUT_ACTION_RGB_SPEED_DOWN:
            rgb_matrix_decrease_speed_noeeprom();
            break;

        case AL80_INPUT_ACTION_RGB_SPEED_UP:
            rgb_matrix_increase_speed_noeeprom();
            break;

        case AL80_INPUT_ACTION_SNAKE_OFF:
            al80_custom_rgb_overlay_enabled = false;
            break;

        case AL80_INPUT_ACTION_SNAKE_ON:
            al80_custom_rgb_overlay_enabled = true;
            break;

        case AL80_INPUT_ACTION_SNAKE_TOGGLE:
            al80_custom_rgb_overlay_enabled =
                !al80_custom_rgb_overlay_enabled;
            break;

        case AL80_INPUT_ACTION_CREATOR_SCENE_OFF:
            al80_creator_scene_enabled = false;
            break;
#endif

        default:
            break;
    }
}

static bool al80_input_route_event(
    uint8_t event
) {
    if (!al80_input_router_enabled) {
        return false;
    }

    /*
     * Specific triggers win over base/NONE bindings.
     * This allows Fn+knob or key+knob to override the normal
     * volume mapping without the default binding masking it.
     */
    for (
        uint8_t pass = 0;
        pass < 2;
        pass++
    ) {
        for (
            uint8_t slot = 0;
            slot < AL80_INPUT_BINDING_MAX;
            slot++
        ) {
            const al80_input_binding_t *binding =
                &al80_input_bindings[slot];

            if (binding->event != event) {
                continue;
            }

            bool base =
                binding->trigger ==
                AL80_INPUT_TRIGGER_NONE;

            if (
                (pass == 0 && base) ||
                (pass == 1 && !base)
            ) {
                continue;
            }

            if (
                !al80_input_binding_valid(binding) ||
                !al80_input_trigger_matches(binding)
            ) {
                continue;
            }

            /*
             * Action execution remains authoritative.
             * Event delivery is best effort and must never
             * suppress the routed keyboard action.
             */
            al80_input_execute_action(
                binding->action
            );

            al80_input_host_event_enqueue(
                event,
                slot,
                binding
            );

            return true;
        }
    }

    return false;
}

static void al80_input_default_event(
    uint8_t event
) {
    switch (event) {
        case AL80_INPUT_EVENT_KNOB_CCW:
            tap_code16(KC_VOLD);
            break;

        case AL80_INPUT_EVENT_KNOB_CW:
            tap_code16(KC_VOLU);
            break;

        case AL80_INPUT_EVENT_KNOB_PRESS:
            tap_code16(KC_MUTE);
            break;

        default:
            break;
    }
}


void housekeeping_task_user(void) {
#ifdef VIA_ENABLE
    /*
     * Never transmit unsolicited Raw HID from encoder or matrix
     * input callbacks. Send at most one queued event here.
     */
    al80_input_host_event_send_one();
#endif
}

bool encoder_update_user(
    uint8_t index,
    bool clockwise
) {
    if (index != 0) {
        return true;
    }

    uint8_t event =
        clockwise
            ? AL80_INPUT_EVENT_KNOB_CW
            : AL80_INPUT_EVENT_KNOB_CCW;

    if (!al80_input_route_event(event)) {
        al80_input_default_event(event);
    }

    return false;
}

bool process_record_user(
    uint16_t keycode,
    keyrecord_t *record
) {
    uint8_t row = record->event.key.row;
    uint8_t col = record->event.key.col;

    if (
        row < MATRIX_ROWS &&
        col < MATRIX_COLS
    ) {
        al80_input_matrix_held[row][col] =
            record->event.pressed;
    }

    /*
     * The knob push switch is matrix (0,14).
     *
     * Only that physical switch is intercepted. Other KC_MUTE
     * keys remain ordinary keymap keys.
     */
    if (row == 0 && col == 14) {
        if (record->event.pressed) {
            if (
                !al80_input_route_event(
                    AL80_INPUT_EVENT_KNOB_PRESS
                )
            ) {
                al80_input_default_event(
                    AL80_INPUT_EVENT_KNOB_PRESS
                );
            }
        }

        return false;
    }

    return true;
}

bool via_command_kb(uint8_t *data, uint8_t length) {
    if (length == 0) {
        return false;
    }

    /*
     * Preserve existing emergency recovery.
     */
    if (data[0] == id_bootloader_jump) {
        bootloader_jump();
        return true;
    }

    /*
     * AL80 SmartBLE test commands.
     *
     * 0x0C = USB / wireless stop
     * 0x0D = Bluetooth slot 1
     * 0x0E = Bluetooth slot 2
     * 0x0F = Bluetooth slot 3
     * 0x10 = 2.4G
     *
     * Pairing deliberately NOT exposed in V1.
     */
    switch (data[0]) {
        case 0x0C:
            al80_smartble_stop();
            return true;

        case 0x0D:
            al80_smartble_start(1);
            return true;

        case 0x0E:
            al80_smartble_start(2);
            return true;

        case 0x0F:
            al80_smartble_start(3);
            return true;

        case 0x10:
            al80_smartble_start(4);
            return true;

        /*
         * 0x11 = READ-ONLY hardware/status probe.
         *
         * Response:
         *   data[0] = 0x11
         *   data[1] = GPIO bitmap:
         *               bit 0 = PB9
         *               bit 1 = PC14
         *               bit 2 = PC15
         *   data[2] = SmartBLE mode
         *   data[3] = SmartBLE connected (0/1)
         *   data[4] = host LED state
         *   data[5] = 0xA1 signature
         *
         * No GPIO configuration or transport state is changed.
         */
        
        /*
         * 0xF0 — clear passive SmartBLE RX capture.
         */
        case 0xF0:
            al80_smartble_rx_capture_clear();
            memset(data, 0, length);
            data[0] = 0xF0;
            raw_hid_send(data, length);
            return true;

        /*
         * 0xF1 — read one captured SmartBLE frame.
         *
         * Request:
         *   [0] = F1
         *   [1] = frame index
         *
         * Response:
         *   [0] = F1
         *   [1] = total frame count
         *   [2] = requested frame index
         *   [3] = original payload length
         *   [4..31] = first up to 28 payload bytes
         */
        case 0xF1: {
            uint8_t requested =
                length >= 2 ? data[1] : 0;

            uint8_t count =
                al80_smartble_rx_capture_count();

            uint8_t frame_len =
                al80_smartble_rx_capture_length(requested);

            memset(data, 0xFF, length);

            data[0] = 0xF1;

            if (length >= 2) {
                data[1] = count;
            }

            if (length >= 3) {
                data[2] = requested;
            }

            if (length >= 4) {
                data[3] = frame_len;
            }

            uint8_t copy =
                frame_len > 28 ? 28 : frame_len;

            for (
                uint8_t i = 0;
                i < copy && (4 + i) < length;
                i++
            ) {
                data[4 + i] =
                    al80_smartble_rx_capture_byte(
                        requested,
                        i
                    );
            }

            raw_hid_send(data, length);
            return true;
        }

        case 0x11: {
            uint8_t gpio_state = 0;

            if (gpio_read_pin(B9)) {
                gpio_state |= (1u << 0);
            }

            if (gpio_read_pin(C14)) {
                gpio_state |= (1u << 1);
            }

            if (gpio_read_pin(C15)) {
                gpio_state |= (1u << 2);
            }

            data[0] = 0x11;
            data[1] = gpio_state;
            data[2] = al80_smartble_mode();
            data[3] = al80_smartble_connected() ? 1 : 0;
            data[4] = al80_smartble_leds();
            data[5] = 0xA1;

            /*
             * SmartBLE V2 Phase 1A diagnostic extension.
             *
             * data[6]  = raw physical selector
             * data[7]  = debounced/stable selector
             * data[8]  = current debounce candidate
             * data[9]  = last requested BT slot
             * data[10] = candidate age, low byte
             * data[11] = candidate age, high byte
             *
             * Existing bytes 0..5 remain compatible with the V1.1
             * diagnostic script.
             */
            if (length >= 12) {
                uint16_t candidate_ms =
                    al80_smartble_selector_candidate_ms();

                data[6]  = al80_smartble_selector_raw();
                data[7]  = al80_smartble_selector_stable();
                data[8]  = al80_smartble_selector_candidate();
                data[9]  = al80_smartble_last_bt_mode();
                data[10] = candidate_ms & 0xFF;
                data[11] = candidate_ms >> 8;

                /*
                 * Phase 1B automatic transition diagnostics.
                 *
                 * data[12] = requested SmartBLE mode
                 * data[13] = transition state
                 * data[14] = transition age low
                 * data[15] = transition age high
                 */
                if (length >= 16) {
                    uint16_t transition_ms =
                        al80_smartble_transition_ms();

                    data[12] = al80_smartble_requested_mode();
                    data[13] = al80_smartble_transition_state();
                    data[14] = transition_ms & 0xFF;
                    data[15] = transition_ms >> 8;
                }
            }

            /*
             * via_command_kb() returning true means the keyboard-level
             * handler is responsible for sending the Raw HID response.
             */
            raw_hid_send(data, length);

            return true;
        }

        /*
         * AL80_FLASH_READ_COMMAND_V1
         *
         * Strictly read-only access to the preserved factory
         * flash candidate region.
         *
         * Host -> keyboard:
         *
         *   data[0]    = 0x45
         *   data[1..4] = absolute STM32 flash address,
         *                little-endian
         *   data[5]    = requested byte count
         *
         * Keyboard -> host:
         *
         *   data[0]    = 0x45
         *   data[1]    = 0x55 success
         *                0x0F rejected
         *   data[2]    = returned byte count
         *   data[3..6] = echoed address
         *   data[7..]  = flash bytes
         *
         * Hard safety gates:
         *
         *   allowed address range:
         *       0x0800D400 .. 0x08012FFF
         *
         *   maximum bytes per request:
         *       Raw HID report length - 7
         *
         * No flash erase/program API is called here.
         * The STM32F1 flash is read through its normal
         * memory-mapped address space only.
         */
        case 0x45: {
            const uint32_t flash_read_min =
                0x0800D400UL;

            const uint32_t flash_read_end =
                0x08013000UL;

            bool ok = false;
            uint8_t count = 0;
            uint32_t address = 0;

            if (length >= 7) {
                address =
                    ((uint32_t)data[1]) |
                    ((uint32_t)data[2] << 8) |
                    ((uint32_t)data[3] << 16) |
                    ((uint32_t)data[4] << 24);

                uint8_t requested =
                    data[5];

                uint8_t max_payload =
                    (uint8_t)(length - 7);

                if (requested > max_payload) {
                    requested = max_payload;
                }

                /*
                 * Reject zero-length reads and reject any
                 * request whose final byte would leave the
                 * whitelisted factory-flash window.
                 *
                 * The subtraction form avoids address+length
                 * overflow.
                 */
                if (
                    requested > 0 &&
                    address >= flash_read_min &&
                    address < flash_read_end &&
                    requested <=
                        (uint32_t)(
                            flash_read_end -
                            address
                        )
                ) {
                    const volatile uint8_t *flash_ptr =
                        (const volatile uint8_t *)
                            (uintptr_t)address;

                    for (
                        uint8_t i = 0;
                        i < requested;
                        i++
                    ) {
                        data[7 + i] =
                            flash_ptr[i];
                    }

                    count = requested;
                    ok = true;
                }

                /*
                 * Preserve command byte and return explicit
                 * status/count/address metadata.
                 */
                data[1] = ok ? 0x55 : 0x0F;
                data[2] = count;

                data[3] =
                    (uint8_t)(address & 0xFF);

                data[4] =
                    (uint8_t)(
                        (address >> 8) &
                        0xFF
                    );

                data[5] =
                    (uint8_t)(
                        (address >> 16) &
                        0xFF
                    );

                data[6] =
                    (uint8_t)(
                        (address >> 24) &
                        0xFF
                    );

                /*
                 * Clear unused response payload bytes so
                 * stale host-request bytes are not echoed.
                 */
                for (
                    uint8_t i =
                        (uint8_t)(7 + count);
                    i < length;
                    i++
                ) {
                    data[i] = 0;
                }

                raw_hid_send(
                    data,
                    length
                );
            }

            return true;
        }

        /*
         * AL80_LCD_UART_RX_COMMAND_V1
         *
         * Host -> keyboard:
         *   data[0] = 0x44
         *
         * Keyboard -> host:
         *   data[0] = 0x44
         *   data[1] = number of UART bytes returned
         *   data[2..] = LCD UART RX bytes
         *
         * Maximum payload in a 32-byte Raw HID report:
         *   30 UART bytes.
         *
         * This command only READS the USART3 RX FIFO.
         * It sends nothing to the LCD.
         */
        case 0x44: {
            uint8_t max_read = 0;

            if (length > 2) {
                max_read = (uint8_t)(length - 2);
            }

            /*
             * Clear response area while preserving command byte.
             */
            for (
                uint8_t i = 1;
                i < length;
                i++
            ) {
                data[i] = 0;
            }

            uint8_t count =
                al80_screen_host_read(
                    data + 2,
                    max_read
                );

            data[1] = count;

            raw_hid_send(
                data,
                length
            );

            return true;
        }

        /*
         * AL80_VOLUME_HOST_COMMAND_V1
         *
         * Host -> keyboard:
         *   data[0] = 0x43
         *   data[1] = volume percentage 0..100
         *   data[2] = muted 0/1
         *
         * Response:
         *   data[3] = 0x55 accepted
         *             0x0F rejected
         */
        case 0x43: {
            bool ok = false;

            if (length >= 4) {
                uint8_t percent = data[1];

                if (percent > 100) {
                    percent = 100;
                }

                ok = al80_screen_show_volume(
                    percent,
                    data[2] != 0
                );

                data[3] =
                    ok ? 0x55 : 0x0F;

                raw_hid_send(
                    data,
                    length
                );
            }

            return true;
        }

        /*
         * AL80_LCD_RAW_HID_BRIDGE_V1
         *
         * Factory-derived host -> LCD bridge.
         *
         * Raw HID report layout:
         *
         *   [0]    command
         *   [1..2] stream offset (host-side bookkeeping)
         *   [3]    number of LCD bytes in this report
         *   [4..5] host transport checksum
         *   [6]    response status
         *   [7..]  raw LCD UART bytes
         *
         * Current AL80 QMK Raw HID report = 32 bytes, therefore
         * maximum LCD payload per report is 25 bytes.
         *
         * Response:
         *   data[6] = 0x55 success
         *   data[6] = 0x0F busy / rejected
         */
        // AL80_C9_SCREEN_PULSE_V1
        // Request bytes:
        //   [0] = 0x46
        //   [1] = 0xA6
        //   [2] = 0x59
        //   [3] = 0xC9
        // Response:
        //   [0] = 0x46
        //   [1] = 0x55 success / 0x0F blocked
        //   [2] = C9 state before pulse
        //   [3] = C9 state while LOW
        //   [4] = C9 state after restore HIGH
        //   [5] = 0xC9 signature
        /*
         * AL80_MATRIX_SCAN_RATE_COMMAND_V1
         *
         * Host -> keyboard:
         *   data[0] = 0x47
         *
         * Keyboard -> host:
         *   data[0] = 0x47
         *   data[1] = 0x55
         *   data[2..5] = matrix scan rate Hz, little-endian
         *
         * Read-only instrumentation.
         */
        /*
         * AL80_RGB_AB_COMMAND_V1
         *
         * data[0] = 0x48
         * data[1] = 0 -> RGB OFF, no EEPROM write
         * data[1] = 1 -> RGB ON,  no EEPROM write
         * data[1] = 2 -> query only
         *
         * Response:
         * data[0] = 0x48
         * data[1] = 0x55
         * data[2] = effective RGB enabled state
         */
        /*
         * AL80_CUSTOM_RGB_OVERLAY_AB_V1
         *
         * 0x49:
         *   data[1] = 0 -> bypass custom Snake/Heart overlay
         *   data[1] = 1 -> enable custom overlay
         *   data[1] = 2 -> query
         *
         * RGB Matrix itself remains enabled.
         * No EEPROM write.
         */
        /*
         * AL80_CREATOR_RGB_SCENE_V1
         *
         * Command 0x4A
         *
         * data[1] operation:
         *   0 = query
         *   1 = disable scene
         *   2 = clear staging to black
         *   3 = write staging chunk
         *   4 = commit staging -> active + enable
         *   5 = enable existing active scene
         *
         * WRITE CHUNK:
         *   data[2] = start LED
         *   data[3] = count, max 9
         *   data[4..] = R,G,B triples
         *
         * Response:
         *   [0] = 0x4A
         *   [1] = 0x55 success / 0x0F reject
         *   [2] = scene enabled
         *   [3] = LED count
         *   [4] = max LEDs per chunk
         *   [5] = operation
         *   [6] = echoed start
         *   [7] = echoed count
         *   [8] = RGB core enabled
         */
        /*
         * AL80_INPUT_ROUTER_V1
         *
         * Raw HID command 0x4B.
         *
         * data[1] op:
         *   0 = query capabilities/state
         *   1 = disable router
         *   2 = enable router
         *   3 = clear all bindings
         *   4 = set binding slot
         *   5 = get binding slot
         *   6 = restore safe default bindings
         *
         * SET BINDING:
         *   [2] slot
         *   [3] input event
         *   [4] trigger kind
         *   [5] trigger A
         *   [6] trigger B
         *   [7] action
         *   [8] flags (reserved, must be 0 in V1)
         *
         * Response:
         *   [0] 0x4B
         *   [1] 0x55 accepted / 0x0F rejected
         *   [2] router enabled
         *   [3] protocol version = 1
         *   [4] max binding slots = 12
         *   [5] max action id = 24
         *   [6] fallback default = 1
         *   [7] echoed op
         *   [8..] binding for GET/SET
         */
        case 0x4B: {
            uint8_t op =
                length >= 2 ? data[1] : 0;

            bool ok = false;
            al80_input_binding_t binding = {0};
            uint8_t slot = 0xFF;

            switch (op) {
                case 0:
                    ok = true;
                    break;

                case 1:
                    al80_input_router_enabled = false;
                    ok = true;
                    break;

                case 2:
                    al80_input_router_enabled = true;
                    ok = true;
                    break;

                case 3:
                    memset(
                        al80_input_bindings,
                        0,
                        sizeof(al80_input_bindings)
                    );
                    ok = true;
                    break;

                case 4:
                    if (length < 9) {
                        break;
                    }

                    slot = data[2];

                    if (
                        slot >= AL80_INPUT_BINDING_MAX ||
                        data[8] != 0
                    ) {
                        break;
                    }

                    binding.event = data[3];
                    binding.trigger = data[4];
                    binding.trigger_a = data[5];
                    binding.trigger_b = data[6];
                    binding.action = data[7];
                    binding.flags = data[8];

                    if (
                        !al80_input_binding_valid(
                            &binding
                        )
                    ) {
                        break;
                    }

                    al80_input_bindings[slot] =
                        binding;

                    ok = true;
                    break;

                case 5:
                    if (length < 3) {
                        break;
                    }

                    slot = data[2];

                    if (
                        slot >= AL80_INPUT_BINDING_MAX
                    ) {
                        break;
                    }

                    binding =
                        al80_input_bindings[slot];

                    ok = true;
                    break;

                case 6:
                    memset(
                        al80_input_bindings,
                        0,
                        sizeof(al80_input_bindings)
                    );

                    al80_input_bindings[0] =
                        (al80_input_binding_t) {
                            AL80_INPUT_EVENT_KNOB_CCW,
                            AL80_INPUT_TRIGGER_NONE,
                            0,
                            0,
                            AL80_INPUT_ACTION_VOLUME_DOWN,
                            0,
                        };

                    al80_input_bindings[1] =
                        (al80_input_binding_t) {
                            AL80_INPUT_EVENT_KNOB_CW,
                            AL80_INPUT_TRIGGER_NONE,
                            0,
                            0,
                            AL80_INPUT_ACTION_VOLUME_UP,
                            0,
                        };

                    al80_input_bindings[2] =
                        (al80_input_binding_t) {
                            AL80_INPUT_EVENT_KNOB_PRESS,
                            AL80_INPUT_TRIGGER_NONE,
                            0,
                            0,
                            AL80_INPUT_ACTION_MUTE,
                            0,
                        };

                    ok = true;
                    break;

                default:
                    break;
            }

            memset(data, 0, length);

            data[0] = 0x4B;
            data[1] = ok ? 0x55 : 0x0F;

            if (length >= 3) {
                data[2] =
                    al80_input_router_enabled ? 1 : 0;
            }

            if (length >= 4) {
                data[3] = 1;
            }

            if (length >= 5) {
                data[4] = AL80_INPUT_BINDING_MAX;
            }

            if (length >= 6) {
                data[5] = AL80_INPUT_ACTION_MAX;
            }

            if (length >= 7) {
                data[6] = 1;
            }

            if (length >= 8) {
                data[7] = op;
            }

            if (
                ok &&
                (op == 4 || op == 5) &&
                length >= 15
            ) {
                data[8] = slot;
                data[9] = binding.event;
                data[10] = binding.trigger;
                data[11] = binding.trigger_a;
                data[12] = binding.trigger_b;
                data[13] = binding.action;
                data[14] = binding.flags;
            }

            raw_hid_send(data, length);
            return true;
        }


        /*
         * AL80_LIVE_RGB_TELEMETRY_V1
         *
         * Request:
         *   [0] = 0x4D
         *   [1] = start LED (0..81)
         *
         * Response:
         *   [0] = 0x4D
         *   [1] = 0x55 OK / 0x0F error
         *   [2] = protocol version
         *   [3] = echoed start
         *   [4] = count (max 8)
         *   [5] = flags:
         *         bit0 RGB core enabled
         *         bit1 overlay enabled
         *         bit2 Creator Scene enabled
         *         bit3 low-battery safety frame
         *         bit4 frame valid/readable
         *   [6] = source (0 native/unknown, 1 Snake, 2 Creator, 3 safety)
         *   [7..] = count * RGB triples
         *
         * Query only. It does not alter RGB, overlay, Creator Scene,
         * EEPROM, flash, or any persistent keyboard state.
         */
        case 0x4D: {
#ifdef RGB_MATRIX_ENABLE
            uint8_t start = length >= 2 ? data[1] : 0;
            bool rgb_enabled = rgb_matrix_is_enabled();
            bool ok = start < AL80_CREATOR_LED_COUNT;
            uint8_t count = 0;

            if (ok) {
                uint8_t remaining =
                    (uint8_t)(AL80_CREATOR_LED_COUNT - start);

                count =
                    remaining > AL80_LIVE_RGB_TELEMETRY_CHUNK
                        ? AL80_LIVE_RGB_TELEMETRY_CHUNK
                        : remaining;
            }

            uint8_t flags = 0;

            if (rgb_enabled) {
                flags |= 0x01;
            }

            if (al80_custom_rgb_overlay_enabled) {
                flags |= 0x02;
            }

            if (al80_creator_scene_enabled) {
                flags |= 0x04;
            }

            if (al80_live_rgb_source == 3) {
                flags |= 0x08;
            }

            if (al80_live_rgb_frame_valid && rgb_enabled) {
                flags |= 0x10;
            }

            uint8_t source =
                rgb_enabled ? al80_live_rgb_source : 0;

            memset(data, 0, length);
            data[0] = 0x4D;
            data[1] = ok ? 0x55 : 0x0F;

            if (length >= 7) {
                data[2] = AL80_LIVE_RGB_TELEMETRY_VERSION;
                data[3] = start;
                data[4] = count;
                data[5] = flags;
                data[6] = source;
            }

            if (ok) {
                for (uint8_t i = 0; i < count; i++) {
                    uint8_t dst = (uint8_t)(7U + i * 3U);

                    if ((uint8_t)(dst + 2U) >= length) {
                        break;
                    }

                    data[dst + 0] =
                        al80_live_rgb_shadow[start + i][0];
                    data[dst + 1] =
                        al80_live_rgb_shadow[start + i][1];
                    data[dst + 2] =
                        al80_live_rgb_shadow[start + i][2];
                }
            }

            raw_hid_send(data, length);
#else
            memset(data, 0, length);
            data[0] = 0x4D;
            if (length >= 2) {
                data[1] = 0x0F;
            }
            raw_hid_send(data, length);
#endif
            return true;
        }

        case 0x4A: {
#ifdef RGB_MATRIX_ENABLE
            uint8_t op =
                length >= 2 ? data[1] : 0;

            bool ok = false;
            uint8_t echoed_start = 0;
            uint8_t echoed_count = 0;

            switch (op) {
                case 0:
                    ok = true;
                    break;

                case 1:
                    al80_creator_scene_enabled = false;
                    ok = true;
                    break;

                case 2:
                    memset(
                        al80_creator_scene_staging,
                        0,
                        sizeof(al80_creator_scene_staging)
                    );
                    ok = true;
                    break;

                case 3: {
                    if (length < 4) {
                        break;
                    }

                    uint8_t start = data[2];
                    uint8_t count = data[3];

                    uint8_t report_max =
                        (uint8_t)((length - 4U) / 3U);

                    if (
                        count == 0 ||
                        count > AL80_CREATOR_CHUNK_MAX ||
                        count > report_max ||
                        start >= AL80_CREATOR_LED_COUNT ||
                        count >
                            (uint8_t)(
                                AL80_CREATOR_LED_COUNT -
                                start
                            )
                    ) {
                        break;
                    }

                    for (
                        uint8_t i = 0;
                        i < count;
                        i++
                    ) {
                        uint8_t src =
                            (uint8_t)(4U + i * 3U);

                        al80_creator_scene_staging[
                            start + i
                        ][0] = data[src + 0];

                        al80_creator_scene_staging[
                            start + i
                        ][1] = data[src + 1];

                        al80_creator_scene_staging[
                            start + i
                        ][2] = data[src + 2];
                    }

                    echoed_start = start;
                    echoed_count = count;
                    ok = true;
                    break;
                }

                case 4:
                    memcpy(
                        al80_creator_scene_active,
                        al80_creator_scene_staging,
                        sizeof(al80_creator_scene_active)
                    );

                    al80_creator_scene_enabled = true;
                    ok = true;
                    break;

                case 5:
                    al80_creator_scene_enabled = true;
                    ok = true;
                    break;

                default:
                    break;
            }

            memset(data, 0, length);

            data[0] = 0x4A;
            data[1] = ok ? 0x55 : 0x0F;

            if (length >= 3) {
                data[2] =
                    al80_creator_scene_enabled ? 1 : 0;
            }

            if (length >= 4) {
                data[3] = AL80_CREATOR_LED_COUNT;
            }

            if (length >= 5) {
                data[4] = AL80_CREATOR_CHUNK_MAX;
            }

            if (length >= 6) {
                data[5] = op;
            }

            if (length >= 7) {
                data[6] = echoed_start;
            }

            if (length >= 8) {
                data[7] = echoed_count;
            }

            if (length >= 9) {
                data[8] =
                    rgb_matrix_is_enabled() ? 1 : 0;
            }

            raw_hid_send(data, length);
#else
            memset(data, 0, length);
            data[0] = 0x4A;

            if (length >= 2) {
                data[1] = 0x0F;
            }

            raw_hid_send(data, length);
#endif
            return true;
        }

        case 0x49: {
#ifdef RGB_MATRIX_ENABLE
            uint8_t op =
                length >= 2 ? data[1] : 2;

            if (op == 0) {
                al80_custom_rgb_overlay_enabled = false;
            } else if (op == 1) {
                al80_custom_rgb_overlay_enabled = true;
            } else if (op != 2) {
                memset(data, 0, length);
                data[0] = 0x49;
                data[1] = 0x0F;
                raw_hid_send(data, length);
                return true;
            }

            bool enabled =
                al80_custom_rgb_overlay_enabled;

            memset(data, 0, length);

            data[0] = 0x49;
            data[1] = 0x55;
            data[2] = enabled ? 1 : 0;
            data[3] =
                rgb_matrix_is_enabled() ? 1 : 0;

            raw_hid_send(data, length);
#else
            memset(data, 0, length);
            data[0] = 0x49;
            data[1] = 0x0F;
            raw_hid_send(data, length);
#endif
            return true;
        }

        case 0x48: {
#ifdef RGB_MATRIX_ENABLE
            uint8_t op = length >= 2 ? data[1] : 2;

            if (op == 0) {
                rgb_matrix_disable_noeeprom();
            } else if (op == 1) {
                rgb_matrix_enable_noeeprom();
            } else if (op != 2) {
                memset(data, 0, length);
                data[0] = 0x48;
                data[1] = 0x0F;
                raw_hid_send(data, length);
                return true;
            }

            uint8_t enabled =
                rgb_matrix_is_enabled() ? 1 : 0;

            memset(data, 0, length);
            data[0] = 0x48;
            data[1] = 0x55;
            data[2] = enabled;

            raw_hid_send(data, length);
#else
            memset(data, 0, length);
            data[0] = 0x48;
            data[1] = 0x0F;
            data[2] = 0;
            raw_hid_send(data, length);
#endif
            return true;
        }

        case 0x47: {
            uint32_t rate =
                al80_matrix_scan_rate_hz;

            memset(data, 0, length);

            data[0] = 0x47;
            data[1] = 0x55;
            data[2] = (uint8_t)(rate & 0xFFU);
            data[3] = (uint8_t)((rate >> 8) & 0xFFU);
            data[4] = (uint8_t)((rate >> 16) & 0xFFU);
            data[5] = (uint8_t)((rate >> 24) & 0xFFU);

            raw_hid_send(data, length);
            return true;
        }

        case 0x46: {
            bool armed =
                length >= 6 &&
                data[1] == 0xA6 &&
                data[2] == 0x59 &&
                data[3] == 0xC9;

            if (!armed) {
                memset(data, 0, length);
                data[0] = 0x46;
                if (length >= 2) {
                    data[1] = 0x0F;
                }
                raw_hid_send(data, length);
                return true;
            }

            bool before = gpio_read_pin(C9);

            gpio_write_pin_low(C9);
            wait_ms(100);

            bool during_low = gpio_read_pin(C9);

            gpio_write_pin_high(C9);
            wait_ms(20);

            bool after = gpio_read_pin(C9);

            memset(data, 0, length);
            data[0] = 0x46;
            data[1] = 0x55;
            data[2] = before ? 1 : 0;
            data[3] = during_low ? 1 : 0;
            data[4] = after ? 1 : 0;
            data[5] = 0xC9;

            raw_hid_send(data, length);
            return true;
        }

        case 0x40:
        case 0x41:
        case 0x42: {
            if (length < 7) {
                return true;
            }

            bool ok = false;

            if (data[0] == 0x42) {
                /*
                 * FINISH carries no LCD payload.
                 */
                ok = al80_screen_host_end();
            } else {
                uint8_t data_len = data[3];

                /*
                 * Critical bounds gate:
                 *
                 * Never allow the host-supplied length field to
                 * read past the current Raw HID report.
                 */
                if (
                    data_len <=
                    (uint8_t)(length - 7)
                ) {
                    if (data[0] == 0x40) {
                        ok =
                            al80_screen_host_begin(
                                data + 7,
                                data_len
                            );
                    } else {
                        ok =
                            al80_screen_host_data(
                                data + 7,
                                data_len
                            );
                    }
                }
            }

            data[6] = ok ? 0x55 : 0x0F;

            raw_hid_send(
                data,
                length
            );

            return true;
        }

        default:
            return false;
    }
}
#endif

#ifdef RGB_MATRIX_ENABLE
bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {
    if (
        !al80_creator_scene_enabled &&
        !al80_custom_rgb_overlay_enabled
    ) {
        al80_live_rgb_frame_valid = false;
        al80_live_rgb_source = 0;
        return true;
    }

    static const uint8_t heart_leds[] = {
        23, 26,
        37, 38, 39, 40, 41,
        52, 53, 54, 55,
        65, 66, 67,
        74
    };

    static const uint8_t heart_orders[] = {
         1,  0,
         2,  3,  4,  5,  6,
        10,  9,  8,  7,
        11, 12, 13,
        14
    };

    static const uint8_t snake_path[] = {
         0,  1,  2,  3,  4,  5,  6,
         7,  8,  9, 10, 11, 12, 13,

        28, 27, 26, 25, 24, 23, 22, 21,
        20, 19, 18, 17, 16, 15, 14,

        29, 30, 31, 32, 33, 34, 35, 36,
        37, 38, 39, 40, 41, 42, 43,

        56, 55, 54, 53, 52, 51, 50,
        49, 48, 47, 46, 45, 44,

        57, 58, 59, 60, 61, 62, 63,
        64, 65, 66, 67, 68, 69,

        70, 71, 72, 73, 74, 75,
        79, 80, 81
    };

    static const int8_t snake_heart_order[] = {
        -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1,

        -1, -1,  0, -1, -1,  1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1,

        -1, -1, -1, -1, -1, -1, -1, -1,
         2,  3,  4,  5,  6, -1, -1,

        -1,  7,  8,  9, 10, -1, -1,
        -1, -1, -1, -1, -1, -1,

        -1, -1, -1, -1, -1, -1, -1,
        -1, 11, 12, 13, -1, -1,

        -1, -1, -1, -1, 14, -1,
        -1, -1, -1
    };

    const uint16_t path_count =
        sizeof(snake_path) / sizeof(snake_path[0]);

    const uint8_t heart_count =
        sizeof(heart_leds) / sizeof(heart_leds[0]);

    const uint16_t step_ms = 180;
    const uint16_t heart_pause_ms = 1000;
    const uint16_t end_pause_ms = 1000;
    const uint8_t base_length = 4;

    const uint32_t travel_duration =
        ((uint32_t)path_count * step_ms) +
        ((uint32_t)heart_count * heart_pause_ms);

    const uint32_t phase_duration =
        travel_duration + end_pause_ms;

    const uint32_t full_cycle =
        phase_duration * 2UL;

    static bool cache_valid = false;
    static bool cached_low_battery_red = false;
    static bool cached_rebuild_phase = false;
    static uint16_t cached_head_pos = 0;
    static uint8_t cached_processed_count = 0;

    if (led_min == 0 || !cache_valid) {
        uint8_t low_battery_percent = 0;

        bool low_battery_valid =
            al80_screen_real_battery_percent(
                &low_battery_percent
            );

        bool usb_present =
            gpio_read_pin(B9);

        cached_low_battery_red = false;

        if (
            low_battery_valid &&
            !usb_present &&
            low_battery_percent <= 15
        ) {
            uint16_t blink_ms =
                (low_battery_percent <= 5)
                    ? 300
                    : 700;

            cached_low_battery_red =
                ((timer_read32() / blink_ms) & 1U) == 0;
        }

        if (!cached_low_battery_red) {
            uint32_t now = timer_read32();
            uint32_t cycle_time = now % full_cycle;

            cached_rebuild_phase =
                cycle_time >= phase_duration;

            uint32_t phase_time =
                cached_rebuild_phase
                    ? cycle_time - phase_duration
                    : cycle_time;

            uint32_t accumulated = 0;
            uint16_t head_pos = 0;
            uint8_t processed_count = 0;

            for (
                uint16_t pidx = 0;
                pidx < path_count;
                pidx++
            ) {
                int8_t heart_order =
                    snake_heart_order[pidx];

                if (
                    phase_time <
                    accumulated + step_ms
                ) {
                    head_pos = pidx;
                    break;
                }

                accumulated += step_ms;
                head_pos = pidx;

                if (heart_order >= 0) {
                    if (
                        phase_time <
                        accumulated + heart_pause_ms
                    ) {
                        break;
                    }

                    accumulated += heart_pause_ms;
                    processed_count++;
                }

                if (pidx == path_count - 1) {
                    head_pos = path_count - 1;
                }
            }

            if (phase_time >= travel_duration) {
                processed_count = heart_count;
                head_pos = path_count - 1;
            }

            cached_head_pos = head_pos;
            cached_processed_count = processed_count;
        }

        cache_valid = true;
    }

    if (cached_low_battery_red) {
        al80_live_rgb_frame_valid = true;
        al80_live_rgb_source = 3;
        for (
            uint8_t i = led_min;
            i < led_max;
            i++
        ) {
            al80_live_rgb_set_color(
                i,
                255, 0, 0
            );
        }

        return false;
    }

    /*
     * Creator Scene overrides Snake/Heart, while the
     * low-battery red safety state above remains highest.
     */
    if (al80_creator_scene_enabled) {
        al80_live_rgb_frame_valid = true;
        al80_live_rgb_source = 2;
        for (
            uint8_t i = led_min;
            i < led_max;
            i++
        ) {
            al80_live_rgb_set_color(
                i,
                al80_creator_scene_active[i][0],
                al80_creator_scene_active[i][1],
                al80_creator_scene_active[i][2]
            );
        }

        return false;
    }

    al80_live_rgb_frame_valid = true;
    al80_live_rgb_source = 1;

    for (
        uint8_t i = led_min;
        i < led_max;
        i++
    ) {
        al80_live_rgb_set_color(
            i,
            255, 255, 255
        );
    }

    for (
        uint8_t h = 0;
        h < heart_count;
        h++
    ) {
        uint8_t heart_led =
            heart_leds[h];

        uint8_t order =
            heart_orders[h];

        bool should_exist;

        if (!cached_rebuild_phase) {
            should_exist =
                order >= cached_processed_count;
        } else {
            should_exist =
                order < cached_processed_count;
        }

        if (
            should_exist &&
            heart_led >= led_min &&
            heart_led < led_max
        ) {
            al80_live_rgb_set_color(
                heart_led,
                255, 0, 0
            );
        }
    }

    uint8_t snake_length;

    if (!cached_rebuild_phase) {
        snake_length =
            base_length +
            cached_processed_count;
    } else {
        snake_length =
            base_length +
            heart_count -
            cached_processed_count;
    }

    for (
        uint8_t segment = 0;
        segment < snake_length;
        segment++
    ) {
        if (segment > cached_head_pos) {
            break;
        }

        uint16_t body_pos =
            cached_head_pos - segment;

        uint8_t led =
            snake_path[body_pos];

        if (
            led < led_min ||
            led >= led_max
        ) {
            continue;
        }

        if (segment == 0) {
            al80_live_rgb_set_color(
                led,
                180, 255, 0
            );
        } else {
            al80_live_rgb_set_color(
                led,
                0, 255, 0
            );
        }
    }

    return false;
}
#endif


#ifdef RGB_MATRIX_ENABLE
void keyboard_post_init_user(void) {
    rgb_matrix_enable_noeeprom();
    rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);

    /*
     * Global brightness/value = 255.
     * Actual colors are overwritten by the indicator callback.
     */
    rgb_matrix_sethsv_noeeprom(0, 0, 255);
}
#endif

