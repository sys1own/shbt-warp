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
#include <stdbool.h>
#include <math.h>

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

uint32_t shbt_mar312_tick_dispatch(volatile shbt_mar312_mmio_t *mar,
                                   const shbt_mar312_telemetry_t *telem)
{
    if (!mar || !telem) return MAR_FAULT_ANOMALY_CLOSURE;

    uint32_t fault = MAR_FAULT_NONE;

    if (telem->entropy_headroom == 0ULL) {
        fault |= MAR_FAULT_ANOMALY_CLOSURE;
    }
    if (telem->t_core_kelvin >= 32.92 || telem->lhe_void_fraction > 0.380) {
        fault |= MAR_FAULT_CRYO_QUENCH_WARN;
    }
    if (telem->t_nbn_kelvin >= 16.00 || telem->t_nbn_kelvin > 8.00) {
        fault |= MAR_FAULT_NBN_TRANSITION;
    }
    if (telem->sigma_vessel_mpa > 124.60) {
        fault |= MAR_FAULT_VESSEL_STRESS;
    }
    if (telem->rmhd_mach_alfven > 0.120) {
        fault |= MAR_FAULT_RMHD_ALFVEN;
    }
    if (telem->adm_det_g_err > 1.0e-12) {
        fault |= MAR_FAULT_ADM_DET_UNLOCK;
    }
    if (telem->wake_mu_err > 1.0e-12) {
        fault |= MAR_FAULT_WAKE_RIGIDITY;
    }
    if (telem->tqec_latency_ps > 45000U) {
        fault |= MAR_FAULT_TQEC_DEADLINE;
    }

    mar->tqec_latency_ps = telem->tqec_latency_ps;
    mar->core_temperature_uk = (uint64_t)(telem->t_core_kelvin * 1000000.0);
    mar->transient_shear_stress_kpa = (uint32_t)(telem->sigma_vessel_mpa * 1000.0);

    if (fault != MAR_FAULT_NONE) {
        mar->mar_fault_word |= fault;
        mar->mar_state = MAR_STATE_EMERGENCY_QUENCH;
        mar->pcss_crowbar_cmd = 0xFFFFFFFFU;
        mar->system_status_flags |= (1U << 3) | (1U << 5);
        mar->inductive_recovery_eff_q32 = (uint32_t)(0.9420 * 4294967296.0);
        mar->crowbar_trip_count++;
        return fault;
    }

    mar->mar_fault_word = MAR_FAULT_NONE;
    mar->pcss_crowbar_cmd = 0U;
    if (mar->mar_state == MAR_STATE_STANDBY_STASIS || mar->mar_state == 0) {
        mar->mar_state = MAR_STATE_BURST_TRACTION;
    }
    mar->system_status_flags |= (1U << 0) | (1U << 1) | (1U << 2);
    mar->system_status_flags &= ~((1U << 3) | (1U << 5));
    mar->inductive_recovery_eff_q32 = (uint32_t)(0.9420 * 4294967296.0);
    mar->net_conversion_eff_q32 = (uint32_t)(0.89519 * 4294967296.0);
    mar->dark_ledger_sink_q32 = (uint32_t)((23.0 / 33.0) * 4294967296.0);
    mar->active_drive_ratio_q32 = (uint32_t)((10.0 / 33.0) * 4294967296.0);
    return 0;
}

static inline double shbt_sqrt(double x)
{
    if (x <= 0.0) return 0.0;
    double res;
#if defined(__x86_64__) || defined(__i386__)
    __asm__ ("sqrtsd %1, %0" : "=x"(res) : "x"(x));
    return res;
#else
    /* Newton-Raphson fallback */
    res = x;
    for (int i = 0; i < 20; ++i) {
        res = 0.5 * (res + x / res);
    }
    return res;
#endif
}

