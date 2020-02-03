use super::*;
use x86_64::registers::model_specific::*;

global_asm!(include_str!("syscall.S"));

pub fn init() {
    unsafe {
        // enable `syscall` instruction
        Efer::update(|flags| {
            *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
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
    pub fn run_user(regs: &mut GeneralRegs);
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
