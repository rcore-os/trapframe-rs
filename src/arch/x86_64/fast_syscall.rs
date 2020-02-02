use super::*;
use x86_64::registers::model_specific::*;

pub fn init() {
    unsafe {
        Efer::update(|flags| {
            *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        });

        let mut star = Msr::new(0xC0000081); // legacy mode SYSCALL target
        let mut lstar = Msr::new(0xC0000082); // long mode SYSCALL target
        let mut sfmask = Msr::new(0xC0000084); // EFLAGS mask for syscall

        // flags to clear on syscall
        // copy from Linux 5.0
        // TF|DF|IF|IOPL|AC|NT
        let rflags_mask = 0x47700;

        star.write(core::mem::transmute(STAR));
        lstar.write(syscall_entry as u64);
        sfmask.write(rflags_mask);
    }
}

extern "C" {
    fn syscall_entry();
    pub fn run_user(regs: &mut GeneralRegs);
}

#[repr(packed)]
struct StarMsr {
    eip: u32,
    kernel_cs: u16,
    user_cs: u16,
}

const STAR: StarMsr = StarMsr {
    eip: 0, // ignored in 64 bit mode
    kernel_cs: KCODE_SELECTOR,
    user_cs: UCODE_SELECTOR,
};

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
