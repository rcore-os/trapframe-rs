#![no_std]
#![feature(asm, global_asm, linkage)]

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[path = "arch/riscv/mod.rs"]
mod arch;

pub use arch::*;

pub fn init() {}
