MODE ?= debug
EFI := target/x86_64-unknown-uefi/$(MODE)/uefi.efi
OVMF := OVMF.fd
ESP := target/esp
BUILD_ARGS := -Z build-std=core,alloc
QEMU_ARGS := -net none -nographic -cpu qemu64,fsgsbase
OBJDUMP := rust-objdump

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

build:
	cargo build $(BUILD_ARGS)

run: build
	mkdir -p $(ESP)/EFI/Boot
	cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi
	qemu-system-x86_64 \
		-bios ${OVMF} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

clippy:
	cargo clippy $(BUILD_ARGS)

fix:
	cargo fix $(BUILD_ARGS) --allow-dirty

header:
	$(OBJDUMP) -h $(EFI) | less

asm:
	$(OBJDUMP) -d $(EFI) | less