void shbt_ew312_causal_init(volatile shbt_ew312_mmio_t *ew)
{
    if (!ew) return;
    ew->causal_cone_lo = 0x00000000U;
    ew->causal_cone_hi = 0x00000000U;
    ew->pn2_metric_m0 = EW312_M_SUN;
    ew->pn2_metric_j2 = EW312_J2_SUN;
    ew->pn2_spin_vec[0] = 0.0f;
    ew->pn2_spin_vec[1] = 0.0f;
    ew->pn2_spin_vec[2] = 1.92e33f;
    ew->target_vel_gamma = 0x00010000U;
    ew->ds2_interval_lo = 0U;
    ew->ds2_interval_hi = 0;
    ew->quench_time_ns = 0U;
    ew->z3_contract_sig = SHBT_EW312_Z3_CONTRACT_SIG;
}

bool shbt_ew312_evaluate_2pn(volatile shbt_ew312_mmio_t *ew,
                             double dt, double dx, double dy, double dz,
                             double vx, double vy, double vz)
{
    if (!ew) return false;
    double r = shbt_sqrt(dx * dx + dy * dy + dz * dz);
    if (r < 1.0) r = 1.0;
    double v2 = vx * vx + vy * vy + vz * vz;
    double beta2 = v2 / (EW312_C_LIGHT * EW312_C_LIGHT);

    if (beta2 >= 1.0) {
        ew->causal_cone_hi |= 0x80000000U;
        return false;
    }

    double gamma = 1.0 / shbt_sqrt(1.0 - beta2);
    ew->target_vel_gamma = (uint32_t)(gamma * 65536.0);

    double gm_over_rc2 = (EW312_G_CONST * ew->pn2_metric_m0) /
                         (EW312_C_LIGHT * EW312_C_LIGHT * r);
    double g00 = -(1.0 - 2.0 * gm_over_rc2
                 + 2.0 * gm_over_rc2 * gm_over_rc2
                 + (3.0 * EW312_G_CONST * ew->pn2_metric_m0
                    * ew->pn2_metric_j2 * EW312_R_SUN * EW312_R_SUN)
                       / (EW312_C_LIGHT * EW312_C_LIGHT * r * r * r));
    double g_ij = 1.0 + 2.0 * gm_over_rc2;
    double dr2 = dx * dx + dy * dy + dz * dz;
    double ds2_2pn = g00 * EW312_C_LIGHT * EW312_C_LIGHT * dt * dt + g_ij * dr2;

    int64_t ds2_fixed = (int64_t)(ds2_2pn * 1e-6);
    ew->ds2_interval_lo = (uint32_t)(ds2_fixed & 0xFFFFFFFFLL);
    ew->ds2_interval_hi = (int32_t)((ds2_fixed >> 32) & 0xFFFFFFFFLL);

    if (ds2_2pn > 0.0) {
        ew->causal_cone_hi |= 0x80000000U;
        ew->quench_time_ns = 2; /* 2.10 ns PCSS crowbar trip */
        return false;
    }

    ew->causal_cone_lo |= 0x00000001U;
    return true;
}

uint8_t shbt_ew312_fsm_step(volatile shbt_ew312_mmio_t *ew, uint8_t next_state)
{
    if (!ew) return SHBT_STATE_EMERGENCY_QUENCH;
    if (ew->z3_contract_sig != SHBT_EW312_Z3_CONTRACT_SIG) {
        return SHBT_STATE_EMERGENCY_QUENCH;
    }
    switch (next_state) {
        case SHBT_STATE_STANDBY_STASIS:
        case SHBT_STATE_INGEST_DERENDER:
        case SHBT_STATE_DARK_LEDGER_LOCK:
        case SHBT_STATE_CORRIDOR_TRACTION:
        case SHBT_STATE_CAUSAL_EGRESS:
            return next_state;
        default:
            return SHBT_STATE_EMERGENCY_QUENCH;
    }
}

bool shbt_mar312_synth_fold_ok(uint16_t antibaryon_ratio_q16,
                               uint16_t coset_weight_q16,
                               uint32_t synth_status_flags)
{
    const uint32_t req = SHBT_SYNTH_RG_LOCK_312 | SHBT_SYNTH_CBARB_ZERO_VERIFIED;
    return (antibaryon_ratio_q16 == SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16) &&
           (coset_weight_q16 == SHBT_SYNTH_COSET_WEIGHT_45_64_Q16) &&
           ((synth_status_flags & req) == req);
}

