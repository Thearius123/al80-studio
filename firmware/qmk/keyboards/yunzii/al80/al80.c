/* Copyright 2022 Jacky
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
#include "quantum.h"
#include "al80_smartble.h"
#include "al80_screen.h"
#ifdef RGB_MATRIX_ENABLE
#include "aw20216s.h"
#include "al80_battery.h"
#endif



void keyboard_post_init_kb(void)
 {
    AFIO->MAPR = (AFIO->MAPR & ~AFIO_MAPR_SWJ_CFG_Msk);
    AFIO->MAPR|= AFIO_MAPR_SWJ_CFG_DISABLE;
    gpio_set_pin_output(A8);
    gpio_write_pin_high(A8);
    gpio_set_pin_output(C9);
    gpio_write_pin_high(C9);
    gpio_set_pin_input(B9);

    al80_smartble_init();
    al80_screen_init();
    keyboard_post_init_user();
}

// --- AL80 AW20216S LED MAP BEGIN ---

#ifdef RGB_MATRIX_ENABLE
const aw20216s_led_t PROGMEM g_aw20216s_leds[RGB_MATRIX_LED_COUNT] = {
    {0, SW1_CS1, SW1_CS2, SW1_CS3},    //14   esc
    {0, SW2_CS1, SW2_CS2, SW2_CS3},
    {0, SW3_CS1, SW3_CS2, SW3_CS3},
    {0, SW4_CS1, SW4_CS2, SW4_CS3},
    {0, SW5_CS1, SW5_CS2, SW5_CS3},
    {0, SW6_CS1, SW6_CS2, SW6_CS3},
    {0, SW7_CS1, SW7_CS2, SW7_CS3},
    {0, SW8_CS1, SW8_CS2, SW8_CS3},
    {1, SW1_CS1, SW1_CS2, SW1_CS3},
    {1, SW2_CS1, SW2_CS2, SW2_CS3},
    {1, SW3_CS1, SW3_CS2, SW3_CS3},
    {1, SW4_CS1, SW4_CS2, SW4_CS3},
    {1, SW5_CS1, SW5_CS2, SW5_CS3},
    {1, SW6_CS1, SW6_CS2, SW6_CS3},
    {0, SW1_CS4, SW1_CS5, SW1_CS6},    //15  ~
    {0, SW2_CS4, SW2_CS5, SW2_CS6},
    {0, SW3_CS4, SW3_CS5, SW3_CS6},
    {0, SW4_CS4, SW4_CS5, SW4_CS6},
    {0, SW5_CS4, SW5_CS5, SW5_CS6},
    {0, SW6_CS4, SW6_CS5, SW6_CS6},
    {0, SW7_CS4, SW7_CS5, SW7_CS6},
    {0, SW8_CS4, SW8_CS5, SW8_CS6},
    {1, SW1_CS4, SW1_CS5, SW1_CS6},
    {1, SW2_CS4, SW2_CS5, SW2_CS6},
    {1, SW3_CS4, SW3_CS5, SW3_CS6},
    {1, SW4_CS4, SW4_CS5, SW4_CS6},
    {1, SW5_CS4, SW5_CS5, SW5_CS6},
    {1, SW6_CS4, SW6_CS5, SW6_CS6},
    {1, SW7_CS4, SW7_CS5, SW7_CS6},
    {0, SW1_CS7, SW1_CS8, SW1_CS9},    //15   tab
    {0, SW2_CS7, SW2_CS8, SW2_CS9},
    {0, SW3_CS7, SW3_CS8, SW3_CS9},
    {0, SW4_CS7, SW4_CS8, SW4_CS9},
    {0, SW5_CS7, SW5_CS8, SW5_CS9},
    {0, SW6_CS7, SW6_CS8, SW6_CS9},
    {0, SW7_CS7, SW7_CS8, SW7_CS9},
    {0, SW8_CS7, SW8_CS8, SW8_CS9},
    {1, SW1_CS7, SW1_CS8, SW1_CS9},
    {1, SW2_CS7, SW2_CS8, SW2_CS9},
    {1, SW3_CS7, SW3_CS8, SW3_CS9},
    {1, SW4_CS7, SW4_CS8, SW4_CS9},
    {1, SW5_CS7, SW5_CS8, SW5_CS9},
    {1, SW6_CS7, SW6_CS8, SW6_CS9},
    {1, SW7_CS7, SW7_CS8, SW7_CS9},
    {0, SW1_CS10, SW1_CS11, SW1_CS12}, //14   caps
    {0, SW2_CS10, SW2_CS11, SW2_CS12},
    {0, SW3_CS10, SW3_CS11, SW3_CS12},
    {0, SW4_CS10, SW4_CS11, SW4_CS12},
    {0, SW5_CS10, SW5_CS11, SW5_CS12},
    {0, SW6_CS10, SW6_CS11, SW6_CS12},
    {0, SW7_CS10, SW7_CS11, SW7_CS12},
    {0, SW8_CS10, SW8_CS11, SW8_CS12},
    {1, SW1_CS10, SW1_CS11, SW1_CS12},
    {1, SW2_CS10, SW2_CS11, SW2_CS12},
    {1, SW3_CS10, SW3_CS11, SW3_CS12},
    {1, SW4_CS10, SW4_CS11, SW4_CS12},
    {1, SW6_CS10, SW6_CS11, SW6_CS12},
    {0, SW1_CS13, SW1_CS14, SW1_CS15},  //lshift  14
    {0, SW3_CS13, SW3_CS14, SW3_CS15},
    {0, SW4_CS13, SW4_CS14, SW4_CS15},
    {0, SW5_CS13, SW5_CS14, SW5_CS15},
    {0, SW6_CS13, SW6_CS14, SW6_CS15},
    {0, SW7_CS13, SW7_CS14, SW7_CS15},
    {0, SW8_CS13, SW8_CS14, SW8_CS15},
    {1, SW1_CS13, SW1_CS14, SW1_CS15},
    {1, SW2_CS13, SW2_CS14, SW2_CS15},
    {1, SW3_CS13, SW3_CS14, SW3_CS15},
    {1, SW4_CS13, SW4_CS14, SW4_CS15},
    {1, SW5_CS13, SW5_CS14, SW5_CS15},
    {1, SW6_CS13, SW6_CS14, SW6_CS15},
    {0, SW1_CS16, SW1_CS17, SW1_CS18}, //12   lctrl
    {0, SW2_CS16, SW2_CS17, SW2_CS18},
    {0, SW3_CS16, SW3_CS17, SW3_CS18},
    {0, SW7_CS16, SW7_CS17, SW7_CS18},
    {0, SW8_CS16, SW8_CS17, SW8_CS18},
    {1, SW1_CS16, SW1_CS17, SW1_CS18},
    {1, SW2_CS16, SW2_CS17, SW2_CS18},
    {1, SW3_CS16, SW3_CS17, SW3_CS18},
    {1, SW4_CS16, SW4_CS17, SW4_CS18},
    {1, SW5_CS16, SW5_CS17, SW5_CS18},
    {1, SW6_CS16, SW6_CS17, SW6_CS18},
    {1, SW7_CS16, SW7_CS17, SW7_CS18}
};
#endif // RGB_MATRIX_ENABLE

// --- AL80 AW20216S LED MAP END ---


void housekeeping_task_kb(void) {
    al80_smartble_task();
    al80_screen_task();
    housekeeping_task_user();
}
