use core::fmt::{Debug, Error, Formatter};
use riscv::register::{scause::Scause, sscratch, sstatus::Sstatus, stvec};

#[cfg(target_arch = "riscv32")]
global_asm!(
    r"
    .equ XLENB,     4
    .equ XLENb,     32
    .macro LOAD_TF a1, a2
        lw \a1, \a2*XLENB(sp)
    .endm
    .macro STORE_TF a1, a2
        sw \a1, \a2*XLENB(sp)
    .endm
    .macro STORE a1, a2
        sw \a1, \a2
    .endm
"
);
#[cfg(target_arch = "riscv64")]
global_asm!(
    r"
    .equ XLENB,     8
    .equ XLENb,     64
    .macro LOAD_TF a1, a2
        ld \a1, \a2*XLENB(sp)
    .endm
    .macro STORE_TF a1, a2
        sd \a1, \a2*XLENB(sp)
    .endm
    .macro STORE a1, a2
        sd \a1, \a2
    .endm
"
);

global_asm!(include_str!("trap.S"));

/// Initialize interrupt handling for the current HART.
pub fn init(hartid: usize) {
    unsafe {
        // Set sscratch register to 0, indicating to exception vector that we are
        // presently executing in the kernel
        sscratch::write(0);
        // Store hartid to gp register
        asm!("mv gp, $0" :: "r"(hartid));
        // Set the exception vector address
        stvec::write(trap_entry as usize, stvec::TrapMode::Direct);
    }
}

/// # Example
/// ```
/// #[no_mangle]
/// pub extern "C" fn trap_hander(scause: Scause, stval: usize, tf: &mut TrapFrame) {
///     panic!("TRAP! scause: {:?}, stval: {:#x}, tf: {:#x?}", scause, stval, tf);
/// }
/// ```
#[no_mangle]
#[linkage = "weak"]
pub extern "C" fn trap_hander() {
    panic!("no trap handler");
}

pub type TrapHandler = extern "C" fn(tf: &mut TrapFrame, scause: Scause, stval: usize);

/// Saved registers on a trap.
#[derive(Clone)]
#[repr(C)]
pub struct TrapFrame {
    /// General registers
    pub x: [usize; 32],
    /// Supervisor Status
    pub sstatus: Sstatus,
    /// Supervisor Exception Program Counter
    pub sepc: usize,
    /// Kernel stack top.
    ///
    /// The next trap from user to kernel will switch stack pointer to here.
    /// It will be ignored if the trap is from kernel.
    pub kernel_sp: usize,
}

impl TrapFrame {
    /// Constructs `TrapFrame` for a new kernel thread.
    ///
    /// The new thread starts at function `entry` with an usize argument `arg`.
    /// The stack pointer will be set to `sp`.
    /// The interrupt will be enabled if `interrupt` is true.
    pub fn new_kernel_thread(
        entry: extern "C" fn(arg: usize) -> !,
        arg: usize,
        sp: usize,
        interrupt: bool,
    ) -> Self {
        let mut tf: Self = unsafe { core::mem::zeroed() };
        tf.x[10] = arg; // a0
        tf.x[2] = sp;
        tf.sepc = entry as usize;
        // SIE=0, SPIE=int?, SPP=S
        let spie: usize = if interrupt { 1 << 5 } else { 0 };
        tf.sstatus = unsafe { core::mem::transmute(1 << 8 | spie) };
        tf
    }

    /// Constructs `TrapFrame` for a new user thread.
    ///
    /// The new thread starts at `entry_addr`.
    /// The stack pointer will be set to `sp`.
    /// The interrupt will be enabled.
    /// When a trap happened in user mode, the stack poiner will be switched
    /// to `kernel_sp`.
    pub fn new_user_thread(entry: usize, sp: usize, kernel_sp: usize) -> Self {
        let mut tf: Self = unsafe { core::mem::zeroed() };
        tf.x[2] = sp;
        tf.sepc = entry;
        // SIE=0, SPIE=1, SPP=U
        tf.sstatus = unsafe { core::mem::transmute(1usize << 5) };
        tf.kernel_sp = kernel_sp;
        tf
    }

    /// Go back to the context before the trap.
    pub unsafe fn go(self) -> ! {
        asm!("mv sp, $0" :: "r"(&self as *const Self as usize));
        trap_return();
    }
}

impl Debug for TrapFrame {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        struct Regs<'a>(&'a [usize; 32]);
        impl<'a> Debug for Regs<'a> {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                const REG_NAME: [&str; 32] = [
                    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2",
                    "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9",
                    "s10", "s11", "t3", "t4", "t5", "t6",
                ];
                f.debug_map().entries(REG_NAME.iter().zip(self.0)).finish()
            }
        }
        f.debug_struct("TrapFrame")
            .field("regs", &Regs(&self.x))
            .field("sstatus", &self.sstatus)
            .field("sepc", &self.sepc)
            .finish()
    }
}

extern "C" {
    fn trap_entry();
    fn trap_return() -> !;
}
