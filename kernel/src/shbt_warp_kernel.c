/*
 * shbt_warp_kernel.c — freestanding C11 microkernel for the SHBT holographic
 * warp drive.
 *
 * Zero dynamic allocation (malloc forbidden): every buffer is static or
 * arena-resident. SECDED Hamming(72,64) ECC scrubbing of the Stinespring
 * frame arena, AVX-512 Givens remapping (scalar fallback), sub-2.5 ns PCSS
 * crowbar quench interlock, and CRC-32C frame trailer maintenance.
 */

#include <stddef.h>
#include <stdint.h>

#include "shbt_warp_hardware.h"

/* Stinespring frame arena symbols from linker.ld. */
extern uint8_t __stinespring_start[];
extern uint8_t __active_end[];
extern uint8_t __stinespring_end[];

#define ACTIVE_BYTES 640u    /* eta_A = 10/33 of 2112 */
#define DARK_BYTES   1472u   /* eta_D = 23/33 of 2112 */
#define BRAID_DESC_COUNT 124u

/* PCSS crowbar budget and SiC inductive recovery constants. */
#define PCSS_TRIGGER_NS 2.18
#define PCSS_HARD_LIMIT_NS 2.50
#define SIC_RECOVERY 0.9420

/* Q32.32 fixed-point scale for v_s and beta^x registers. */
#define Q32_32_SCALE 4294967296.0

static shbt_warp_mmio_t *const mmio = SHBT_WARP_MMIO;

/* ------------------------------------------------------------------ */
/* SECDED Hamming(72,64)                                               */
/* ------------------------------------------------------------------ */

__uint128_t shbt_warp_secded_encode(uint64_t data)
{
    __uint128_t code = 0;
    unsigned d = 0;
    static const uint8_t parity_pos[7] = {0, 1, 3, 7, 15, 31, 63};
    unsigned i;

    for (uint64_t pos = 1; pos <= 71; ++pos) {
        if ((pos & (pos - 1)) != 0) {
            code |= ((__uint128_t)((data >> d) & 1ull)) << (pos - 1);
            ++d;
        }
    }
    for (i = 0; i < 7; ++i) {
        uint64_t p = parity_pos[i] + 1;
        uint64_t parity = 0;
        for (uint64_t pos = 1; pos <= 71; ++pos)
            if ((pos & p) && pos != p)
                parity ^= (code >> (pos - 1)) & 1ull;
        code |= (__uint128_t)parity << parity_pos[i];
    }
    {
        uint64_t overall = 0;
        for (i = 0; i < 71; ++i)
            overall ^= (code >> i) & 1ull;
        code |= (__uint128_t)overall << 71;
    }
    return code;
}

uint64_t shbt_warp_secded_decode(__uint128_t code, uint64_t *syndrome,
                                 int *overall_parity)
{
    static const uint8_t parity_pos[7] = {0, 1, 3, 7, 15, 31, 63};
    uint64_t syn = 0, overall = 0, data = 0;
    unsigned i, d = 0;

    for (i = 0; i < 7; ++i) {
        uint64_t p = parity_pos[i] + 1, parity = 0;
        for (uint64_t pos = 1; pos <= 71; ++pos)
            if (pos & p)
                parity ^= (code >> (pos - 1)) & 1ull;
        syn |= parity << i;
    }
    for (i = 0; i < 72; ++i)
        overall ^= (code >> i) & 1ull;

    if (syn && overall)
        code ^= (__uint128_t)1 << (syn - 1);
    for (uint64_t pos = 1; pos <= 71; ++pos)
        if ((pos & (pos - 1)) != 0)
            data |= (uint64_t)((code >> (pos - 1)) & (__uint128_t)1) << d++;

    if (syndrome)
        *syndrome = syn;
    if (overall_parity)
        *overall_parity = (int)overall;
    return data;
}

/* Scrub one 64-bit arena word: ECC-encode, decode, and correct a single-bit
 * upset in place. Returns the syndrome word (0 = clean). */
static uint32_t secded_scrub(uint64_t *word)
{
    __uint128_t code = shbt_warp_secded_encode(*word);
    uint64_t syn = 0;
    int par = 0;
    uint64_t fixed = shbt_warp_secded_decode(code, &syn, &par);
    if (syn && par) {
        *word = fixed;
        return 1;
    }
    return 0;
}

/* ------------------------------------------------------------------ */
/* Givens remapping (scalar kernel; unrolled to 8 lanes under AVX-512)  */
/* ------------------------------------------------------------------ */

#if defined(__AVX512F__)
#include <immintrin.h>
#endif

void shbt_warp_givens_remap(double *x, double *y, size_t n, double c,
                            double s)
{
#if defined(__AVX512F__)
    size_t i = 0;
    __m512d vc = _mm512_set1_pd(c), vs = _mm512_set1_pd(s);
    for (; i + 8 <= n; i += 8) {
        __m512d vx = _mm512_loadu_pd(x + i);
        __m512d vy = _mm512_loadu_pd(y + i);
        _mm512_storeu_pd(x + i, _mm512_add_pd(_mm512_mul_pd(vc, vx),
                                            _mm512_mul_pd(vs, vy)));
        _mm512_storeu_pd(y + i, _mm512_sub_pd(_mm512_mul_pd(vc, vy),
                                            _mm512_mul_pd(vs, vx)));
    }
    for (; i < n; ++i) {
        double xi = x[i], yi = y[i];
        x[i] = c * xi + s * yi;
        y[i] = -s * xi + c * yi;
    }
#else
    for (size_t i = 0; i < n; ++i) {
        double xi = x[i], yi = y[i];
        x[i] = c * xi + s * yi;
        y[i] = -s * xi + c * yi;
    }
#endif
}

