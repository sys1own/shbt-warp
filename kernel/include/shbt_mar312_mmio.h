/**
 * @file shbt_mar312_mmio.h
 * @brief Memory-Mapped I/O Register Interface for SHBT-MAR-312 Mass-Annihilation Reactor.
 * Contract: Exactly 128 bytes total size, aligned to 64-byte cacheline boundary.
 */

#ifndef SHBT_MAR312_MMIO_H
#define SHBT_MAR312_MMIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stdalign.h>
#include <stddef.h>
#include "shbt_ew312_mmio.h"

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_MAR312_MMIO_BASE_ADDR        (0x70000000ULL)
#define SHBT_MAR312_MMIO_TOTAL_SIZE       (128UL)
#define SHBT_MAR312_FRAME_BASE_ADDR       (0x70001000ULL)
#define SHBT_MAR312_FRAME_TOTAL_SIZE      (2112UL)
#define SHBT_MAR312_ACTIVE_BYTES          (640UL)
#define SHBT_MAR312_DARK_BYTES            (1472UL)

/* Unified Architecture: 1.0 CEU = 10 x SHBT-MAR-312C Seed Wafer Refills */
#define SHBT_CEU_CORE_MASS_KG             (376.99)
#define SHBT_CEU_GROSS_ENERGY_EJ          (33.8822)
#define SHBT_CEU_NET_ENERGY_EJ            (30.3310)
#define SHBT_CEU_WAFER_REFILLS_RATIO      (10U)
#define SHBT_MAR312C_SEED_WAFER_MASS_KG   (37.699)
#define SHBT_MAR312C_SEED_WAFER_VOL_L     (3.59)
#define SHBT_MAR312C_COMPACTED_NACELLE_KG (2257.70)
#define SHBT_MAR312_STATIONARY_NACELLE_KG (4096.99)

/* 5-Phase Dispatch State Machine Bitmasks */
#define MAR_STATE_STANDBY_STASIS   ((uint32_t)0x01U)
#define MAR_STATE_TRIGGER_ARMED    ((uint32_t)0x02U)
#define MAR_STATE_BURST_TRACTION   ((uint32_t)0x04U)
#define MAR_STATE_DEC_COOLDOWN     ((uint32_t)0x08U)
#define MAR_STATE_EMERGENCY_QUENCH ((uint32_t)0x10U)

/* Bipartite Synthesis FSM State Enums & Control Flags */
#define COSET_ANTIBARYON_FOLD         ((uint8_t)0x06U)
#define SYMPLECTIC_CRYSTALLIZATION    ((uint8_t)0x08U)

#ifndef SHBT_CTRL_DYNAMIC_GST_MATCH
#define SHBT_CTRL_DYNAMIC_GST_MATCH     (1ULL << 16)
#define SHBT_CTRL_PROGRESSIVE_EIKONAL   (1ULL << 17)
#define SHBT_CTRL_TTN_MERA_ADAPT        (1ULL << 18)
#define SHBT_CTRL_HYPERFRAME_64BANK     (1ULL << 19)
#endif

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

