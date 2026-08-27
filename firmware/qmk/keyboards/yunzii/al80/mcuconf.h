// Copyright 2026
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include_next <mcuconf.h>

#undef STM32_SPI_USE_SPI1
#define STM32_SPI_USE_SPI1 TRUE

/* AL80 SmartBLE UART */
#undef STM32_SERIAL_USE_USART1
#define STM32_SERIAL_USE_USART1 TRUE

/*
 * AL80 screen UART.
 * Factory-derived USART3 @ 921600 baud.
 */
#undef STM32_SERIAL_USE_USART3
#define STM32_SERIAL_USE_USART3 TRUE