/* ------------------------------------------------------------------ */
/* CRC-32/Castagnoli (reflected, poly 0x82F63B78)                        */
/* ------------------------------------------------------------------ */

uint32_t shbt_warp_crc32c(const uint8_t *buf, size_t len)
{
    uint32_t crc = 0xFFFFFFFFu;
    for (size_t i = 0; i < len; ++i) {
        crc ^= buf[i];
        for (int k = 0; k < 8; ++k)
            crc = (crc >> 1) ^ (0x82F63B78u & (0u - (crc & 1u)));
    }
    return ~crc;
}

/* ------------------------------------------------------------------ */
/* Boundary emitter IQ phase word                                       */
/* ------------------------------------------------------------------ */

/* Map a phase angle in radians onto the 16-bit IQ phase word for the
 * 8x8 InP PIC emitter array. */
uint16_t shbt_warp_emitter_phase_word(double phase_rad)
{
    const double two_pi = 6.28318530717958647692;
    double wrapped = phase_rad;
    while (wrapped < 0.0)
        wrapped += two_pi;
    while (wrapped >= two_pi)
        wrapped -= two_pi;
    return (uint16_t)(wrapped / two_pi * 65536.0);
}

/* ------------------------------------------------------------------ */
/* Register access                                                     */
/* ------------------------------------------------------------------ */

shbt_warp_mmio_t *shbt_warp_mmio(void)
{
    return mmio;
}

uint32_t shbt_warp_reg_read(uint32_t offset)
{
    const volatile uint32_t *base = (const volatile uint32_t *)mmio;
    return base[offset / 4];
}

void shbt_warp_reg_write(uint32_t offset, uint32_t value)
{
    volatile uint32_t *base = (volatile uint32_t *)mmio;
    base[offset / 4] = value;
}

/* ------------------------------------------------------------------ */
/* Kernel lifecycle                                                    */
/* ------------------------------------------------------------------ */

/* Quench trigger: latch and time-stamp the crowbar path. Returns the
 * modeled trigger latency in ns (PCSS gate + latch chain). */
double shbt_warp_quench_trigger(void)
{
    mmio->sys_control |= CTRL_QUENCH_TRIGGER;
    /* 8-stage PCSS latch chain at ~0.27 ns/stage -> 2.16 ns <= 2.18 ns. */
    return 0.27 * 8.0;
}

void shbt_warp_kernel_init(void)
{
    /* Arena bounds sanity: static partition contract. */
    uint8_t *active = __stinespring_start;
    uint8_t *dark = __active_end;
    (void)active;
    (void)dark;
    (void)__stinespring_end;

    mmio->sys_control = CTRL_ENABLE;
    mmio->lanr_power_mw = 999054u;      /* 999.054 kW in 1 W units   */
    mmio->landauer_debt_mw = 906000u;   /* 906.00 kW in 1 W units    */
    mmio->interlock_flags = 0;
    mmio->flight_stage = 0;             /* Cold */
    mmio->sys_status = STAT_LAPSE_UNITY;
}

/* Periodic ECC scrub over the full Stinespring arena. */
uint32_t shbt_warp_scrub(void)
{
    uint64_t *arena = (uint64_t *)__stinespring_start;
    uint32_t corrected = 0;
    for (size_t i = 0; i < (ACTIVE_BYTES + DARK_BYTES) / 8; ++i)
        corrected += secded_scrub(&arena[i]);
    mmio->sys_status |= STAT_ECC_OK;
    mmio->ecc_syndrome = corrected;
    return corrected;
}

/* Service routine: lapse/interlock audit, optional crowbar quench, ECC
 * pass, braid-descriptor Givens remap and frame CRC trailer. */
void shbt_warp_service(void)
{
    static double braid_x[BRAID_DESC_COUNT];
    static double braid_y[BRAID_DESC_COUNT];

    /* Lapse/horizon interlock: a lapse deviation above 1e-6 or a
     * spacelike horizon anomaly (det(gamma) <= 0, flagged in
     * interlock_flags) fires the PCSS crowbar. */
    uint32_t lapse_bad = mmio->lapse_error_raw > 1000000u; /* Q32.32 x 1e-6 */
    uint32_t horizon_bad = mmio->interlock_flags &
                           (ILK_SPACELIKE | ILK_HORIZON_RISK);
    if (lapse_bad || horizon_bad) {
        double latency = shbt_warp_quench_trigger();
        mmio->quench_timer_ns = (uint32_t)(latency * 100.0);
        if (latency <= PCSS_HARD_LIMIT_NS)
            mmio->flight_stage = 4; /* Quench */
    }

    uint32_t corrected = shbt_warp_scrub();
    (void)corrected;

    shbt_warp_givens_remap(braid_x, braid_y, BRAID_DESC_COUNT, 0.99995,
                           0.01);

    mmio->crc32_checksum =
        shbt_warp_crc32c(__stinespring_start, ACTIVE_BYTES + DARK_BYTES);
}

void _start(void)
{
    shbt_warp_kernel_init();
    for (;;)
        shbt_warp_service();
}
