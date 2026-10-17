/*
 * @file shbt_mar312_mmio.h
 * @brief C11 Hardware Register Architecture for 4th-Gen Stinespring-SO(10)
 *        Mass-Annihilation Battery-Reactor (SHBT-MAR-312).
 * @details Physical base address: 0x70000000 (128 bytes, 64-byte dual cacheline aligned)
 *          UnifiedStinespringFrame arena base: 0x70001000 (2,112 bytes SRAM arena)
 */

#ifndef SHBT_MAR312_MMIO_H
#define SHBT_MAR312_MMIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stdalign.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_MAR312_MMIO_BASE_ADDR        (0x70000000ULL)
#define SHBT_MAR312_MMIO_TOTAL_SIZE       (128UL)
#define SHBT_MAR312_FRAME_BASE_ADDR       (0x70001000ULL)
#define SHBT_MAR312_FRAME_TOTAL_SIZE      (2112UL)
#define SHBT_MAR312_ACTIVE_BYTES          (640UL)
#define SHBT_MAR312_DARK_BYTES            (1472UL)

/* 5-Phase Dispatch State Machine Bitmasks */
#define MAR_STATE_STANDBY_STASIS   ((uint32_t)0x01U)
#define MAR_STATE_TRIGGER_ARMED    ((uint32_t)0x02U)
#define MAR_STATE_BURST_TRACTION   ((uint32_t)0x04U)
#define MAR_STATE_DEC_COOLDOWN     ((uint32_t)0x08U)
#define MAR_STATE_EMERGENCY_QUENCH ((uint32_t)0x10U)

/* Hardware Fault & Interlock Bitmasks */
#define MAR_FAULT_NONE             ((uint32_t)0x00000000U)
#define MAR_FAULT_ANOMALY_CLOSURE  ((uint32_t)0x00000001U) /* R_entropy < 0          */
#define MAR_FAULT_CRYO_QUENCH_WARN ((uint32_t)0x00000002U) /* T_op > 31.50 K         */
#define MAR_FAULT_NBN_TRANSITION   ((uint32_t)0x00000004U) /* T_NbN > 8.00 K         */
#define MAR_FAULT_VESSEL_STRESS    ((uint32_t)0x00000008U) /* sigma > 124.60 MPa     */
#define MAR_FAULT_RMHD_ALFVEN      ((uint32_t)0x00000010U) /* M_A > 0.120            */
#define MAR_FAULT_ADM_DET_UNLOCK   ((uint32_t)0x00000020U) /* |det(g) + 1| > 1e-12   */
#define MAR_FAULT_WAKE_RIGIDITY    ((uint32_t)0x00000040U) /* |mu_comp-mu_0| > 1e-12 */
#define MAR_FAULT_TQEC_DEADLINE    ((uint32_t)0x00000080U) /* t_decode > 45.0 ns     */

/**
 * @brief Telemetry snapshot fed to shbt_mar312_tick_dispatch()
 */
typedef struct {
    double   t_core_kelvin;       /**< Core operating temperature (K)           */
    double   t_nbn_kelvin;        /**< NbN transducer bath temperature (K)      */
    double   sigma_vessel_mpa;    /**< Peak diamond vessel von Mises stress     */
    double   rmhd_mach_alfven;    /**< Relativistic MHD Alfvén Mach number M_A  */
    double   adm_det_g_err;       /**< |det(g) + 1| foliation metric error      */
    double   wake_mu_err;         /**< |mu_comp - mu_0| permeability error      */
    double   lhe_void_fraction;   /**< Helium two-phase exit void fraction      */
    uint32_t tqec_latency_ps;     /**< Blossom-V TQEC decode latency (ps)       */
    uint64_t entropy_headroom;    /**< Holographic entropy closure headroom     */
} shbt_mar312_telemetry_t;

/**
 * @brief Memory-Mapped Register Layout for SHBT-MAR-312 at 0x70000000 (Exact 128 Bytes)
 */
