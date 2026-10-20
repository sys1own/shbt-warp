/*
 * @file shbt_ew312_mmio.h
 * @brief C11 Hardware Register Architecture for SHBT-EW-312
 *        Traversable Effective Wormhole System (Alcubierre-Stinespring Corridor)
 * @details Physical base address: 0x70000000 (56 bytes packed, 4-byte aligned)
 *          z3_contract_sig at offset 0x34 verified by 0x3120CAFE
 */

#ifndef SHBT_EW312_MMIO_H
#define SHBT_EW312_MMIO_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_EW312_MMIO_BASE_ADDR   (0x70000000ULL)
#define SHBT_EW312_MMIO_SIZE        (56UL)
#define SHBT_EW312_Z3_CONTRACT_SIG  (0x3120CAFEU)

/* 6-State Wormhole Gateway / Flight FSM */
#define SHBT_STATE_STANDBY_STASIS     ((uint8_t)0x00U)
#define SHBT_STATE_INGEST_DERENDER    ((uint8_t)0x01U)
#define SHBT_STATE_DARK_LEDGER_LOCK   ((uint8_t)0x02U)
#define SHBT_STATE_CORRIDOR_TRACTION  ((uint8_t)0x03U)
#define SHBT_STATE_CAUSAL_EGRESS      ((uint8_t)0x04U)
#define SHBT_STATE_EMERGENCY_QUENCH   ((uint8_t)0xFFU)

/* Scaled Control Register Flags */
#define SHBT_EW_CTRL_ENABLE_PIPELINE       (1ULL << 0)
#define SHBT_EW_CTRL_DYNAMIC_GST_MATCH     (1ULL << 16)
#define SHBT_EW_CTRL_PROGRESSIVE_EIKONAL   (1ULL << 17)
#define SHBT_EW_CTRL_TTN_MERA_ADAPT        (1ULL << 18)
#define SHBT_EW_CTRL_HYPERFRAME_64BANK     (1ULL << 19)
#define SHBT_EW_CTRL_TERNARY_HUFFMAN_PACK  (1ULL << 20)

#define SHBT_CTRL_DYNAMIC_GST_MATCH        SHBT_EW_CTRL_DYNAMIC_GST_MATCH
#define SHBT_CTRL_PROGRESSIVE_EIKONAL      SHBT_EW_CTRL_PROGRESSIVE_EIKONAL
#define SHBT_CTRL_TTN_MERA_ADAPT           SHBT_EW_CTRL_TTN_MERA_ADAPT
#define SHBT_CTRL_HYPERFRAME_64BANK        SHBT_EW_CTRL_HYPERFRAME_64BANK

/* 2PN Relativistic Physical Constants */
#define EW312_G_CONST   (6.67430e-11)
#define EW312_C_LIGHT   (299792458.0)
#define EW312_M_SUN     (1.98847e30)
#define EW312_J2_SUN    (2.20e-7)
#define EW312_R_SUN     (6.96342e8)

#define SHBT_VISIBLE_BYTES          640U
#define SHBT_BRAID_DESCRIPTORS      124U
#define SHBT_BRAID_BYTES            992U
#define SHBT_ECC_BYTES              480U
#define SHBT_SINGLE_FRAME_BYTES     2112U
#define SHBT_HYPERFRAME_MAX_BANKS   64U
#define SHBT_HYPERFRAME_TOTAL_BYTES (SHBT_HYPERFRAME_MAX_BANKS * SHBT_SINGLE_FRAME_BYTES) /* 135,168 B */

typedef struct __attribute__((aligned(64))) {
    uint8_t  visible_register[SHBT_VISIBLE_BYTES];     /* 640 B (eta_A = 10/33) */
    uint64_t braid_descriptors[SHBT_BRAID_DESCRIPTORS];/* 992 B (124 x 8B Ternary Huffman) */
    uint64_t secded_ecc[60];                           /* 480 B Hamming(72,64) */
} UnifiedStinespringFrame;

