use core::default::Default;

mod gdt;
mod idt;
mod syscall;
mod trap;

pub use syscall::{GeneralRegs, UserContext, VectorRegs};
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
/// - Enable [`rdfsbase`] series instructions.
///     - set `CR4::FSGSBASE`
/// - Enable [FPU].
///     - clear `CR0::EMULATE_COPROCESSOR`
///     - set `CR0::MONITOR_COPROCESSOR`
/// - Enable [`fxsave`] instruction.
///     - set `CR4::OSFXSR`
///     - set `CR4::OSXMMEXCPT_ENABLE`
///
/// [GDT]: https://wiki.osdev.org/GDT
/// [IDT]: https://wiki.osdev.org/IDT
/// [TSS]: https://wiki.osdev.org/Task_State_Segment
/// [FPU]: https://wiki.osdev.org/FPU
/// [`syscall`]: https://www.felixcloutier.com/x86/syscall
/// [`rdfsbase`]: https://www.felixcloutier.com/x86/rdfsbase:rdgsbase
/// [`fxsave`]: https://www.felixcloutier.com/x86/fxsave
///
pub unsafe fn init() {
    interrupts::disable();
    gdt::init();
    idt::init();
    syscall::init();
    info!("initialize end");
}
