
MCU_LDSCRIPT = STM32F103xB
ENCODER_ENABLE = yes




SPI_DRIVER_REQUIRED = yes
UART_DRIVER_REQUIRED = yes
SRC += al80_smartble.c
SRC += al80_screen.c
SRC += al80_battery.c

DEBOUNCE_TYPE = asym_eager_defer_pk
