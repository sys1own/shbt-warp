/*
 * arena_stubs.c — hosted Stinespring arena for the userland reference
 * library (libshbt_warp_reference.so).
 *
 * The freestanding kernel resolves these symbols from linker.ld's
 * .stinespring_frame section; the hosted build supplies the same
 * 2,112-byte arena (640 B active + 1,472 B dark) as ordinary C arrays so
 * the reference test can link and exercise the kernel C-ABI in userland.
 */
#include <stdint.h>

uint8_t __stinespring_start[2112] __attribute__((aligned(64)));
uint8_t __active_end[1];
uint8_t __stinespring_end[1];
