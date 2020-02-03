use core::default::Default;
use core::fmt;

mod fast_syscall;
mod gdt;
mod trap;

pub use fast_syscall::{run_user, GeneralRegs};

pub fn init() {
    gdt::init();
    fast_syscall::init();
}
