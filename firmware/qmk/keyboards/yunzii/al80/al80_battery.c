#include "quantum.h"
#include "hal.h"
#include "al80_battery.h"
#include "eeconfig.h"

/*
 * AL80 factory-derived battery measurement.
 *
 * STM32F103:
 *   ADC1 channel 9  = PB1 = battery sense
 *   ADC1 channel 17 = VREFINT
 *
 * Factory behavior:
 *   - 239.5 cycle sample time
 *   - 10 samples
 *   - sort
 *   - discard lowest + highest
 *   - average middle 8
 *   - battery_value = adc9 * 1764 / vref17
 */

#define AL80_ADC_SAMPLE_COUNT 10

/*
 * Hard upper bound for STM32F1 ADC hardware waits.
 *
 * These are iteration-based guards, not timing-critical delays.
 * Their purpose is solely to guarantee that an ADC hardware fault
 * can never trap the keyboard firmware forever.
 */
#define AL80_ADC_CAL_TIMEOUT  100000UL
#define AL80_ADC_EOC_TIMEOUT  100000UL

static bool al80_adc_initialized = false;

static void al80_adc_delay(volatile uint32_t count) {
    while (count--) {
        __asm__ volatile("nop");
    }
}

static void al80_adc_invalidate(void) {
    /*
     * Force the next measurement to perform full ADC setup again.
     * Do not block keyboard operation if ADC1 ever enters a bad state.
     */
    al80_adc_initialized = false;

    ADC1->CR1 = 0;
    ADC1->CR2 = 0;
}

static bool al80_adc_wait_clear(
    volatile uint32_t *reg,
    uint32_t mask,
    uint32_t timeout
) {
    while ((*reg & mask) != 0U) {
        if (timeout-- == 0U) {
            return false;
        }
    }

    return true;
}

static bool al80_adc_init_once(void) {
    if (al80_adc_initialized) {
        return true;
    }

    /*
     * PB1 analog input.
     */
    palSetPadMode(GPIOB, 1, PAL_MODE_INPUT_ANALOG);

    /*
     * Factory-derived ADC clock:
     * ADCPRE = PCLK2 / 8.
     */
    RCC->APB2ENR |= RCC_APB2ENR_ADC1EN;

    RCC->CFGR &= ~RCC_CFGR_ADCPRE;
    RCC->CFGR |= RCC_CFGR_ADCPRE_DIV8;

    ADC1->CR1 = 0;
    ADC1->CR2 = 0;

    /*
     * Factory sampling time:
     * channel 9 and VREFINT/channel 17 = 239.5 cycles.
     */
    ADC1->SMPR2 &= ~(7U << 27);
    ADC1->SMPR2 |=  (7U << 27);

    ADC1->SMPR1 &= ~(7U << 21);
    ADC1->SMPR1 |=  (7U << 21);

    /*
     * Enable VREFINT internal path.
     */
    ADC1->CR2 |= ADC_CR2_TSVREFE;

    /*
     * Power ADC.
     */
    ADC1->CR2 |= ADC_CR2_ADON;

    al80_adc_delay(1000);

    /*
     * Calibration reset — bounded.
     */
    ADC1->CR2 |= ADC_CR2_RSTCAL;

    if (!al80_adc_wait_clear(
            &ADC1->CR2,
            ADC_CR2_RSTCAL,
            AL80_ADC_CAL_TIMEOUT
        )) {
        al80_adc_invalidate();
        return false;
    }

    /*
     * Calibration — bounded.
     */
    ADC1->CR2 |= ADC_CR2_CAL;

    if (!al80_adc_wait_clear(
            &ADC1->CR2,
            ADC_CR2_CAL,
            AL80_ADC_CAL_TIMEOUT
        )) {
        al80_adc_invalidate();
        return false;
    }

    al80_adc_initialized = true;
    return true;
}

static bool al80_adc_read_channel(
    uint8_t channel,
    uint16_t *value
) {
    if (!value) {
        return false;
    }

    ADC1->SQR1 = 0;
    ADC1->SQR2 = 0;
    ADC1->SQR3 = channel & 0x1F;

    ADC1->SR = 0;

    /*
     * STM32F1: second ADON write starts conversion.
     */
    ADC1->CR2 |= ADC_CR2_ADON;

    uint32_t timeout = AL80_ADC_EOC_TIMEOUT;

    while (!(ADC1->SR & ADC_SR_EOC)) {
        if (timeout-- == 0U) {
            al80_adc_invalidate();
            return false;
        }
    }

    *value =
        (uint16_t)(ADC1->DR & 0x0FFFU);

    return true;
}

static void al80_sort_u16(
    uint16_t *v,
    uint8_t n
) {
    for (uint8_t i = 0; i < n - 1; i++) {
        for (uint8_t j = i + 1; j < n; j++) {
            if (v[j] < v[i]) {
                uint16_t t = v[i];
                v[i] = v[j];
                v[j] = t;
            }
        }
    }
}

