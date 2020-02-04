use super::*;
use core::fmt;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags, Msr};

global_asm!(include_str!("syscall.S"));

pub fn init() {
    unsafe {
        // enable `syscall` instruction
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });

        // enable `fxsave` `fxrstor` instruction
        Cr0::update(|cr0| {
            cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
            cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
        Cr4::update(|cr4| {
            cr4.insert(Cr4Flags::OSFXSR);
            cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        });

        //        let mut star = Msr::new(0xC0000081); // legacy mode SYSCALL target
        let mut lstar = Msr::new(0xC0000082); // long mode SYSCALL target
        let mut sfmask = Msr::new(0xC0000084); // EFLAGS mask for syscall

        // flags to clear on syscall
        // copy from Linux 5.0
        // TF|DF|IF|IOPL|AC|NT
        const RFLAGS_MASK: u64 = 0x47700;

        //        star.write(core::mem::transmute(STAR));
        lstar.write(syscall_entry as u64);
        sfmask.write(RFLAGS_MASK);
    }
}

extern "sysv64" {
    fn syscall_entry();
    fn run_user(regs: &mut UserContext);
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct UserContext {
    pub vector: VectorRegs,
    pub general: GeneralRegs,
}

impl UserContext {
    pub fn run(&mut self) {
        unsafe {
            run_user(self);
        }
    }
}

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
