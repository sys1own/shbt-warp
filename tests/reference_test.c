/*
 * reference_test.c — hosted reference test for the shbt-warp C11
 * microkernel.
 *
 * Compiled per the Phase-4 workflow:
 *   gcc -std=c11 -Wall -Wextra -pedantic tests/reference_test.c \
 *       -Ikernel/include -Lkernel/build -lshbt_warp_reference -o test_c_abi
 *
 * Validates:
 *   - 128-byte shbt_warp_mmio_t alignment contract and register offsets
 *   - SECDED Hamming(72,64): single-bit flip corrected w/ syndrome set,
 *     double-bit flip flagged uncorrectable (DUE)
 *   - Givens vector remapping preserves the vector norm (AVX-512 or
 *     scalar fallback path)
 *   - PCSS crowbar trigger latches sub-2.5 ns (<= 2.18 ns target)
 */
#define _GNU_SOURCE /* MAP_ANONYMOUS under -std=c11 -pedantic */
#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include <sys/mman.h>

#include "shbt_warp_hardware.h"

/* Arena symbols are provided by the hosted arena TU inside
 * libshbt_warp_reference.so (kernel/src/arena_stubs.c). */

int main(void)
{
    /* 1. 128-byte dual-cacheline register block and offset contract. */
    assert(sizeof(shbt_warp_mmio_t) == 128);
    assert(REG_CTRL_STATUS == 0x00 && REG_TARGET_VELOCITY == 0x08);
    assert(REG_CURRENT_VELOCITY == 0x10 && REG_CAVITY_ACCEL_RAW == 0x18);
    assert(REG_BUBBLE_RADIUS_NM == 0x20 && REG_WALL_THICKNESS_PM == 0x28);
    assert(REG_RF_PHASE_GRAD_URAD == 0x30 && REG_OPTICAL_POWER_MW == 0x38);
    assert(REG_CRYO_TEMP_MILLIK == 0x40 && REG_KAPITZA_DROP_UV == 0x48);
    assert(REG_LANR_POWER_MW == 0x50 && REG_DARK_LEDGER_SINK == 0x58);
    assert(REG_ECC_SYNDROME_REG == 0x60 && REG_PCSS_INTERLOCK_RAW == 0x68);
    assert(REG_WATCHDOG_HEARTBEAT == 0x70 && REG_RESERVED_PADDING == 0x78);

    /* Map the fixed MMIO page so the kernel can touch 0x70000000. */
    void *page = mmap((void *)SHBT_WARP_MMIO_BASE, 4096,
                      PROT_READ | PROT_WRITE,
                      MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0);
    assert(page == (void *)SHBT_WARP_MMIO_BASE);

    /* 2. SECDED Hamming(72,64). */
    {
        uint64_t data = 0xDEADBEEF12345678ull;
        __uint128_t code = shbt_warp_secded_encode(data);
        uint64_t syn; int par;

        /* Clean codeword decodes to the same data. */
        assert(shbt_warp_secded_decode(code, &syn, &par) == data
               && syn == 0 && par == 0);

        /* Single-bit flip: corrected, syndrome set, parity odd. */
        code ^= (__uint128_t)1 << 20;
        assert(shbt_warp_secded_decode(code, &syn, &par) == data
               && syn != 0 && par == 1);

        /* Double-bit flip: uncorrectable (DUE) — syndrome nonzero while
         * overall parity is even. */
        code = shbt_warp_secded_encode(data) ^ ((__uint128_t)3 << 20);
        shbt_warp_secded_decode(code, &syn, &par);
        assert(syn != 0 && par == 0);
    }

    /* 3. Givens vector remapping preserves the norm. */
    {
        double x[8] = {1, 2, 3, 4, 5, 6, 7, 8};
        double y[8] = {8, 7, 6, 5, 4, 3, 2, 1};
        double n0 = 0.0, n1 = 0.0;
        for (int i = 0; i < 8; ++i) n0 += x[i] * x[i] + y[i] * y[i];
        shbt_warp_givens_remap(x, y, 8, 0.6, 0.8);
        for (int i = 0; i < 8; ++i) n1 += x[i] * x[i] + y[i] * y[i];
        assert(fabs(n1 - n0) < 1e-9);
    }

    /* 4. Kernel init writes the ledger/thermal register contract. */
    shbt_warp_kernel_init();
    assert(shbt_warp_mmio()->ctrl_status & CTRL_ARM);
    assert(shbt_warp_mmio()->lanr_power_mw == 999054000ull);
    assert(shbt_warp_mmio()->dark_ledger_sink == 906000ull);
    assert(shbt_warp_mmio()->cryo_temp_millik == 21130ull);
    assert(shbt_warp_flight_stage() == 0);

    /* 5. PCSS crowbar trigger: 180 ps + 820 ps + 8x0.1425 ns = 2.140 ns,
     * under the 2.50 ns sub-cycle hard limit. */
    {
        double latency = shbt_warp_quench_trigger();
        assert(latency <= 2.140 + 1e-9);
        assert(latency < 2.50);
        assert(shbt_warp_mmio()->ctrl_status & CTRL_ABORT);
        shbt_warp_mmio()->ctrl_status &= ~CTRL_ABORT;
    }

    /* 6. Interlock-driven quench: excessive residual cavity acceleration
     * (Q32.32 > 1e-7 m/s^2) fires the crowbar and moves the flight stage
     * to Quench with SiC inductive recovery latched. */
    shbt_warp_reg_write(REG_CAVITY_ACCEL_RAW, 1ull << 20);
    shbt_warp_service();
    assert(shbt_warp_mmio()->ctrl_status & CTRL_ABORT);
    assert(shbt_warp_flight_stage() == 4);
    assert(shbt_warp_mmio()->pcss_interlock_raw & PCSS_TRIPPED);
    assert(shbt_warp_mmio()->pcss_interlock_raw & PCSS_SIC_RECOVERED);
    assert(((shbt_warp_mmio()->pcss_interlock_raw >> 8) & 0xFFFFFFull) <= 2500ull);

    /* 7. One service pass: ECC scrub over the 2112 B arena, Givens remap
     * of the braid descriptors, CRC-32C trailer update, watchdog tick. */
    shbt_warp_reg_write(REG_CAVITY_ACCEL_RAW, 0);
    shbt_warp_mmio()->ctrl_status &= ~CTRL_ABORT;
    uint64_t hb0 = shbt_warp_mmio()->watchdog_heartbeat;
    shbt_warp_service();
    assert(shbt_warp_frame_crc() != 0);
    assert(!(shbt_warp_mmio()->ctrl_status & CTRL_SECDED_ERR));
    assert(shbt_warp_mmio()->watchdog_heartbeat > hb0);

    /* 8. Emitter phase word wraps into 16 bits. */
    assert(shbt_warp_emitter_phase_word(0.0) == 0);
    assert(shbt_warp_emitter_phase_word(3.141592653589793) >= 32760u);
    assert(shbt_warp_emitter_phase_word(6.283185307179586) == 0);

    puts("reference_test: all checks passed");
    return 0;
}