typedef struct {
    /* 0x00 - 0x07: State-of-Charge & Primary Energy Storage (Joules) */
    _Alignas(64) volatile uint64_t energy_remaining_joules;
    /* 0x08 - 0x0F: Absolute State-of-Charge (Fixed-Point 32.32) */
    volatile uint64_t state_of_charge_q32;
    /* 0x10 - 0x17: Regulated Bus Voltage (Microvolts) */
    volatile uint64_t bus_voltage_uv;
    /* 0x18 - 0x1F: Regulated Bus Current (Microamperes) */
    volatile uint64_t bus_current_ua;
    /* 0x20 - 0x27: Real-Time Core Thermal Sensor (Microkelvin) */
    volatile uint64_t core_temperature_uk;
    /* 0x28 - 0x2F: Cryogenic Temperature Headroom (Microkelvin) */
    volatile uint64_t cryo_headroom_uk;
    /* 0x30 - 0x33: Optical Gateway Trigger Delay (Picoseconds) */
    volatile uint32_t trigger_delay_ps;
    /* 0x34 - 0x37: Optical Gateway Trigger Pulse Width (Picoseconds) */
    volatile uint32_t trigger_pulse_width_ps;
    /* 0x38 - 0x3F: Gateway Coherent Radiant Flux Monitor (Watts / m^2) */
    volatile uint64_t graser_coherent_flux_w_m2;
    /* 0x40 - 0x43: Peak Transient Vessel Stress (Kilopascals) */
    volatile uint32_t transient_shear_stress_kpa;
    /* 0x44 - 0x47: Borrmann Anomalous Transmission Suppression (Q16.16) */
    volatile uint32_t borrmann_epsilon_q16;
    /* 0x48 - 0x4B: System Status Flags */
    volatile uint32_t system_status_flags;
    /* 0x4C - 0x4F: PCSS Crowbar Trip Counter */
    volatile uint32_t crowbar_trip_count;
    /* 0x50 - 0x53: SMES Inductive Energy Recovery Ratio (Q0.32) */
    volatile uint32_t inductive_recovery_eff_q32;
    /* 0x54 - 0x57: 5-Stage DEC Net Conversion Efficiency (Q0.32) */
    volatile uint32_t net_conversion_eff_q32;
    /* 0x58 - 0x5B: Dark Ledger Coupling Ratio (eta_D = 23/33 in Q0.32) */
    volatile uint32_t dark_ledger_sink_q32;
    /* 0x5C - 0x5F: Active Drive Ratio (eta_A = 10/33 in Q0.32) */
    volatile uint32_t active_drive_ratio_q32;
    /* 0x60 - 0x63: TMSV Squeezing Parameter (r = 2.50 in Q16.16) */
    volatile uint32_t tmsv_squeezing_r_q16;
    /* 0x64 - 0x67: Hardware Interlock Control Command Register */
    volatile uint32_t interlock_cmd_reg;

    /* 0x68 - 0x6B: Sub-2.10 ns PCSS Crowbar Command Register (0xFFFFFFFF = TRIP) */
    volatile uint32_t pcss_crowbar_cmd;
    /* 0x6C - 0x6F: 5-Phase Dispatch State Machine Word */
    volatile uint32_t mar_state;
    /* 0x70 - 0x73: MAR-312 Hardware Fault Word */
    volatile uint32_t mar_fault_word;
    /* 0x74 - 0x77: Blossom-V TQEC Decoding Latency in picoseconds (<= 45,000 ps) */
    volatile uint32_t tqec_latency_ps;
    /* 0x78 - 0x7F: Hardware Alignment Padding */
    volatile uint8_t  padding[8];
} shbt_mar312_mmio_t;

_Static_assert(sizeof(shbt_mar312_mmio_t) == SHBT_MAR312_MMIO_TOTAL_SIZE,
               "shbt_mar312_mmio_t must be exactly 128 bytes in size");
_Static_assert(_Alignof(shbt_mar312_mmio_t) >= 64,
               "shbt_mar312_mmio_t must be dual-cacheline (64 B) aligned");
_Static_assert(offsetof(shbt_mar312_mmio_t, cryo_headroom_uk) == 0x28,
               "cryo_headroom_uk offset must be 0x28");
_Static_assert(offsetof(shbt_mar312_mmio_t, pcss_crowbar_cmd) == 0x68,
               "pcss_crowbar_cmd offset must be 0x68");
_Static_assert(offsetof(shbt_mar312_mmio_t, mar_state) == 0x6C,
               "mar_state offset must be 0x6C");
_Static_assert(offsetof(shbt_mar312_mmio_t, mar_fault_word) == 0x70,
               "mar_fault_word offset must be 0x70");
_Static_assert(offsetof(shbt_mar312_mmio_t, tqec_latency_ps) == 0x74,
               "tqec_latency_ps offset must be 0x74");

/**
 * @brief 2,112-Byte SRAM Unified Stinespring Arena at 0x70001000
 */
typedef struct {
    _Alignas(64) uint8_t active_frame[SHBT_MAR312_ACTIVE_BYTES];  /* 640 B  (eta_A = 10/33) */
    uint8_t              dark_ledger[SHBT_MAR312_DARK_BYTES];    /* 1472 B (eta_D = 23/33) */
} UnifiedStinespringFrame_t;

_Static_assert(sizeof(UnifiedStinespringFrame_t) == SHBT_MAR312_FRAME_TOTAL_SIZE,
               "UnifiedStinespringFrame_t must be exactly 2112 bytes");

static inline volatile shbt_mar312_mmio_t* shbt_get_mar312_hw(void) {
    return (volatile shbt_mar312_mmio_t*)((uintptr_t)SHBT_MAR312_MMIO_BASE_ADDR);
}

static inline volatile UnifiedStinespringFrame_t* shbt_get_stinespring_frame(void) {
    return (volatile UnifiedStinespringFrame_t*)((uintptr_t)SHBT_MAR312_FRAME_BASE_ADDR);
}

/**
 * @brief MAR-312 Tick Dispatch Routine
 * @param mar Pointer to MMIO register block
 * @param telem Pointer to telemetry struct
 * @return 0 on nominal dispatch, or fault bitmask on emergency trip.
 */
uint32_t shbt_mar312_tick_dispatch(volatile shbt_mar312_mmio_t *mar,
                                   const shbt_mar312_telemetry_t *telem);

#ifdef __cplusplus
}
#endif

#endif /* SHBT_MAR312_MMIO_H */
