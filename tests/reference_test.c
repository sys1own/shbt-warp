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
    /* 1. 128-byte register block and offset contract. */
    assert(sizeof(shbt_warp_mmio_t) == 128);
    assert(REG_SYS_CONTROL == 0x00 && REG_SYS_STATUS == 0x04);
    assert(REG_LANR_POWER_MW == 0x08 && REG_LANDAUER_DEBT_MW == 0x10);
    assert(REG_BUBBLE_VELOCITY_C == 0x18 && REG_WALL_THICKNESS_NM == 0x20);
    assert(REG_LAPSE_ERROR_RAW == 0x28 && REG_QUENCH_TIMER_NS == 0x2C);
    assert(REG_INTERLOCK_FLAGS == 0x30 && REG_SHIFT_BETA_X == 0x40);
    assert(REG_RICCI_SCALAR == 0x48 && REG_QI_INTEGRAL_BOUND == 0x50);
    assert(REG_RF_EMITTER_PHASE == 0x58 && REG_ECC_SYNDROME == 0x5C);
    assert(REG_FLIGHT_STAGE == 0x60 && REG_CRC32_CHECKSUM == 0x64);

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

    /* 4. Kernel init writes the LANR/Landauer ledger registers. */
    shbt_warp_kernel_init();
    assert(shbt_warp_mmio()->sys_control & CTRL_ENABLE);
    assert(shbt_warp_mmio()->lanr_power_mw == 999054u);
    assert(shbt_warp_mmio()->landauer_debt_mw == 906000u);
    assert(shbt_warp_mmio()->sys_status & STAT_LAPSE_UNITY);

    /* 5. PCSS crowbar trigger: 8 x 0.27 ns = 2.16 ns under both the
     * 2.18 ns target and the 2.50 ns hard limit. */
    {
        double latency = shbt_warp_quench_trigger();
        assert(latency <= 2.18 + 1e-9);
        assert(latency < 2.50);
    }

    /* 6. Interlock-driven quench: a spacelike anomaly flag fires the
     * crowbar and moves the flight stage to Quench. */
    shbt_warp_reg_write(REG_INTERLOCK_FLAGS, ILK_SPACELIKE);
    shbt_warp_service();
    assert(shbt_warp_mmio()->sys_control & CTRL_QUENCH_TRIGGER);
    assert(shbt_warp_mmio()->flight_stage == 4);
    assert(shbt_warp_mmio()->quench_timer_ns <= 218u);

    /* 7. One service pass: ECC scrub over the 2112 B arena, Givens remap
     * of the braid descriptors, CRC-32C trailer update. */
    shbt_warp_reg_write(REG_INTERLOCK_FLAGS, 0);
    shbt_warp_service();
    assert(shbt_warp_mmio()->crc32_checksum != 0);
    assert(shbt_warp_mmio()->sys_status & STAT_ECC_OK);

    /* 8. Emitter phase word wraps into 16 bits. */
    assert(shbt_warp_emitter_phase_word(0.0) == 0);
    assert(shbt_warp_emitter_phase_word(3.141592653589793) >= 32760u);
    assert(shbt_warp_emitter_phase_word(6.283185307179586) == 0);

    puts("reference_test: all checks passed");
    return 0;
}
