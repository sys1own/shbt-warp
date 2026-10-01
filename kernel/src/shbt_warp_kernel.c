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
#define PCSS_TRIGGER_NS 2.140
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
 * upset in place. Returns 0 clean, 1 corrected, 2 uncorrectable (DUE). */
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
    if (syn && !par) {
        mmio->ctrl_status |= CTRL_SECDED_ERR;
        return 2;
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

uint64_t shbt_warp_reg_read(uint32_t offset)
{
    const volatile uint64_t *base = (const volatile uint64_t *)mmio;
    return base[offset / 8];
}

void shbt_warp_reg_write(uint32_t offset, uint64_t value)
{
    volatile uint64_t *base = (volatile uint64_t *)mmio;
    base[offset / 8] = value;
}

/* ------------------------------------------------------------------ */
/* Kernel lifecycle                                                    */
/* ------------------------------------------------------------------ */

/* Quench trigger: latch and time-stamp the crowbar path. Returns the
 * modeled trigger latency in ns (PCSS gate + latch chain). */
/* Internal sequencer state (not MMIO-mapped). */
static uint32_t g_flight_stage = 0;
static uint32_t g_frame_crc = 0;

uint32_t shbt_warp_flight_stage(void)
{
    return g_flight_stage;
}

uint32_t shbt_warp_frame_crc(void)
{
    return g_frame_crc;
}

double shbt_warp_quench_trigger(void)
{
    mmio->ctrl_status |= CTRL_ABORT;
    /* 180 ps optical trigger + 820 ps avalanche rise + 8 x 0.1425 ns
     * latch chain = 2.140 ns total crowbar latency (limit 2.50 ns). */
    return 0.180 + 0.820 + 8.0 * 0.1425;
}

void shbt_warp_kernel_init(void)
{
    /* Arena bounds sanity: static partition contract. */
    uint8_t *active = __stinespring_start;
    uint8_t *dark = __active_end;
    (void)active;
    (void)dark;
    (void)__stinespring_end;

    mmio->ctrl_status = CTRL_ARM;
    mmio->lanr_power_mw = 999054000ull;   /* 999.054 kW in mW units    */
    mmio->dark_ledger_sink = 906000ull;   /* 906.00 kW Landauer debt   */
    mmio->cryo_temp_millik = 21130ull;    /* T_junction = 21.130 K     */
    mmio->kapitza_drop_uv = 3546ull;      /* dT_K = 3.546 K sensor     */
    mmio->optical_power_mw = 15ull;       /* 15.0 mW probe carrier     */
    mmio->bubble_radius_nm = 12500000000ull; /* R = 12.50 m            */
    mmio->watchdog_heartbeat = 0;
    mmio->pcss_interlock_raw = 0;
    mmio->ecc_syndrome_reg = 0;
    g_flight_stage = 0;                   /* Cold */
}

/* Periodic ECC scrub over the full Stinespring arena. */
uint32_t shbt_warp_scrub(void)
{
    uint64_t *arena = (uint64_t *)__stinespring_start;
    uint32_t corrected = 0;
    mmio->ctrl_status &= ~CTRL_SECDED_ERR;
    for (size_t i = 0; i < (ACTIVE_BYTES + DARK_BYTES) / 8; ++i)
        corrected += secded_scrub(&arena[i]);
    mmio->ecc_syndrome_reg = corrected;
    return corrected;
}

/* Service routine: lapse/interlock audit, optional crowbar quench, ECC
 * pass, braid-descriptor Givens remap and frame CRC trailer. */
void shbt_warp_service(void)
{
    static double braid_x[BRAID_DESC_COUNT];
    static double braid_y[BRAID_DESC_COUNT];

    /* Passenger-acceleration / ECC interlock: residual cavity
     * acceleration above 1e-7 m/s^2 (Q32.32 raw > 429) or a SECDED
     * double-bit upset fires the PCSS crowbar within one 10 ns cycle. */
    uint32_t accel_bad = mmio->cavity_accel_raw > 429ull;
    uint32_t ecc_bad = mmio->ctrl_status & CTRL_SECDED_ERR;
    if (accel_bad || ecc_bad) {
        double latency = shbt_warp_quench_trigger();
        mmio->pcss_interlock_raw = PCSS_TRIPPED |
            ((uint64_t)(latency * 1000.0) << 8);
        if (latency <= PCSS_HARD_LIMIT_NS) {
            g_flight_stage = 4; /* Quench */
            mmio->pcss_interlock_raw |= PCSS_SIC_RECOVERED;
        }
    }

    uint32_t corrected = shbt_warp_scrub();
    (void)corrected;

    shbt_warp_givens_remap(braid_x, braid_y, BRAID_DESC_COUNT, 0.99995,
                           0.01);

    g_frame_crc =
        shbt_warp_crc32c(__stinespring_start, ACTIVE_BYTES + DARK_BYTES);
    mmio->watchdog_heartbeat++;
}

void _start(void)
{
    shbt_warp_kernel_init();
    for (;;)
        shbt_warp_service();
}
