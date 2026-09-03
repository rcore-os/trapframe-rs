use super::*;
use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

/// Initialize interrupt handling for the current HART.
///
/// # Safety
///
/// This function will:
/// - Set `vbar_el1` to internal exception vector.
///
/// You **MUST NOT** modify these registers later.
pub unsafe fn init() {
    // Set the exception vector address
    unsafe {
        asm!("msr VBAR_EL1, {}", in(reg) __vectors as *const () as usize);
    }
}

/// Register frame saved when an exception enters the kernel.
///
/// # Trap handler
///
/// You need to define a handler function like this:
///
/// ```no_run
/// use trapframe::TrapFrame;
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
///     println!("TRAP! tf: {:#x?}", tf);
/// }
/// ```
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    /// Encoded exception source and kind.
    pub trap_num: usize,
    /// Reserved for the assembly frame layout.
    pub __reserved: usize,
    /// Exception Link Register (`ELR_EL1`).
    pub elr: usize,
    /// Saved Process Status Register (`SPSR_EL1`).
    pub spsr: usize,
    /// Kernel stack pointer (`SP_EL1`).
    pub sp: usize,
    /// Kernel thread pointer (`TPIDR_EL1`).
    pub tpidr: usize,
    /// General-purpose registers; kept last for the assembly layout.
    pub general: GeneralRegs,
}

impl UserContext {
    /// Go to user space with the context, and come back when a trap occurs.
    ///
    /// On return, the context will be reset to the status before the trap.
    /// Trap reason and error code will be returned.
    ///
    /// # Example
    /// ```no_run
    /// use trapframe::{UserContext, GeneralRegs};
    ///
    /// // init user space context
    /// let mut context = UserContext {
    ///     general: GeneralRegs {
    ///         ..Default::default()
    ///     },
    ///     sp: 0x10000,
    ///     elr: 0x1000,
    ///     ..Default::default()
    /// };
    /// // go to user
    /// context.run();
    /// // back from user
    /// println!("back from user: {:#x?}", context);
    /// ```
    pub fn run(&mut self) {
        let mut context = UserContextWithExtensions {
            trap_num: self.trap_num,
            __reserved: self.__reserved,
            elr: self.elr,
            spsr: self.spsr,
            sp: self.sp,
            tpidr: self.tpidr,
            general: self.general,
            ..Default::default()
        };
        context.run();
        self.trap_num = context.trap_num;
        self.__reserved = context.__reserved;
        self.elr = context.elr;
        self.spsr = context.spsr;
        self.sp = context.sp;
        self.tpidr = context.tpidr;
        self.general = context.general;
    }
}

impl UserContextWithExtensions {
    /// Goes to user space while preserving floating-point and SIMD state.
    pub fn run(&mut self) {
        unsafe { run_user(self) }
    }
}

#[allow(improper_ctypes)]
unsafe extern "C" {
    fn __vectors();
    fn run_user(regs: &mut UserContextWithExtensions);
}
