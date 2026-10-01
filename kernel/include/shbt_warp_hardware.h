/*
 * shbt_warp_hardware.h — SHBT-MMIO-WARP register map (normative contract
 * from warp1.txt).
 *
 * Packed 128-byte dual-cacheline register block anchored at physical base
 * address 0x70000000. Cacheline 0 carries command, telemetry and
 * metrology registers; cacheline 1 carries diagnostics, thermal and the
 * dark-ledger registers. Layout is enforced by static assertions.
 */
#ifndef SHBT_WARP_MMIO_H
#define SHBT_WARP_MMIO_H

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#define SHBT_WARP_MMIO_BASE_ADDR (0x70000000UL)
#define SHBT_WARP_MMIO_BASE      SHBT_WARP_MMIO_BASE_ADDR
#define SHBT_WARP_EXPECTED_SIZE  (128UL)
#define SHBT_WARP_MMIO_SIZE      SHBT_WARP_EXPECTED_SIZE

/* Register offsets */
#define REG_CTRL_STATUS        0x00 /* Bit0=Arm 1=Fire 2=Abort 3=SECDED Err */
#define REG_TARGET_VELOCITY    0x08 /* Q32.32 target v_s in units of c      */
#define REG_CURRENT_VELOCITY   0x10 /* Q32.32 heterodyne telemetry          */
#define REG_CAVITY_ACCEL_RAW   0x18 /* Q32.32 passenger proper acceleration */
#define REG_BUBBLE_RADIUS_NM   0x20 /* Cavity radius R in nanometers        */
#define REG_WALL_THICKNESS_PM  0x28 /* Wall thickness Delta in picometers   */
#define REG_RF_PHASE_GRAD_URAD 0x30 /* RF phase steering gradient (urad)    */
#define REG_OPTICAL_POWER_MW   0x38 /* Optical carrier power (mW)           */
#define REG_CRYO_TEMP_MILLIK   0x40 /* Junction temperature T_j (mK)        */
#define REG_KAPITZA_DROP_UV    0x48 /* Kapitza interface sensor drop (uV)   */
#define REG_LANR_POWER_MW      0x50 /* LANR generated DC power (mW)         */
#define REG_DARK_LEDGER_SINK   0x58 /* Topological dark-ledger accumulator  */
#define REG_ECC_SYNDROME_REG   0x60 /* SECDED Hamming(72,64) syndrome       */
#define REG_PCSS_INTERLOCK_RAW 0x68 /* PCSS crowbar trip status             */
#define REG_WATCHDOG_HEARTBEAT 0x70 /* Sub-microsecond watchdog counter     */
#define REG_RESERVED_PADDING   0x78 /* Cacheline-1 closure                  */

/* ctrl_status bits */
#define CTRL_ARM               (1ull << 0)
#define CTRL_FIRE              (1ull << 1)
#define CTRL_ABORT             (1ull << 2)
#define CTRL_SECDED_ERR        (1ull << 3)

/* pcss_interlock_raw status bits */
#define PCSS_TRIPPED           (1ull << 0)
#define PCSS_SIC_RECOVERED     (1ull << 1)

#pragma pack(push, 1)
typedef struct {
    /* Cacheline 0: Command, Telemetry, and Metrology (Bytes 0-63) */
    volatile uint64_t ctrl_status;       /* Offset 0x00: Bit 0=Arm, 1=Fire, 2=Abort, 3=SECDED Err */
    volatile uint64_t target_velocity;   /* Offset 0x08: Q32.32 format in units of c                */
    volatile uint64_t current_velocity;  /* Offset 0x10: Q32.32 telemetry from heterodyne metrology  */
    volatile uint64_t cavity_accel_raw;  /* Offset 0x18: Q32.32 raw passenger proper acceleration    */
    volatile uint64_t bubble_radius_nm;  /* Offset 0x20: Cavity radius R in nanometers               */
    volatile uint64_t wall_thickness_pm; /* Offset 0x28: Bubble wall thickness Delta in picometers   */
    volatile uint64_t rf_phase_grad_urad;/* Offset 0x30: RF phase steering gradient (microradians)   */
    volatile uint64_t optical_power_mw;  /* Offset 0x38: Optical carrier total power (milliwatts)    */

    /* Cacheline 1: Diagnostics, Thermal, and Dark Ledger (Bytes 64-127) */
    volatile uint64_t cryo_temp_millik;  /* Offset 0x40: Junction temperature T_j (milli-Kelvin)     */
    volatile uint64_t kapitza_drop_uv;   /* Offset 0x48: Kapitza interface micro-volts / sensor drop */
    volatile uint64_t lanr_power_mw;     /* Offset 0x50: LANR generated DC power (milliwatts)        */
    volatile uint64_t dark_ledger_sink;  /* Offset 0x58: Topological dark ledger balance accumulator */
    volatile uint64_t ecc_syndrome_reg;  /* Offset 0x60: SECDED Hamming ECC syndrome code            */
    volatile uint64_t pcss_interlock_raw;/* Offset 0x68: Optical crowbar PCSS hardware trip status   */
    volatile uint64_t watchdog_heartbeat;/* Offset 0x70: Sub-microsecond hardware watchdog counter   */
    volatile uint64_t reserved_padding;  /* Offset 0x78: Reserved for cacheline 1 closure (64 bytes) */
} shbt_warp_mmio_t;
#pragma pack(pop)

_Static_assert(sizeof(shbt_warp_mmio_t) == SHBT_WARP_EXPECTED_SIZE,
               "FATAL: shbt_warp_mmio_t layout does not match 128-byte dual-cacheline spec!");
_Static_assert(offsetof(shbt_warp_mmio_t, cryo_temp_millik) == 64,
               "FATAL: Cacheline 1 alignment boundary violated at offset 64!");
_Static_assert(offsetof(shbt_warp_mmio_t, watchdog_heartbeat) == 112,
               "FATAL: Watchdog timer register displaced from offset 112!");

#define SHBT_WARP_MMIO ((shbt_warp_mmio_t *)SHBT_WARP_MMIO_BASE_ADDR)

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

/* Register access for the hosted test harness (offsets in bytes). */
shbt_warp_mmio_t *shbt_warp_mmio(void);
uint64_t shbt_warp_reg_read(uint32_t offset);
void shbt_warp_reg_write(uint32_t offset, uint64_t value);

/* Flight stage sequencer: 0=Cold 1=Inception 2=Cruise 3=Decel 4=Quench. */
uint32_t shbt_warp_flight_stage(void);
/* Latest CRC-32C trailer over the Stinespring arena. */
uint32_t shbt_warp_frame_crc(void);

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

#endif /* SHBT_WARP_MMIO_H */