/* SHBT-RECON-SYNTH-MAR312 Bipartite Coset Spinor Synthesis Constants */
#define SHBT_SYNTH_STATE_COSET_ANTIBARYON_FOLD      ((uint8_t)0x06U)
#define SHBT_SYNTH_STATE_SYMPLECTIC_CRYSTALLIZATION ((uint8_t)0x08U)
#define SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16       ((uint16_t)0x8000U) /* 0.500 in Q0.16         */
#define SHBT_SYNTH_COSET_WEIGHT_45_64_Q16           ((uint16_t)0xB400U) /* 45/64 = 0.703125 Q0.16 */
#define SHBT_SYNTH_RG_LOCK_312                      ((uint32_t)(1U << 16))
#define SHBT_SYNTH_CBARB_ZERO_VERIFIED              ((uint32_t)(1U << 17))

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
typedef struct __attribute__((aligned(64))) {
    /* Cacheline 0 (0x00 - 0x3F): Dispatch, Thermal & Wavefront Telemetry */
    volatile uint64_t reactor_status_flags;          /* 0x00 */
    volatile uint64_t de_rendering_tau_q32;          /* 0x08 */
    volatile uint64_t active_bus_power_w;            /* 0x10 */
    volatile uint32_t cryo_temp_millikelvin;         /* 0x18 */
    volatile uint32_t phononic_damping_coeff;        /* 0x1C */
    volatile uint32_t rebco_coil_current_amperes;    /* 0x20 */
    volatile uint32_t magnetic_nozzle_field_mt;      /* 0x24 */
    volatile uint32_t eikonal_velocity_kms;          /* 0x28 */
    volatile uint32_t kohn_sham_shift_ev_q16;        /* 0x2C */
    volatile uint16_t synth_antibaryon_ratio_q16;    /* 0x30: 0x8000 = 0.500 */
    volatile uint16_t synth_coset_weight_q16;        /* 0x32: 0xB400 = 45/64 */
    volatile uint16_t synth_jitter_bound_q16;        /* 0x34: Max allowable jitter (attoseconds) */
    volatile uint16_t synth_tether_displacement_q16; /* 0x36: CS tether displacement (10^-5 A) */
    volatile uint32_t reserved_c0_align;             /* 0x38 */
    volatile uint32_t reserved_c0_pad;               /* 0x3C */

    /* Cacheline 1 (0x40 - 0x7F): DEC Cascade, Space-Charge & TQEC Telemetry */
    volatile uint64_t dec_discharge_energy_uj;       /* 0x40 */
    volatile uint16_t dec_space_charge_margin_q16;   /* 0x48: Child-Langmuir headroom (Q16) */
    volatile uint16_t tqec_syndrome_err_rate_q16;    /* 0x4A: Physical error rate (Q16) */
    volatile uint32_t rebco_quench_headroom_q16;     /* 0x4C: Critical current margin (Q16) */
    volatile uint64_t poynting_recovery_flux_w;      /* 0x50 */
    volatile uint32_t mhd_diamagnetic_beta_q16;      /* 0x58 */
    volatile uint32_t anyon_braid_cycle_count;       /* 0x5C */
    volatile uint64_t dark_ledger_wzw_checksum;      /* 0x60 */
    volatile uint8_t  hardware_pad[24];              /* 0x68 - 0x7F: Zero-fill to 128 bytes */
} shbt_mar312_mmio_t;

_Static_assert(sizeof(shbt_mar312_mmio_t) == 128, "shbt_mar312_mmio_t must be exactly 128 bytes");
_Static_assert(__alignof__(shbt_mar312_mmio_t) == 64, "shbt_mar312_mmio_t must be 64-byte aligned");

