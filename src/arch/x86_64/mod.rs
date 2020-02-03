use core::default::Default;
use core::fmt;

mod fast_syscall;
mod gdt;
mod idt;
mod trap;

pub use fast_syscall::{run_user, GeneralRegs};
pub use trap::TrapFrame;
use x86_64::instructions::interrupts;

pub fn init() {
    interrupts::disable();
    gdt::init();
    idt::init();
    fast_syscall::init();
    info!("initialize end");
}
