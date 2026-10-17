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

/* 2PN Relativistic Physical Constants */
#define EW312_G_CONST   (6.67430e-11)
#define EW312_C_LIGHT   (299792458.0)
#define EW312_M_SUN     (1.98847e30)
#define EW312_J2_SUN    (2.20e-7)
#define EW312_R_SUN     (6.96342e8)

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

#ifdef __cplusplus
}
#endif

#endif /* SHBT_EW312_MMIO_H */