static bool al80_adc_filtered(
    uint8_t channel,
    uint16_t *result
) {
    if (!result) {
        return false;
    }

    uint16_t samples[AL80_ADC_SAMPLE_COUNT];

    for (
        uint8_t i = 0;
        i < AL80_ADC_SAMPLE_COUNT;
        i++
    ) {
        if (!al80_adc_read_channel(
                channel,
                &samples[i]
            )) {
            return false;
        }
    }

    al80_sort_u16(
        samples,
        AL80_ADC_SAMPLE_COUNT
    );

    uint32_t sum = 0;

    /*
     * Exact factory filter:
     * discard minimum and maximum,
     * average sorted samples 1..8.
     */
    for (uint8_t i = 1; i < 9; i++) {
        sum += samples[i];
    }

    *result =
        (uint16_t)(sum / 8U);

    return true;
}

static uint8_t al80_battery_percent(uint16_t v) {
    /*
     * Exact piecewise mapping reconstructed from AL80 factory firmware.
     */

    if (v > 4150) {
        return 100;
    }

    /*
     * Factory special case:
     * values below 1000 are treated as 100.
     */
    if (v < 1000) {
        return 100;
    }

    if (v < 3100) {
        return 0;
    }

    if (v <= 3230) {
        return (uint8_t)((v - 3100U) / 26U);
    }

    if (v <= 3360) {
        return (uint8_t)(((v - 3230U) / 26U) + 5U);
    }

    if (v < 3630) {
        return (uint8_t)(((v - 3360U) / 9U) + 10U);
    }

    if (v <= 3760) {
        return (uint8_t)(
            (((uint32_t)(v - 3630U) * 20U) / 130U) + 40U
        );
    }

    if (v <= 3930) {
        return (uint8_t)(
            (((uint32_t)(v - 3760U) * 20U) / 170U) + 60U
        );
    }

    if (v <= 3980) {
        return (uint8_t)(((v - 3930U) / 10U) + 80U);
    }

    return (uint8_t)(
        (((uint32_t)(v - 3980U) * 14U) / 170U) + 85U
    );
}

void al80_battery_init(void) {
    /*
     * Best-effort initialization.
     *
     * A failure here must never prevent the rest of the keyboard
     * from starting. al80_battery_measure() retries later.
     */
    (void)al80_adc_init_once();
}

bool al80_battery_measure(
    al80_battery_sample_t *out
) {
    if (!out) {
        return false;
    }

    memset(out, 0, sizeof(*out));

    if (!al80_adc_init_once()) {
        return false;
    }

    uint16_t adc9 = 0;
    uint16_t vref = 0;

    if (!al80_adc_filtered(9, &adc9)) {
        return false;
    }

    if (!al80_adc_filtered(17, &vref)) {
        return false;
    }

    out->adc9_raw   = adc9;
    out->vref17_raw = vref;

    if (vref == 0U) {
        return false;
    }

    uint32_t scaled =
        ((uint32_t)adc9 * 1764U) /
        (uint32_t)vref;

    if (scaled > 0xFFFFU) {
        return false;
    }

    out->battery_value =
        (uint16_t)scaled;

    uint8_t percent =
        al80_battery_percent(
            out->battery_value
        );

    /*
     * Defensive clamp.
     * Factory mapping should already produce <= 100.
     */
    if (percent > 100U) {
        percent = 100U;
    }

    out->percent = percent;
    out->valid   = true;

    return true;
}


/*
 * Persistent last-known REAL battery percentage.
 *
 * Layout:
 *
 * bits 31..24 = signature 0xA8
 * bits 23..16 = inverse signature 0x57
 * bits 15..8  = percentage
 * bits 7..0   = inverse percentage
 *
 * Only battery-side ADC measurements are stored.
 * USB-side invalid ADC9 readings are never persisted.
 */

#define AL80_BATTERY_EE_SIG     0xA8U
#define AL80_BATTERY_EE_SIG_INV 0x57U

bool al80_battery_saved_percent_load(
    uint8_t *percent
) {
    if (!percent) {
        return false;
    }

    uint32_t raw = eeconfig_read_kb();

    uint8_t sig =
        (uint8_t)(raw >> 24);

    uint8_t sig_inv =
        (uint8_t)(raw >> 16);

    uint8_t value =
        (uint8_t)(raw >> 8);

    uint8_t value_inv =
        (uint8_t)raw;

    if (
        sig != AL80_BATTERY_EE_SIG ||
        sig_inv != AL80_BATTERY_EE_SIG_INV
    ) {
        return false;
    }

    if (
        (uint8_t)(value ^ value_inv) != 0xFFU ||
        value > 100
    ) {
        return false;
    }

    *percent = value;
    return true;
}

void al80_battery_saved_percent_store(
    uint8_t percent
) {
    if (percent > 100) {
        return;
    }

    uint8_t previous = 0;

    if (
        al80_battery_saved_percent_load(&previous) &&
        previous == percent
    ) {
        /*
         * Avoid unnecessary EEPROM writes.
         */
        return;
    }

    uint32_t raw =
        ((uint32_t)AL80_BATTERY_EE_SIG << 24) |
        ((uint32_t)AL80_BATTERY_EE_SIG_INV << 16) |
        ((uint32_t)percent << 8) |
        (uint32_t)((uint8_t)~percent);

    eeconfig_update_kb(raw);
}
