ifeq ($(ARCH), x86_64)
ifeq ($(shell uname), Darwin)
TARGET := x86_64-apple-darwin
else
TARGET := x86_64-unknown-linux-gnu
endif
else ifeq ($(ARCH), aarch64)
TARGET := aarch64-unknown-none
else ifeq ($(ARCH), mipsel)
TARGET := mipsel-unknown-linux-gnu
else ifeq ($(ARCH), riscv32)
TARGET := riscv32imac-unknown-none-elf
else ifeq ($(ARCH), riscv64)
TARGET := riscv64imac-unknown-none-elf
endif

all: build

env:
	rustup target add $(TARGET)

build:
	cargo build --target $(TARGET)

doc:
	cargo doc --target $(TARGET)