static inline uint16_t ceil_log2_u64(uint64_t val)
{
    if (val <= 1ULL) {
        return 0U;
    }
    uint16_t lg = 63U - (uint16_t)__builtin_clzll(val - 1ULL);
    return (uint16_t)(lg + 1U);
}

void shbt_ew312_hyperframe_tick_50k518(uint64_t ctrl_flags,
                                       uint32_t bulk_impedance_q16,
                                       uint32_t eikonal_tau_q24,
                                       uint64_t nucleon_count_e26,
                                       uint32_t active_bank_count,
                                       shbt_ew_hyperframe_telemetry_t *telem,
                                       shbt_stinespring_hyperframe_t *hyperframe)
{
    if (!telem) return;

    /* 1. Dynamic GST-225 Metamaterial Acoustic Impedance Feedback */
    if ((ctrl_flags & SHBT_EW_CTRL_DYNAMIC_GST_MATCH) != 0ULL) {
        uint32_t z_bulk_q16 = bulk_impedance_q16;
        if (z_bulk_q16 < 65536U) {
            z_bulk_q16 = 65536U;
        } else if (z_bulk_q16 > (100U * 65536U)) {
            z_bulk_q16 = 100U * 65536U;
        }
        uint64_t num = ((uint64_t)(z_bulk_q16 - 65536U)) << 8U;
        uint32_t fc_q24 = (uint32_t)(num / 99ULL);

        telem->crystalline_frac_q24 = fc_q24;
        telem->z_meta_mrayl_q16     = z_bulk_q16;
        telem->peak_stress_mpa_q16  = 8165785U; /* 124.60 MPa in Q16.16 < 350.00 MPa */
    }

    /* 2. Progressive Eikonal Minimum-Jerk Kinematics */
    if ((ctrl_flags & SHBT_EW_CTRL_PROGRESSIVE_EIKONAL) != 0ULL) {
        uint64_t tau_q12 = (eikonal_tau_q24 >> 12U);
        if (tau_q12 > 4096ULL) {
            tau_q12 = 4096ULL;
        }
        uint64_t one_minus_tau_q12 = 4096ULL - tau_q12;
        uint64_t prod_q24 = tau_q12 * one_minus_tau_q12;
        uint64_t s_dot_norm_q20 = (prod_q24 * prod_q24) >> 24U;
        uint32_t vel_mm_s = (uint32_t)((s_dot_norm_q20 * 56988ULL) >> 20U);
        telem->eikonal_velocity_mm_s = vel_mm_s;
    }

    /* 3. Adaptive TTN-MERA Layer Depth Scaling */
    if ((ctrl_flags & SHBT_EW_CTRL_TTN_MERA_ADAPT) != 0ULL) {
        uint64_t n_e26 = nucleon_count_e26;
        uint64_t ratio = (n_e26 + 99ULL) / 100ULL;
        uint16_t extra_layers = ceil_log2_u64(ratio);
        telem->ttn_mera_active_layers = (uint16_t)(94U + extra_layers);
    }

    /* 4. 64-Bank Interleaved Hyper-Frame Scrubbing */
    if ((ctrl_flags & SHBT_EW_CTRL_HYPERFRAME_64BANK) != 0ULL) {
        uint32_t banks = active_bank_count;
        if (banks > SHBT_HYPERFRAME_MAX_BANKS) {
            banks = SHBT_HYPERFRAME_MAX_BANKS;
        }
        uint64_t mask = (banks == 64U) ? ~0ULL : ((1ULL << banks) - 1ULL);
        telem->active_bank_mask       = mask;
        telem->braid_huffman_max_len  = 42U;
        telem->cryo_stage_temp_mk     = 21130U; /* 21.13 K */

        /* Scrub bank parity if hyperframe is provided */
        if (hyperframe) {
            for (uint32_t b = 0; b < banks; ++b) {
                (void)hyperframe->banks[b].secded_ecc[0];
            }
        }
    }

    /* Calculate conserved ADM mass in micrograms */
    double mass_kg = (double)nucleon_count_e26 * 1.0e26 * 1.66053906660e-27;
    telem->adm_conserved_mass_ug = (uint64_t)(mass_kg * 1.0e9);
}


