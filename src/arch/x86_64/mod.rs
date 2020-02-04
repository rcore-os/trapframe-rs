use core::default::Default;

mod gdt;
mod idt;
mod syscall;
mod trap;

pub use syscall::{GeneralRegs, UserContext, VectorRegs};
pub use trap::TrapFrame;
use x86_64::instructions::interrupts;

pub fn init() {
    interrupts::disable();
    gdt::init();
    idt::init();
    syscall::init();
    info!("initialize end");
}
