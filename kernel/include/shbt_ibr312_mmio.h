/*
 * @file shbt_ibr312_mmio.h
 * @brief C11 Hardware Register Architecture for Compacted Catalytic Wafer Reactor
 *        Nacelle (SHBT-MAR-312C) & Intergalactic Dark-Sector Anyon Braid Reload
 *        Magazine (SHBT-IBR-312).
 * @details Physical base address: 0x70000000 (128 bytes, 64-byte dual cacheline aligned)
 *          Magazine arena base: 0x70002000 (33,792 bytes = 33.0 KB 16-bank ring buffer)
 */

#ifndef SHBT_IBR312_MMIO_H
#define SHBT_IBR312_MMIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stdalign.h>
#include <stddef.h>
#include "shbt_mar312_mmio.h"

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_IBR312_MAGAZINE_BASE_ADDR   (0x70002000ULL)
#define SHBT_IBR312_MAGAZINE_BANKS       (16UL)
#define SHBT_IBR312_MAGAZINE_TOTAL_SIZE  (33792UL) /* 16 * 2112 B */

/* Control Bitflags at Offset 0x04 / system_status_flags */
#define SHBT_MAR_SYMPLECTIC_SHIELD_LOCK   ((uint32_t)(1U << 8))
#define SHBT_MAR_REBCO_DEFLECTOR_18T5     ((uint32_t)(1U << 9))
#define SHBT_IBR_AUTO_RELOAD_ARMED        ((uint32_t)(1U << 10))
#define SHBT_IBR_RELOAD_IN_PROGRESS       ((uint32_t)(1U << 11))
#define SHBT_IBR_ZERO_VOLUME_LOCK         ((uint32_t)(1U << 12))
#define SHBT_IBR_ADM_FRAME_COMPLIANT      ((uint32_t)(1U << 13))

/* Ghost-Seed Control Bitflags */
#define SHBT_EW_CTRL_GHOST_HOVER_LOCK     (1ULL << 21)
#define SHBT_EW_CTRL_GHOST_HARVEST_FUNNEL (1ULL << 22)
#define SHBT_EW_CTRL_HABITAT_1G_FLOOR     (1ULL << 23)
#define SHBT_EW_CTRL_WAKE_COMP_3RD_ORDER  (1ULL << 24)

/* Default Normative Constant Values */
#define SHBT_IBR_SYMPLECTIC_S_Q32         ((uint32_t)0x17829CBCU) /* (10/33)^2 in Q0.32 */
#define SHBT_IBR_SHIELD_MASS_G            ((uint32_t)384880U)     /* 384.88 kg in grams */
#define SHBT_IBR_NACELLE_MASS_G           ((uint32_t)2257700U)    /* 2,257.70 kg in grams */
#define SHBT_IBR_CREW_DOSE_NSV_YR         ((uint32_t)114000U)     /* 0.114 mSv/yr in nSv/yr */

/* 8-State Autonomous Hot-Swap FSM States */
#define IBR_FSM_S0_NOMINAL_BURN           ((uint16_t)0x00)
#define IBR_FSM_S1_SOC_THRESHOLD_DETECT   ((uint16_t)0x01)
#define IBR_FSM_S2_SMES_BUS_CLAMP         ((uint16_t)0x02)
#define IBR_FSM_S3_GIVENS_BANK_ALIGN      ((uint16_t)0x03)
#define IBR_FSM_S4_STINESPRING_UNFOLD     ((uint16_t)0x04)
#define IBR_FSM_S5_SUGAWARA_PARITY_LOCK   ((uint16_t)0x05)
#define IBR_FSM_S6_MOSSBAUER_SYNC         ((uint16_t)0x06)
#define IBR_FSM_S7_HANDOFF_RESUME         ((uint16_t)0x07)

/* 4 Braid Reload Modes */
#define IBR_MODE_PREFOLDED_UNFOLD         ((uint16_t)0x00)
#define IBR_MODE_1C_ISOBARIC_RECYCLE      ((uint16_t)0x01)
#define IBR_MODE_1A_MATCHED_FOUNDRY       ((uint16_t)0x02)
#define IBR_MODE_1B_RAW_SLAG_SCOOP        ((uint16_t)0x03)

/**
 * @brief 16-Bank Stinespring Dark-Braid Magazine Ring Buffer (33,792 Bytes = 33.0 KB)
 */
typedef struct {
    _Alignas(64) UnifiedStinespringFrame_t banks[SHBT_IBR312_MAGAZINE_BANKS];
} shbt_stinespring_magazine_t;

_Static_assert(sizeof(shbt_stinespring_magazine_t) == SHBT_IBR312_MAGAZINE_TOTAL_SIZE,
               "shbt_stinespring_magazine_t must be exactly 33,792 bytes (33.0 KB)");

