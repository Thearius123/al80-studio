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


#pragma once

/*
 * YUNZII AL80 / MK856
 * Recovered AW20216S hardware configuration.
 */

#define RGB_MATRIX_LED_COUNT 82

/* SPI1 */
#define SPI_DRIVER SPID1
#define SPI_SCK_PIN A5
#define SPI_MOSI_PIN A7
#define SPI_MISO_PIN A6

/* Dual AW20216S RGB drivers */
#define AW20216S_CS_PIN_1 B6
#define AW20216S_CS_PIN_2 C8
#define AW20216S_EN_PIN B7

/* Vendor AL80 configuration used SPI mode 3. */
#define AW20216S_SPI_MODE 3

/*
 * AL80 SmartBLE wireless MCU
 * Factory firmware recovered configuration:
 * USART1 / PA9 TX / PA10 RX / 460800 baud
 */
/* AL80 encoder rests with PC6=HIGH, PC7=HIGH. */
#define ENCODER_DEFAULT_POS 0x3

#define UART_DRIVER SD1
#define UART_TX_PIN A9
#define UART_RX_PIN A10

#define DEBOUNCE 3
