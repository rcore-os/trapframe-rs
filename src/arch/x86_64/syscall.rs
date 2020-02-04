use super::*;
use core::fmt;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags, Msr};

global_asm!(include_str!("syscall.S"));

pub fn init() {
    let cpuid = raw_cpuid::CpuId::new();
    unsafe {
        // enable `syscall` instruction
        assert!(cpuid
            .get_extended_function_info()
            .unwrap()
            .has_syscall_sysret());
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });

        // enable FPU
        assert!(cpuid.get_feature_info().unwrap().has_fpu());
        Cr0::update(|cr0| {
            cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
            cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });

        // enable `fxsave` `fxrstor` instruction
        assert!(cpuid.get_feature_info().unwrap().has_fxsave_fxstor());
        Cr4::update(|cr4| {
            cr4.insert(Cr4Flags::OSFXSR);
            cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        });

        // enable `rdfsbase` series instructions.
        assert!(cpuid.get_extended_feature_info().unwrap().has_fsgsbase());
        Cr4::update(|cr4| cr4.insert(Cr4Flags::FSGSBASE));

        //        let mut star = Msr::new(0xC000_0081); // legacy mode SYSCALL target
        let mut lstar = Msr::new(0xC000_0082); // long mode SYSCALL target
        let mut sfmask = Msr::new(0xC000_0084); // EFLAGS mask for syscall

        // flags to clear on syscall
        // copy from Linux 5.0
        // TF|DF|IF|IOPL|AC|NT
        const RFLAGS_MASK: u64 = 0x47700;

        //        star.write(core::mem::transmute(STAR));
        lstar.write(syscall_entry as usize as u64);
        sfmask.write(RFLAGS_MASK);
    }
}

extern "sysv64" {
    fn syscall_entry();
    fn syscall_return(regs: &mut UserContext);
}

/// User space context
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct UserContext {
    pub vector: VectorRegs,
    pub general: GeneralRegs,
    pub trap_num: usize,
    pub error_code: usize,
}

impl UserContext {
    /// Go to user space with the context, and come back when a trap occurs.
    ///
    /// On return, the context will be reset to the status before the trap.
    /// Trap reason and error code will be placed at `trap_num` and `error_code`.
    ///
    /// If the trap was triggered by `syscall` instruction, the `trap_num` will be set to `0x100`.
    ///
    /// If `trap_num` is `0x100`, it will go user by `sysret` (`rcx` and `r11` are dropped),
    /// otherwise it will use `iret`.
    ///
    /// # Example
    /// ```no_run
    /// use trapframe::{UserContext, GeneralRegs};
    ///
    /// // init user space context
    /// let mut context = UserContext {
    ///     general: GeneralRegs {
    ///         rip: 0x1000,
    ///         rsp: 0x10000,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// // go to user
    /// context.run();
    /// // back from user
    /// println!("back from user: {:#x?}", context);
    /// ```
    pub fn run(&mut self) {
        unsafe {
            syscall_return(self);
        }
    }
}

/// General registers
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct GeneralRegs {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rbp: usize,
    pub rsp: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rip: usize,
    pub rflags: usize,
    pub fsbase: usize,
    pub gsbase: usize,
}

/// Vector registers
///
/// Currently the structure is same as the layout of the [`fxsave` map].
///
/// [`fxsave` map]: https://www.felixcloutier.com/x86/FXSAVE.html#tbl-3-47
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, align(16))]
pub struct VectorRegs {
    pub fcw: u16,
    pub fsw: u16,
    pub ftw: u8,
    pub _pad0: u8,
    pub fop: u16,
    pub fip: u32,
    pub fcs: u16,
    pub _pad1: u16,

    pub fdp: u32,
    pub fds: u16,
    pub _pad2: u16,
    pub mxcsr: u32,
    pub mxcsr_mask: u32,

    pub mm: [U128; 8],
    pub xmm: [U128; 16],
    pub reserved: [U128; 3],
    pub available: [U128; 3],
    //    /// When only 16 registers are supported (pre-AVX-512), zmm[16-31] will be 0.
    //    /// YMM registers (256 bits) are v[0-4], XMM registers (128 bits) are v[0-2].
    //    pub zmm: [[u64; 8]; 32],
    //
    //    /// AVX-512 opmask registers. Will be 0 unless AVX-512 is supported.
    //    pub opmask: [u64; 8],
    //
    //    /// SIMD control and status register.
    //    pub mxcsr: u32,
}

// workaround: libcore has bug on Debug print u128 ??
#[derive(Default, Clone, Copy)]
#[repr(C, align(16))]
pub struct U128([u64; 2]);

impl fmt::Debug for U128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#016x}{:016x}", self.0[1], self.0[0])
    }
}
