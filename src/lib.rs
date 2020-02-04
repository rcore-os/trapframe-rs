#![no_std]
#![feature(asm, global_asm, linkage)]
#![feature(vec_leak)]
#![deny(warnings)]

extern crate alloc;

#[macro_use]
extern crate log;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[path = "arch/riscv/mod.rs"]
mod arch;

#[cfg(target_arch = "mips")]
#[path = "arch/mipsel/mod.rs"]
pub mod arch;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
pub mod arch;

pub use arch::*;