/**
 * @brief Standalone Compacted SHBT-MAR-312-IBR MMIO Register Block (128 Bytes)
 */
typedef struct {
    /* 0x00 - 0x07 */
    _Alignas(64) volatile uint64_t energy_remaining_joules;
    /* 0x08 - 0x0F */
    volatile uint64_t state_of_charge_q32;
    /* 0x10 - 0x17 */
    volatile uint64_t bus_voltage_uv;
    /* 0x18 - 0x1F */
    volatile uint64_t bus_current_ua;
    /* 0x20 - 0x27 */
    volatile uint64_t core_temperature_uk;
    /* 0x28 - 0x2F */
    volatile uint64_t cryo_headroom_uk;
    /* 0x30 - 0x33 */
    volatile uint32_t trigger_delay_ps;
    /* 0x34 - 0x37 */
    volatile uint32_t trigger_pulse_width_ps;
    /* 0x38 - 0x3F */
    volatile uint64_t graser_coherent_flux_w_m2;
    /* 0x40 - 0x43 */
    volatile uint32_t transient_shear_stress_kpa;
    /* 0x44 - 0x47 */
    volatile uint32_t borrmann_epsilon_q16;
    /* 0x48 - 0x4B */
    volatile uint32_t system_status_flags;
    /* 0x4C - 0x4F */
    volatile uint32_t crowbar_trip_count;
    /* 0x50 - 0x53 */
    volatile uint32_t inductive_recovery_eff_q32;
    /* 0x54 - 0x57 */
    volatile uint32_t net_conversion_eff_q32;

    /* 0x58 - 0x5B: Symplectic S Factor (0x17829CBC) */
    volatile uint32_t symplectic_s_factor_q32;
    /* 0x5C - 0x5F: Symplectic Shield Mass in grams (384,880 g) */
    volatile uint32_t symplectic_shield_mass_g;
    /* 0x60 - 0x63: Compacted Nacelle Mass in grams (2,257,700 g) */
    volatile uint32_t compacted_nacelle_mass_g;
    /* 0x64 - 0x65: Active Bank Selector (0..15) */
    volatile uint16_t ibr_frame_bank_select;
    /* 0x66 - 0x67: Reload Mode (0..3) */
    volatile uint16_t ibr_reload_mode;
    /* 0x68 - 0x6B: Stored Micro-Batches Remaining */
    volatile uint32_t ibr_stored_micro_batches;
    /* 0x6C - 0x6F: Total Hot-Swap Reload Count */
    volatile uint32_t ibr_reload_count;
    /* 0x70 - 0x73: Last Reload Execution Latency in nanoseconds */
    volatile uint32_t ibr_last_latency_ns;
    /* 0x74 - 0x77: Crew Cabin Dose Rate in nSv/yr (114,000 nSv/yr) */
    volatile uint32_t ibr_crew_dose_nsv_yr;
    /* 0x78 - 0x7F: Frame Decoupling Angle (cos^2 theta in Q0.64) */
    volatile uint64_t ibr_frame_angle_q64;
} shbt_ibr312_mmio_t;

_Static_assert(sizeof(shbt_ibr312_mmio_t) == 128,
               "shbt_ibr312_mmio_t must be exactly 128 bytes in size");
_Static_assert(offsetof(shbt_ibr312_mmio_t, symplectic_s_factor_q32) == 0x58,
               "symplectic_s_factor_q32 offset must be 0x58");
_Static_assert(offsetof(shbt_ibr312_mmio_t, symplectic_shield_mass_g) == 0x5C,
               "symplectic_shield_mass_g offset must be 0x5C");
_Static_assert(offsetof(shbt_ibr312_mmio_t, compacted_nacelle_mass_g) == 0x60,
               "compacted_nacelle_mass_g offset must be 0x60");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_frame_bank_select) == 0x64,
               "ibr_frame_bank_select offset must be 0x64");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_reload_mode) == 0x66,
               "ibr_reload_mode offset must be 0x66");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_stored_micro_batches) == 0x68,
               "ibr_stored_micro_batches offset must be 0x68");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_reload_count) == 0x6C,
               "ibr_reload_count offset must be 0x6C");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_last_latency_ns) == 0x70,
               "ibr_last_latency_ns offset must be 0x70");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_crew_dose_nsv_yr) == 0x74,
               "ibr_crew_dose_nsv_yr offset must be 0x74");
_Static_assert(offsetof(shbt_ibr312_mmio_t, ibr_frame_angle_q64) == 0x78,
               "ibr_frame_angle_q64 offset must be 0x78");

#ifdef __cplusplus
}
#endif

#endif /* SHBT_IBR312_MMIO_H */