_Static_assert(offsetof(shbt_mar312_mmio_t, reactor_status_flags) == 0x00, "reactor_status_flags offset must be 0x00");
_Static_assert(offsetof(shbt_mar312_mmio_t, de_rendering_tau_q32) == 0x08, "de_rendering_tau_q32 offset must be 0x08");
_Static_assert(offsetof(shbt_mar312_mmio_t, active_bus_power_w) == 0x10, "active_bus_power_w offset must be 0x10");
_Static_assert(offsetof(shbt_mar312_mmio_t, cryo_temp_millikelvin) == 0x18, "cryo_temp_millikelvin offset must be 0x18");
_Static_assert(offsetof(shbt_mar312_mmio_t, phononic_damping_coeff) == 0x1C, "phononic_damping_coeff offset must be 0x1C");
_Static_assert(offsetof(shbt_mar312_mmio_t, rebco_coil_current_amperes) == 0x20, "rebco_coil_current_amperes offset must be 0x20");
_Static_assert(offsetof(shbt_mar312_mmio_t, magnetic_nozzle_field_mt) == 0x24, "magnetic_nozzle_field_mt offset must be 0x24");
_Static_assert(offsetof(shbt_mar312_mmio_t, eikonal_velocity_kms) == 0x28, "eikonal_velocity_kms offset must be 0x28");
_Static_assert(offsetof(shbt_mar312_mmio_t, kohn_sham_shift_ev_q16) == 0x2C, "kohn_sham_shift_ev_q16 offset must be 0x2C");
_Static_assert(offsetof(shbt_mar312_mmio_t, synth_antibaryon_ratio_q16) == 0x30, "synth_antibaryon_ratio_q16 offset must be 0x30");
_Static_assert(offsetof(shbt_mar312_mmio_t, synth_coset_weight_q16) == 0x32, "synth_coset_weight_q16 offset must be 0x32");
_Static_assert(offsetof(shbt_mar312_mmio_t, synth_jitter_bound_q16) == 0x34, "synth_jitter_bound_q16 offset must be 0x34");
_Static_assert(offsetof(shbt_mar312_mmio_t, synth_tether_displacement_q16) == 0x36, "synth_tether_displacement_q16 offset must be 0x36");
_Static_assert(offsetof(shbt_mar312_mmio_t, dec_discharge_energy_uj) == 0x40, "dec_discharge_energy_uj offset must be 0x40");
_Static_assert(offsetof(shbt_mar312_mmio_t, dec_space_charge_margin_q16) == 0x48, "dec_space_charge_margin_q16 offset must be 0x48");
_Static_assert(offsetof(shbt_mar312_mmio_t, tqec_syndrome_err_rate_q16) == 0x4A, "tqec_syndrome_err_rate_q16 offset must be 0x4A");
_Static_assert(offsetof(shbt_mar312_mmio_t, rebco_quench_headroom_q16) == 0x4C, "rebco_quench_headroom_q16 offset must be 0x4C");
_Static_assert(offsetof(shbt_mar312_mmio_t, poynting_recovery_flux_w) == 0x50, "poynting_recovery_flux_w offset must be 0x50");
_Static_assert(offsetof(shbt_mar312_mmio_t, mhd_diamagnetic_beta_q16) == 0x58, "mhd_diamagnetic_beta_q16 offset must be 0x58");
_Static_assert(offsetof(shbt_mar312_mmio_t, anyon_braid_cycle_count) == 0x5C, "anyon_braid_cycle_count offset must be 0x5C");
_Static_assert(offsetof(shbt_mar312_mmio_t, dark_ledger_wzw_checksum) == 0x60, "dark_ledger_wzw_checksum offset must be 0x60");

/**
 * @brief 2,112-Byte SRAM Unified Stinespring Arena at 0x70001000
 *        Active visible: 640 B (eta_A = 10/33)
 *        Dark braids: 992 B (124 Fibonacci braid channels x 8 B)
 *        SECDED ECC & syndrome metadata: 480 B
 */
typedef struct {
    union {
        struct {
            _Alignas(64) uint8_t active_visible[SHBT_MAR312_ACTIVE_BYTES]; /* 0x000..0x280: 640 B  */
            uint8_t              dark_braids[992];                         /* 0x280..0x660: 992 B  */
            uint8_t              ecc_syndrome_meta[480];                   /* 0x660..0x840: 480 B  */
        };
        struct {
            _Alignas(64) uint8_t active_frame[SHBT_MAR312_ACTIVE_BYTES];   /* 640 B  (eta_A = 10/33) */
            uint8_t              dark_ledger[SHBT_MAR312_DARK_BYTES];     /* 1472 B (eta_D = 23/33) */
        };
    };
} UnifiedStinespringFrame_t;

_Static_assert(sizeof(UnifiedStinespringFrame_t) == SHBT_MAR312_FRAME_TOTAL_SIZE,
               "UnifiedStinespringFrame_t must be exactly 2112 bytes");
_Static_assert(offsetof(UnifiedStinespringFrame_t, dark_braids) == 0x280,
               "dark_braids must sit at offset 0x280 (640 B)");
_Static_assert(offsetof(UnifiedStinespringFrame_t, ecc_syndrome_meta) == 0x660,
               "ecc_syndrome_meta must sit at offset 0x660 (1632 B)");

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

/**
 * @brief Validate SHBT-RECON-SYNTH-MAR312 bipartite coset fold MMIO words
 */
bool shbt_mar312_synth_fold_ok(uint16_t antibaryon_ratio_q16,
                               uint16_t coset_weight_q16,
                               uint32_t synth_status_flags);

#ifdef __cplusplus
}
#endif

#endif /* SHBT_MAR312_MMIO_H */