_Static_assert(sizeof(UnifiedStinespringFrame) == 2112U,
               "C-ABI Invariant Violation: sizeof(UnifiedStinespringFrame) must equal 2112 bytes");

typedef struct __attribute__((aligned(64))) {
    UnifiedStinespringFrame banks[SHBT_HYPERFRAME_MAX_BANKS];
} shbt_stinespring_hyperframe_t;

_Static_assert(sizeof(shbt_stinespring_hyperframe_t) == 135168U,
               "C-ABI Invariant Violation: sizeof(shbt_stinespring_hyperframe_t) must equal 135168 bytes");

/* Hyper-Frame Real-Time Telemetry Mapping */
typedef struct {
    uint64_t active_bank_mask;        /* 64-bit mask of active banks */
    uint32_t z_meta_mrayl_q16;        /* tuned Z_meta in Q16.16 MRayl */
    uint32_t crystalline_frac_q24;    /* active GST-225 f_c(t) in Q8.24 */
    uint32_t peak_stress_mpa_q16;     /* clamped interfacial stress in Q16.16 MPa */
    uint16_t ttn_mera_active_layers;  /* active TTN-MERA layer depth (e.g. 101) */
    uint16_t braid_huffman_max_len;   /* braid word length (42 generators) */
    uint32_t cryo_stage_temp_mk;      /* temperature in mK (21130 mK) */
    uint32_t eikonal_velocity_mm_s;   /* instantaneous de-render velocity in mm/s */
    uint64_t adm_conserved_mass_ug;   /* conserved ADM mass monitor */
} shbt_ew_hyperframe_telemetry_t;

#pragma pack(push, 1)
typedef struct {
    volatile uint32_t causal_cone_lo;     /* 0x00: Causal auth control/status low        */
    volatile uint32_t causal_cone_hi;     /* 0x04: Bit31: trigger 2PN eval               */
    volatile double   pn2_metric_m0;      /* 0x08: Central mass M_sun (f64)              */
    volatile double   pn2_metric_j2;      /* 0x10: Quadrupole J2 (f64)                   */
    volatile float    pn2_spin_vec[3];    /* 0x18: Spin S_x, S_y, S_z (f32 x 3)          */
    volatile uint32_t target_vel_gamma;   /* 0x24: Gamma, 16.16 fixed point              */
    volatile uint32_t ds2_interval_lo;    /* 0x28: ds^2_2PN bits 31:0                    */
    volatile int32_t  ds2_interval_hi;    /* 0x2C: ds^2_2PN bits 63:32, signed           */
    volatile uint32_t quench_time_ns;     /* 0x30: Anomaly->quench latch ns              */
    volatile uint32_t z3_contract_sig;    /* 0x34: Z3 formal signature (0x3120CAFE)      */
} shbt_ew312_mmio_t;
#pragma pack(pop)

_Static_assert(sizeof(shbt_ew312_mmio_t) == 56,
               "shbt_ew312_mmio_t must be exactly 56 bytes");
_Static_assert(offsetof(shbt_ew312_mmio_t, z3_contract_sig) == 0x34,
               "z3_contract_sig must be at offset 0x34");

void shbt_ew312_causal_init(volatile shbt_ew312_mmio_t *mmio);
bool shbt_ew312_evaluate_2pn(volatile shbt_ew312_mmio_t *mmio,
                             double dt, double dx, double dy, double dz,
                             double vx, double vy, double vz);
uint8_t shbt_ew312_fsm_step(volatile shbt_ew312_mmio_t *mmio, uint8_t next_state);

void shbt_ew312_hyperframe_tick_50k518(uint64_t ctrl_flags,
                                       uint32_t bulk_impedance_q16,
                                       uint32_t eikonal_tau_q24,
                                       uint64_t nucleon_count_e26,
                                       uint32_t active_bank_count,
                                       shbt_ew_hyperframe_telemetry_t *telem,
                                       shbt_stinespring_hyperframe_t *hyperframe);

#ifdef __cplusplus
}
#endif

#endif /* SHBT_EW312_MMIO_H */
