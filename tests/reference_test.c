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
#include <stddef.h>
#include <stdio.h>
#include <stdint.h>
#include <math.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <sys/mman.h>
#endif

#include "shbt_warp_hardware.h"
#include "shbt_isomer_battery_mmio.h"
#include "shbt_mar312_mmio.h"
#include "shbt_ibr312_mmio.h"

/* Arena symbols are provided by the hosted arena TU inside
 * libshbt_warp_reference.so (kernel/src/arena_stubs.c). */

int main(void)
{
    /* 1. 128-byte dual-cacheline register block and offset contract. */
    assert(sizeof(shbt_warp_mmio_t) == 128);
    assert(sizeof(shbt_mar312_mmio_t) == 128);
    assert(sizeof(shbt_ibr312_mmio_t) == 128);
    assert(sizeof(shbt_stinespring_magazine_t) == 33792);
    assert(REG_CTRL_STATUS == 0x00 && REG_TARGET_VELOCITY == 0x08);
    assert(REG_CURRENT_VELOCITY == 0x10 && REG_CAVITY_ACCEL_RAW == 0x18);
    assert(REG_BUBBLE_RADIUS_NM == 0x20 && REG_WALL_THICKNESS_PM == 0x28);
    assert(REG_RF_PHASE_GRAD_URAD == 0x30 && REG_OPTICAL_POWER_MW == 0x38);
    assert(REG_CRYO_TEMP_MILLIK == 0x40 && REG_KAPITZA_DROP_UV == 0x48);
    assert(REG_LANR_POWER_MW == 0x50 && REG_DARK_LEDGER_SINK == 0x58);
    assert(REG_ECC_SYNDROME_REG == 0x60 && REG_PCSS_INTERLOCK_RAW == 0x68);
    assert(REG_WATCHDOG_HEARTBEAT == 0x70 && REG_RESERVED_PADDING == 0x78);

    /* Map the fixed MMIO page so the kernel can touch 0x70000000. */
#ifdef _WIN32
    void *page = VirtualAlloc((void *)SHBT_WARP_MMIO_BASE, 4096,
                              MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
#else
    void *page = mmap((void *)SHBT_WARP_MMIO_BASE, 4096,
                      PROT_READ | PROT_WRITE,
                      MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0);
#endif
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

    /* 9. Isomer battery MMIO contract: bitfields, offsets, and volatile
     * accessors over a hosted instance of the 128-byte register block. */
    {
        volatile shbt_isomer_battery_mmio_t bat = {0};

        /* Status flag bitfield positions. */
        assert(SHBT_ISOMER_STAT_READY == 0x01);
        assert(SHBT_ISOMER_STAT_BORRMANN_LOCKED == 0x02);
        assert(SHBT_ISOMER_STAT_CROWBAR_ARMED == 0x04);
        assert(SHBT_ISOMER_STAT_CROWBAR_TRIPPED == 0x08);
        assert(SHBT_ISOMER_STAT_CRYO_WARNING == 0x10);
        assert(SHBT_ISOMER_STAT_QUENCH_FAULT == 0x20);
        assert(SHBT_ISOMER_STAT_LASING_ACTIVE == 0x40);
        assert(SHBT_ISOMER_STAT_DARK_SINK_SYNC == 0x80);

        /* Register offsets inside the struct (dual-cacheline contract). */
        assert(offsetof(shbt_isomer_battery_mmio_t, energy_remaining_joules) == 0x00);
        assert(offsetof(shbt_isomer_battery_mmio_t, state_of_charge_q32) == 0x08);
        assert(offsetof(shbt_isomer_battery_mmio_t, bus_voltage_uv) == 0x10);
        assert(offsetof(shbt_isomer_battery_mmio_t, bus_current_ua) == 0x18);
        assert(offsetof(shbt_isomer_battery_mmio_t, core_temperature_uk) == 0x20);
        assert(offsetof(shbt_isomer_battery_mmio_t, trigger_delay_ps) == 0x30);
        assert(offsetof(shbt_isomer_battery_mmio_t, graser_coherent_flux_w_m2) == 0x38);
        assert(offsetof(shbt_isomer_battery_mmio_t, transient_shear_stress_kpa) == 0x40);
        assert(offsetof(shbt_isomer_battery_mmio_t, system_status_flags) == 0x48);
        assert(offsetof(shbt_isomer_battery_mmio_t, inductive_recovery_eff_q32) == 0x50);
        assert(offsetof(shbt_isomer_battery_mmio_t, dark_ledger_sink_q32) == 0x58);
        assert(offsetof(shbt_isomer_battery_mmio_t, interlock_cmd_reg) == 0x64);

        /* Volatile accessor roundtrip. */
        bat.energy_remaining_joules = 500000000000000ull;   /* 500.0 TJ */
        bat.state_of_charge_q32 = 1ull << 32;               /* 100.0% */
        bat.bus_voltage_uv = 400000000000ull;               /* 400 kV */
        bat.core_temperature_uk = 21130000ull;              /* 21.13 K */
        bat.cryo_headroom_uk = 11790000ull;                 /* 11.79 K */
        bat.trigger_pulse_width_ps = 10000u;                /* 10 ns */
        bat.borrmann_epsilon_q16 = (uint32_t)(0.985 * 65536.0 + 0.5);
        bat.transient_shear_stress_kpa = 124600u;           /* 124.60 MPa */
        bat.system_status_flags = SHBT_ISOMER_STAT_READY
                                | SHBT_ISOMER_STAT_BORRMANN_LOCKED
                                | SHBT_ISOMER_STAT_CROWBAR_ARMED;

        assert(bat.energy_remaining_joules == 500000000000000ull);
        assert(bat.state_of_charge_q32 == (1ull << 32));
        assert(bat.core_temperature_uk + bat.cryo_headroom_uk == 32920000ull);
        assert(bat.transient_shear_stress_kpa <= 124600u);
        assert(bat.system_status_flags & SHBT_ISOMER_STAT_CROWBAR_ARMED);
        assert(!(bat.system_status_flags & SHBT_ISOMER_STAT_CROWBAR_TRIPPED));

        /* crowbar_trip_count increments are visible through the block. */
        bat.crowbar_trip_count += 1;
        assert(bat.crowbar_trip_count == 1u);
    }

    /* 10. MAR-312 MMIO & state machine dispatch qualification */
    {
        volatile shbt_mar312_mmio_t mar = {0};
        assert(sizeof(shbt_mar312_mmio_t) == 128);
        assert(sizeof(UnifiedStinespringFrame_t) == 2112);
        assert(SHBT_MAR312_ACTIVE_BYTES == 640);
        assert(SHBT_MAR312_DARK_BYTES == 1472);

        /* Nominal dispatch test */
        shbt_mar312_telemetry_t telem_nom = {
            .t_core_kelvin = 21.13,
            .t_nbn_kelvin = 4.21,
            .sigma_vessel_mpa = 120.0,
            .rmhd_mach_alfven = 0.044,
            .adm_det_g_err = 0.0,
            .wake_mu_err = 0.0,
            .lhe_void_fraction = 0.162,
            .tqec_latency_ps = 23700,
            .entropy_headroom = 1000ULL,
        };
        uint32_t f_nom = shbt_mar312_tick_dispatch(&mar, &telem_nom);
        assert(f_nom == MAR_FAULT_NONE);
        assert(mar.reactor_status_flags & 0x01ULL);

        /* Fault condition test: thermal over-limit triggers emergency quench */
        shbt_mar312_telemetry_t telem_fault = telem_nom;
        telem_fault.t_core_kelvin = 33.0; /* above 32.92 K quench threshold */
        uint32_t f_trip = shbt_mar312_tick_dispatch(&mar, &telem_fault);
        assert(f_trip & MAR_FAULT_CRYO_QUENCH_WARN);

        /* Test Register 0x30, 0x32, 0x34, 0x36 coset synthesis & first-principles registers */
        mar.synth_antibaryon_ratio_q16 = SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16;
        mar.synth_coset_weight_q16 = SHBT_SYNTH_COSET_WEIGHT_45_64_Q16;
        mar.synth_jitter_bound_q16 = 4;
        mar.synth_tether_displacement_q16 = 36;
        assert(mar.synth_antibaryon_ratio_q16 == 0x8000);
        assert(mar.synth_coset_weight_q16 == 0xB400);
        assert(mar.synth_jitter_bound_q16 == 4);
        assert(mar.synth_tether_displacement_q16 == 36);
        assert(shbt_mar312_synth_fold_ok(mar.synth_antibaryon_ratio_q16, mar.synth_coset_weight_q16,
                                         SHBT_SYNTH_RG_LOCK_312 | SHBT_SYNTH_CBARB_ZERO_VERIFIED) == true);
        assert(COSET_ANTIBARYON_FOLD == 0x06);
        assert(SYMPLECTIC_CRYSTALLIZATION == 0x08);
    }

    /* 11. EW-312 MMIO, 2PN Causal Lightcone & 6-State FSM */
    {
        assert(sizeof(shbt_ew312_mmio_t) == 56);
        assert(offsetof(shbt_ew312_mmio_t, z3_contract_sig) == 0x34);
        assert(offsetof(UnifiedStinespringFrame_t, dark_braids) == 0x280);
        assert(offsetof(UnifiedStinespringFrame_t, ecc_syndrome_meta) == 0x660);

        volatile shbt_ew312_mmio_t ew = {0};
        shbt_ew312_causal_init(&ew);
        assert(ew.z3_contract_sig == SHBT_EW312_Z3_CONTRACT_SIG);

        /* Timelike authorized interval */
        bool auth = shbt_ew312_evaluate_2pn(&ew, 1.0, 1000.0, 0.0, 0.0, 1000.0, 0.0, 0.0);
        assert(auth == true);
        assert(ew.causal_cone_lo & 0x01);

        /* Spacelike unauthorized interval */
        bool unauth = shbt_ew312_evaluate_2pn(&ew, 1.0e-5, 1.0e9, 0.0, 0.0, 1000.0, 0.0, 0.0);
        assert(unauth == false);
        assert(ew.causal_cone_hi & 0x80000000U);
        assert(ew.quench_time_ns <= 2);

        /* 6-State FSM transitions */
        assert(shbt_ew312_fsm_step(&ew, SHBT_STATE_STANDBY_STASIS) == SHBT_STATE_STANDBY_STASIS);
        assert(shbt_ew312_fsm_step(&ew, SHBT_STATE_INGEST_DERENDER) == SHBT_STATE_INGEST_DERENDER);
        assert(shbt_ew312_fsm_step(&ew, SHBT_STATE_DARK_LEDGER_LOCK) == SHBT_STATE_DARK_LEDGER_LOCK);
        assert(shbt_ew312_fsm_step(&ew, SHBT_STATE_CORRIDOR_TRACTION) == SHBT_STATE_CORRIDOR_TRACTION);
        assert(shbt_ew312_fsm_step(&ew, SHBT_STATE_CAUSAL_EGRESS) == SHBT_STATE_CAUSAL_EGRESS);
        assert(shbt_ew312_fsm_step(&ew, 0x99) == SHBT_STATE_EMERGENCY_QUENCH);
    }

    /* 12. 64-Bank Hyper-Frame & Scaling Telemetry */
    {
        assert(sizeof(UnifiedStinespringFrame) == 2112);
        assert(sizeof(shbt_stinespring_hyperframe_t) == 135168);

        shbt_ew_hyperframe_telemetry_t telem = {0};
        uint64_t ctrl = SHBT_EW_CTRL_ENABLE_PIPELINE
                      | SHBT_EW_CTRL_DYNAMIC_GST_MATCH
                      | SHBT_EW_CTRL_PROGRESSIVE_EIKONAL
                      | SHBT_EW_CTRL_TTN_MERA_ADAPT
                      | SHBT_EW_CTRL_HYPERFRAME_64BANK
                      | SHBT_EW_CTRL_TERNARY_HUFFMAN_PACK;

        uint32_t z_bulk_q16 = (uint32_t)(44.178 * 65536.0);
        uint32_t tau_q24 = (uint32_t)(0.5 * 16777216.0);
        uint64_t n_e26 = 10000ULL; /* 10^30 nucleons */
        uint32_t banks = 64U;

        shbt_ew312_hyperframe_tick_50k518(ctrl, z_bulk_q16, tau_q24, n_e26, banks, &telem, NULL);

        assert(telem.active_bank_mask == ~0ULL);
        assert(telem.z_meta_mrayl_q16 == z_bulk_q16);
        assert(telem.ttn_mera_active_layers == 101);
        assert(telem.braid_huffman_max_len == 42);
        assert(telem.cryo_stage_temp_mk == 21130);
        assert(telem.peak_stress_mpa_q16 <= 8165785U);
    }

    puts("reference_test: all checks passed");
    return 0;
}
