arch ?= riscv64
target := $(arch)imac-unknown-none-elf
mode := debug
kernel := target/$(target)/$(mode)/riscv

sysroot := $(shell rustc --print sysroot)
objdump := $(shell find $(sysroot) -name llvm-objdump) --arch-name=$(arch)
objcopy := $(shell find $(sysroot) -name llvm-objcopy)

BUILD_ARGS += --target $(target)
ifeq ($(mode), release)
	BUILD_ARGS += --release
endif

ifeq ($(arch), riscv32)
	START_ADDR := 0x80400000
else ifeq ($(arch), riscv64)
	START_ADDR := 0x80200000
endif

.PHONY: kernel build clean qemu run env

build: kernel

env:
	rustup component add llvm-tools-preview rustfmt --toolchain nightly
	rustup target add $(target) --toolchain nightly

kernel:
	cargo +nightly build $(BUILD_ARGS)

asm:
	$(objdump) -d $(kernel) | less

sym:
	$(objdump) -t $(kernel) | less

header:
	$(objdump) -x $(kernel) | less

clean:
	cargo clean

qemu: $(bin)
	qemu-system-$(arch) \
		-machine virt \
		-nographic \
		-bios default \
		-kernel $(kernel)

run: build qemu
