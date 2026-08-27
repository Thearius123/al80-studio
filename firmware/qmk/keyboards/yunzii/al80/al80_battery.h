#pragma once

#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint16_t adc9_raw;
    uint16_t vref17_raw;
    uint16_t battery_value;
    uint8_t  percent;
    bool     valid;
} al80_battery_sample_t;

void al80_battery_init(void);
bool al80_battery_measure(al80_battery_sample_t *out);

bool al80_battery_saved_percent_load(uint8_t *percent);
void al80_battery_saved_percent_store(uint8_t percent);
