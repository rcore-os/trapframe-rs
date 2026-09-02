target := mipsel-unknown-none
mode := debug
kernel := target/$(target)/$(mode)/mipsel

.PHONY: build clean env

build:
	cargo build

env:
	rustup toolchain install nightly-2026-09-01 --component rust-src

clean:
	cargo clean
