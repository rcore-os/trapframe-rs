use super::*;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("vector.S"));

#[derive(Clone)]
#[repr(C)]
pub struct FpState([u8; 16 + 512]);

impl fmt::Debug for FpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fpstate")
    }
}

impl Default for FpState {
    fn default() -> Self {
        FpState([0u8; 16 + 512])
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct TrapFrame {
    // Pushed by 'trap.S'

    // fpstate needs to be 16-byte aligned
    // so we reserve some space here and save the offset
    // the read fpstate begin from fpstate[offset]
    pub fpstate_offset: usize,
    pub fpstate: FpState,

    pub fsbase: usize,
    pub gsbase: usize,

    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    // Pushed by 'vector.S'
    pub trap_num: usize,
    pub error_code: usize,

    // Pushed by CPU
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,

    // Pushed by CPU when Ring3->0
    pub rsp: usize,
    pub ss: usize,
}

/// 用于在内核栈中构造新线程的中断帧
impl TrapFrame {
    fn new_kernel_thread(entry: extern "C" fn(usize) -> !, arg: usize, rsp: usize) -> Self {
        let mut tf = TrapFrame::default();
        tf.rdi = arg;
        tf.cs = 0; // FIXME: gdt::KCODE_SELECTOR.0 as usize;
        tf.rip = entry as usize;
        tf.ss = 0; // FIXME: gdt::KDATA_SELECTOR.0 as usize;
        tf.rsp = rsp;
        tf.rflags = 0x282;
        tf.fpstate_offset = 16; // skip restoring for first time
        tf
    }
    pub fn new_user_thread(entry_addr: usize, rsp: usize) -> Self {
        let mut tf = TrapFrame::default();
        tf.cs = 0; // FIXME: gdt::UCODE_SELECTOR.0 as usize;
        tf.rip = entry_addr;
        tf.ss = 0; // FIXME: gdt::UDATA_SELECTOR.0 as usize;
        tf.rsp = rsp;
        tf.rflags = 0x282;
        tf.fpstate_offset = 16; // skip restoring for first time
        tf
    }
}
