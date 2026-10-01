/*
 * @file shbt_isomer_battery_mmio.h
 * @brief C11 Hardware Register Extensions for Coherent Graser Nuclear Isomer Battery
 * @details Target Base Address: 0x70000000 (Physical MMIO Contract, 128 Bytes Total)
 *          Repository Alignment: sys1own/shbt-warp, sys1own/shbt-ghost
 */

#ifndef SHBT_ISOMER_BATTERY_MMIO_H
#define SHBT_ISOMER_BATTERY_MMIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stdalign.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_ISOMER_MMIO_BASE_ADDR   (0x70000000ULL)
#define SHBT_ISOMER_MMIO_TOTAL_SIZE  (128UL)

/**
 * @brief Hardware Status and Interlock Bitfield Flags
 */
typedef enum {
    SHBT_ISOMER_STAT_READY           = (1U << 0),  /**< Battery armed and within cryogenic limits */
    SHBT_ISOMER_STAT_BORRMANN_LOCKED = (1U << 1),  /**< Crystal superlattice Bragg angle aligned */
    SHBT_ISOMER_STAT_CROWBAR_ARMED   = (1U << 2),  /**< Sub-2.5ns PCSS interlock operational */
    SHBT_ISOMER_STAT_CROWBAR_TRIPPED = (1U << 3),  /**< Crowbar diverted bus due to overvoltage */
    SHBT_ISOMER_STAT_CRYO_WARNING    = (1U << 4),  /**< Delta-T headroom < 11.79 K */
    SHBT_ISOMER_STAT_QUENCH_FAULT    = (1U << 5),  /**< Superconducting bus quench detected */
    SHBT_ISOMER_STAT_LASING_ACTIVE   = (1U << 6),  /**< Graser superradiant discharge in progress */
    SHBT_ISOMER_STAT_DARK_SINK_SYNC  = (1U << 7)   /**< Stinespring dark ledger synchronized */
} shbt_isomer_status_flags_t;

/**
 * @brief Memory-Mapped Register Layout for 0x70000000 (Exact 128-Byte Structural Contract)
 */
typedef struct {
    /* 0x00 - 0x07: State-of-Charge & Primary Energy Storage (Joules) */
    _Alignas(64) volatile uint64_t energy_remaining_joules;

    /* 0x08 - 0x0F: Absolute State-of-Charge (Fixed-Point 32.32, 1.0 == 100.0%) */
    volatile uint64_t state_of_charge_q32;

    /* 0x10 - 0x17: Instantaneous Output Voltage (Microvolts, 15 kV - 400 kV DC) */
    volatile uint64_t bus_voltage_uv;

    /* 0x18 - 0x1F: Instantaneous Output Current (Microamperes) */
    volatile uint64_t bus_current_ua;

    /* 0x20 - 0x27: Real-Time Core Thermal Sensor (Microkelvin, Base: 21130000 uK) */
    volatile uint64_t core_temperature_uk;

    /* 0x28 - 0x2F: Cryogenic Temperature Headroom (Microkelvin, Min: 11790000 uK) */
    volatile uint64_t cryo_headroom_uk;

    /* 0x30 - 0x33: Trigger Laser Delay / Seed Timing (Picoseconds) */
    volatile uint32_t trigger_delay_ps;
    /* 0x34 - 0x37: Trigger Seed Pulse Width (Picoseconds) */
    volatile uint32_t trigger_pulse_width_ps;

    /* 0x38 - 0x3F: Graser Coherent Radiant Flux Monitor (Watts / m^2) */
    volatile uint64_t graser_coherent_flux_w_m2;

    /* 0x40 - 0x43: Peak Transient Stress Monitor (Kilopascals, Limit: 124600 kPa) */
    volatile uint32_t transient_shear_stress_kpa;
    /* 0x44 - 0x47: Borrmann Anomalous Transmission Suppression Ratio (Q16.16) */
    volatile uint32_t borrmann_epsilon_q16;

    /* 0x48 - 0x4B: System Status Flags and Fault Registers */
    volatile uint32_t system_status_flags;
    /* 0x4C - 0x4F: PCSS Crowbar Trip Counter */
    volatile uint32_t crowbar_trip_count;

    /* 0x50 - 0x53: Inductive Energy Recovery Ratio (Fixed-Point 0.32, Target: >= 0.9420) */
    volatile uint32_t inductive_recovery_eff_q32;
    /* 0x54 - 0x57: Direct Conversion Net Efficiency (Fixed-Point 0.32, Target: 0.4580) */
    volatile uint32_t net_conversion_eff_q32;

    /* 0x58 - 0x5B: Metric Dark-Ledger Coupling Sink Ratio (eta_D = 23/33 in Q0.32) */
    volatile uint32_t dark_ledger_sink_q32;
    /* 0x5C - 0x5F: Metric Active Drive Coupling Ratio (eta_A = 10/33 in Q0.32) */
    volatile uint32_t active_drive_ratio_q32;

    /* 0x60 - 0x63: Squeezed Vacuum Angle & Squeezing Parameter (r = 2.50 in Q16.16) */
    volatile uint32_t tmsv_squeezing_r_q16;
    /* 0x64 - 0x67: Hardware Interlock Control Command Register */
    volatile uint32_t interlock_cmd_reg;

    /* 0x68 - 0x7F: Reserved Hardware Alignment Padding to strictly enforce 128 Bytes */
    volatile uint8_t  hardware_reserved_pad[24];
} shbt_isomer_battery_mmio_t;

_Static_assert(_Alignof(shbt_isomer_battery_mmio_t) >= 64,
               "shbt_isomer_battery_mmio_t must be dual-cacheline (64 B) aligned");
_Static_assert(sizeof(shbt_isomer_battery_mmio_t) == SHBT_ISOMER_MMIO_TOTAL_SIZE,
               "shbt_isomer_battery_mmio_t must be exactly 128 bytes in size");
_Static_assert(offsetof(shbt_isomer_battery_mmio_t, cryo_headroom_uk) == 0x28,
               "cryo_headroom_uk must sit on cache line 1 at 0x28");
_Static_assert(offsetof(shbt_isomer_battery_mmio_t, tmsv_squeezing_r_q16) == 0x60,
               "tmsv_squeezing_r_q16 must sit at 0x60");
_Static_assert(offsetof(shbt_isomer_battery_mmio_t, hardware_reserved_pad) == 0x68,
               "reserved padding must begin at 0x68");

/**
 * @brief Volatile Pointer Cast Helper for Physical MMIO Mapping
 */
static inline volatile shbt_isomer_battery_mmio_t* shbt_get_isomer_battery_hw(void) {
    return (volatile shbt_isomer_battery_mmio_t*)((uintptr_t)SHBT_ISOMER_MMIO_BASE_ADDR);
}

#ifdef __cplusplus
}
#endif

#endif /* SHBT_ISOMER_BATTERY_MMIO_H */
