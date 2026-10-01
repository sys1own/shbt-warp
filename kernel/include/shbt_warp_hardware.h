/*
 * shbt_warp_hardware.h — SHBT-MMIO-WARP register map.
 *
 * Normative 128-byte dual-cacheline aligned register block anchored at
 * physical base address 0x70000000. Cache line 0 carries control/status,
 * the LANR/Landauer power ledger and the warp metric registers; cache
 * line 1 carries the curvature/QI/emitter/ECC/flight registers.
 */
#ifndef SHBT_WARP_HARDWARE_H
#define SHBT_WARP_HARDWARE_H

#include <stdint.h>

#define SHBT_WARP_MMIO_BASE 0x70000000UL
#define SHBT_WARP_MMIO_SIZE 128u

/* Register offsets */
#define REG_SYS_CONTROL        0x00 /* Bit0 Enable | Bit1 Bubble | Bit2 Quench */
#define REG_SYS_STATUS         0x04 /* Bit0 Stable | Bit1 ECC OK | Bit2 Lapse  */
#define REG_LANR_POWER_MW      0x08 /* LANR ledger, kW (999.054 baseline)      */
#define REG_LANDAUER_DEBT_MW   0x10 /* Active Landauer debt, kW (906.00)       */
#define REG_BUBBLE_VELOCITY_C  0x18 /* Commanded v_s in Q32.32                 */
#define REG_WALL_THICKNESS_NM  0x20 /* Bubble wall thickness sigma_w           */
#define REG_LAPSE_ERROR_RAW    0x28 /* Scaled |det(g)+1| residual              */
#define REG_QUENCH_TIMER_NS    0x2C /* Crowbar latch timer (<= 2.18 ns)        */
#define REG_INTERLOCK_FLAGS    0x30 /* Bit0 Spacelike|Bit1 Under|Bit2 Horizon  */
#define REG_SHIFT_BETA_X       0x40 /* Shift vector beta^x in Q32.32           */
#define REG_RICCI_SCALAR       0x48 /* Bulk Ricci curvature scalar R           */
#define REG_QI_INTEGRAL_BOUND  0x50 /* Integrated Ford-Roman QI margin         */
#define REG_RF_EMITTER_PHASE   0x58 /* 16-bit IQ phase word (InP PIC array)    */
#define REG_ECC_SYNDROME       0x5C /* SECDED Hamming(72,64) syndrome          */
#define REG_FLIGHT_STAGE       0x60 /* 0=Cold 1=Inception 2=Cruise 3=Decel     */
#define REG_CRC32_CHECKSUM     0x64 /* Castagnoli CRC32 over register block    */

/* REG_SYS_CONTROL bits */
#define CTRL_ENABLE            (1u << 0)
#define CTRL_BUBBLE_ACTIVE     (1u << 1)
#define CTRL_QUENCH_TRIGGER    (1u << 2)

/* REG_SYS_STATUS bits */
#define STAT_BUBBLE_STABLE     (1u << 0)
#define STAT_ECC_OK            (1u << 1)
#define STAT_LAPSE_UNITY       (1u << 2)

/* REG_INTERLOCK_FLAGS bits */
#define ILK_SPACELIKE          (1u << 0)
#define ILK_UNDERPOWER         (1u << 1)
#define ILK_HORIZON_RISK       (1u << 2)

/*
 * shbt_warp_mmio_t — packed 128-byte register file, aligned to a 64-byte
 * boundary so both 64-byte cache lines are independently coherent for
 * zero-copy C-ABI streaming.
 */
typedef struct __attribute__((aligned(64), packed)) {
    /* Cache line 0 (0x00-0x3F) */
    volatile uint32_t sys_control;        /* 0x00 */
    volatile uint32_t sys_status;         /* 0x04 */
    volatile uint32_t lanr_power_mw;      /* 0x08 */
    volatile uint32_t _rsv0;              /* 0x0C */
    volatile uint32_t landauer_debt_mw;   /* 0x10 */
    volatile uint32_t _rsv1;              /* 0x14 */
    volatile uint32_t bubble_velocity_c;  /* 0x18 */
    volatile uint32_t _rsv2;              /* 0x1C */
    volatile uint32_t wall_thickness_nm;  /* 0x20 */
    volatile uint32_t _rsv3;              /* 0x24 */
    volatile uint32_t lapse_error_raw;    /* 0x28 */
    volatile uint32_t quench_timer_ns;    /* 0x2C */
    volatile uint32_t interlock_flags;    /* 0x30 */
    volatile uint32_t _rsv4[3];           /* 0x34-0x3C */
    /* Cache line 1 (0x40-0x7F) */
    volatile uint32_t shift_beta_x;       /* 0x40 */
    volatile uint32_t _rsv5;              /* 0x44 */
    volatile uint32_t ricci_scalar;       /* 0x48 */
    volatile uint32_t _rsv6;              /* 0x4C */
    volatile uint32_t qi_integral_bound;  /* 0x50 */
    volatile uint32_t _rsv7;              /* 0x54 */
    volatile uint32_t rf_emitter_phase;   /* 0x58 */
    volatile uint32_t ecc_syndrome;       /* 0x5C */
    volatile uint32_t flight_stage;       /* 0x60 */
    volatile uint32_t crc32_checksum;     /* 0x64 */
    volatile uint32_t _rsv8[6];           /* 0x68-0x7F */
} shbt_warp_mmio_t;

_Static_assert(sizeof(shbt_warp_mmio_t) == 128, "MMIO block must be 128 B");

#define SHBT_WARP_MMIO ((shbt_warp_mmio_t *)SHBT_WARP_MMIO_BASE)

/* ------------------------------------------------------------------ */
/* Public C-ABI (also exported by the reference shared object)          */
/* ------------------------------------------------------------------ */

#ifdef __cplusplus
extern "C" {
#endif

/* Kernel lifecycle. */
void shbt_warp_kernel_init(void);
void shbt_warp_service(void);
uint32_t shbt_warp_scrub(void);

/* Register access for the hosted test harness. */
shbt_warp_mmio_t *shbt_warp_mmio(void);
uint32_t shbt_warp_reg_read(uint32_t offset);
void shbt_warp_reg_write(uint32_t offset, uint32_t value);

/* Hardware primitives (public for the reference test). */
__uint128_t shbt_warp_secded_encode(uint64_t data);
uint64_t shbt_warp_secded_decode(__uint128_t code, uint64_t *syndrome,
                                 int *overall_parity);
void shbt_warp_givens_remap(double *x, double *y, size_t n, double c,
                            double s);
double shbt_warp_quench_trigger(void);
uint32_t shbt_warp_crc32c(const uint8_t *buf, size_t len);
uint16_t shbt_warp_emitter_phase_word(double phase_rad);

#ifdef __cplusplus
}
#endif

#endif /* SHBT_WARP_HARDWARE_H */
