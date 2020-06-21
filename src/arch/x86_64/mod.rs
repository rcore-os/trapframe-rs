use core::default::Default;

mod gdt;
mod idt;
mod syscall;
mod trap;

pub use syscall::{GeneralRegs, UserContext};
pub use trap::TrapFrame;
use x86_64::instructions::interrupts;

/// Initialize interrupt handling on x86_64.
///
/// # Safety
///
/// This function will:
///
/// - Disable interrupt.
/// - Switch to a new [GDT], extend 7 more entries from the current one.
/// - Switch to a new [TSS], set `GSBASE` to its base address.
/// - Switch to a new [IDT], override the current one.
/// - Enable [`syscall`] instruction.
///     - set `EFER::SYSTEM_CALL_EXTENSIONS`
///
/// [GDT]: https://wiki.osdev.org/GDT
/// [IDT]: https://wiki.osdev.org/IDT
/// [TSS]: https://wiki.osdev.org/Task_State_Segment
/// [`syscall`]: https://www.felixcloutier.com/x86/syscall
///
pub unsafe fn init() {
    interrupts::disable();
    gdt::init();
    idt::init();
    syscall::init();
}
