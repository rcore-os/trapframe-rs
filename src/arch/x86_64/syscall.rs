use super::*;
use core::fmt;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

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
            cr0.remove(Cr0Flags::TASK_SWITCHED);
        });

        // enable `fxsave` `fxrstor` instruction
        assert!(cpuid.get_feature_info().unwrap().has_fxsave_fxstor());
        Cr4::update(|cr4| {
            cr4.insert(Cr4Flags::OSFXSR);
            cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        });

        // flags to clear on syscall
        // copy from Linux 5.0
        // TF|DF|IF|IOPL|AC|NT
        const RFLAGS_MASK: u64 = 0x47700;

        LStar::write(VirtAddr::new(syscall_entry as usize as u64));
        SFMask::write(RFlags::from_bits(RFLAGS_MASK).unwrap());
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
            if self.vector.lazy_restore() {
                Cr0::update(|f| f.insert(Cr0Flags::TASK_SWITCHED));
            } else {
                core::arch::x86_64::_fxrstor64(&self.vector as *const _ as *const u8);
            }

            syscall_return(self);

            if !Cr0::read().contains(Cr0Flags::TASK_SWITCHED) {
                core::arch::x86_64::_fxsave64(&mut self.vector as *mut _ as *mut u8);
            }
            asm!("clts");
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
#[derive(Debug, Clone, Copy)]
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

// https://xem.github.io/minix86/manual/intel-x86-and-64-manual-vol1/o_7281d5ea06a5b67a-274.html
impl Default for VectorRegs {
    fn default() -> Self {
        VectorRegs {
            mxcsr: 0x1f80,
            ..unsafe { core::mem::zeroed() }
        }
    }
}

impl VectorRegs {
    /// Set lazy restore vector registers.
    ///
    /// If set to true, vector registers will not be restored on switching to user space.
    /// Instead, `CR0.TS` bit will be set, and all following operations will cause a #NM
    /// exception (vector 7), so that we can lazily restore the FPU state.
    pub fn set_lazy_restore(&mut self, value: bool) {
        // now use the last byte of available area
        unsafe {
            (self as *mut _ as *mut bool).add(511).write(value);
        }
    }

    /// Whether lazy restore vector registers.
    pub fn lazy_restore(&mut self) -> bool {
        unsafe { (self as *mut _ as *mut bool).add(511).read() }
    }
}

// workaround: libcore has bug on Debug print u128 ??
#[derive(Default, Clone, Copy)]
#[repr(C, align(16))]
pub struct U128(pub [u64; 2]);

impl fmt::Debug for U128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#016x}{:016x}", self.0[1], self.0[0])
    }
}
