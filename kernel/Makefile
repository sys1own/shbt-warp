ifeq ($(origin CC),default)
CC := gcc
endif
CC ?= gcc
CFLAGS ?= -std=c11 -O2 -ffreestanding -fno-builtin -nostdlib \
	-fpic -fno-asynchronous-unwind-tables -Wall -Wextra -Iinclude
# AVX-512 when the toolchain/host supports it; the kernel carries a scalar
# fallback for targets without it.
AVX512 := $(shell printf 'int main(void){return 0;}' | $(CC) -mavx512f -x c - -o /dev/null 2>/dev/null && echo yes)
AVX512_MFLAG := $(if $(filter yes,$(AVX512)),-mavx512f,)

comma := ,
IS_MINGW := $(findstring mingw,$(shell $(CC) -dumpmachine))
SO_LDFLAGS := $(if $(IS_MINGW),-Wl$(comma)--image-base=0x80000000 -Wl$(comma)--entry=0x80000000,)

ifeq ($(AVX512),yes)
CFLAGS += -mavx512f
endif

BUILD := build
OBJ := $(BUILD)/shbt_warp_kernel.o
LIB := $(BUILD)/libshbt_warp_reference.so
SO  := $(BUILD)/shbt_warp_reference.so

all: $(SO) $(LIB)

$(BUILD):
	mkdir -p $(BUILD)

$(OBJ): src/shbt_warp_kernel.c include/shbt_warp_hardware.h | $(BUILD)
	$(CC) $(CFLAGS) -c $< -o $@

# Freestanding image linked against the bare-metal linker script.
$(SO): $(OBJ) linker.ld
	$(CC) -shared -nostdlib -T linker.ld $(SO_LDFLAGS) -o $@ $(OBJ)

# Hosted reference library for the userland C-ABI test: same kernel TU,
# hosted arena definitions, normal dynamic linking.
$(LIB): src/shbt_warp_kernel.c src/arena_stubs.c include/shbt_warp_hardware.h | $(BUILD)
	$(CC) -std=c11 -O2 -fPIC -Wall -Wextra -Iinclude $(AVX512_MFLAG) -shared \
		src/shbt_warp_kernel.c src/arena_stubs.c -o $@

$(BUILD)/reference_test: ../tests/reference_test.c include/shbt_warp_hardware.h $(LIB) | $(BUILD)
	$(CC) -std=c11 -O2 -Wall -Wextra -pedantic -Iinclude $(AVX512_MFLAG) \
		../tests/reference_test.c $(LIB) -lm -o $@

test-c: $(BUILD)/reference_test
ifeq ($(OS),Windows_NT)
	$(BUILD)/reference_test
else
	PATH="$(BUILD):$(PATH)" LD_LIBRARY_PATH=$(BUILD) ./$(BUILD)/reference_test
endif

clean:
	rm -rf $(BUILD)

.PHONY: all clean test-c
