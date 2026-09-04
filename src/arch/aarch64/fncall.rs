//! Switch context by function call within the same privilege level.
//!
use super::{UserContext, UserContextWithExtensions};
use core::arch::global_asm;
use core::sync::atomic::{AtomicUsize, Ordering};

const FNCALL_CONTEXT_SLOTS: usize = 256;
const RESERVED_THREAD_KEY: usize = usize::MAX;

#[repr(C)]
struct FncallContextSlot {
    thread_key: AtomicUsize,
    context: AtomicUsize,
}

#[unsafe(no_mangle)]
static FNCALL_CONTEXTS: [FncallContextSlot; FNCALL_CONTEXT_SLOTS] = [const {
    FncallContextSlot {
        thread_key: AtomicUsize::new(0),
        context: AtomicUsize::new(0),
    }
}; FNCALL_CONTEXT_SLOTS];

fn register_fncall_context(thread_key: usize, context: *mut UserContext) {
    assert_ne!(thread_key, 0);
    for slot in &FNCALL_CONTEXTS {
        if slot
            .thread_key
            .compare_exchange(0, RESERVED_THREAD_KEY, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            slot.context.store(context as usize, Ordering::Relaxed);
            slot.thread_key.store(thread_key, Ordering::Release);
            return;
        }
    }
    panic!("too many concurrent AArch64 fncall contexts");
}

#[cfg(target_os = "linux")]
fn host_thread_key() -> usize {
    let thread_id: usize;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") 178usize,
            lateout("x0") thread_id,
            options(nostack)
        );
    }
    thread_id
}

#[cfg(target_os = "macos")]
fn host_thread_key() -> usize {
    let thread_pointer: usize;
    unsafe {
        core::arch::asm!(
            "mrs {thread_pointer}, tpidrro_el0",
            thread_pointer = out(reg) thread_pointer,
            options(nomem, nostack, preserves_flags)
        );
    }
    thread_pointer
}

fn prepare_fncall_context(context: &mut UserContext) {
    if context.tpidr == 0 {
        let kernel_thread_pointer: usize;
        #[cfg(target_os = "linux")]
        unsafe {
            core::arch::asm!(
                "mrs {thread_pointer}, tpidr_el0",
                thread_pointer = out(reg) kernel_thread_pointer,
                options(nomem, nostack, preserves_flags)
            );
            context.tpidr = kernel_thread_pointer + 72;
        }
        #[cfg(target_os = "macos")]
        unsafe {
            core::arch::asm!(
                "mrs {thread_pointer}, tpidrro_el0",
                thread_pointer = out(reg) kernel_thread_pointer,
                options(nomem, nostack, preserves_flags)
            );
            context.tpidr = kernel_thread_pointer + 240;
        }
    }
    register_fncall_context(host_thread_key(), context);
}

#[cfg(target_os = "linux")]
global_asm!(
    r#"
.macro INIT_USER_TP dst, kernel_tp
    add     \dst, \kernel_tp, #72
.endm

.macro LOAD_CONTEXT_TABLE dst
    adrp    \dst, :got:FNCALL_CONTEXTS
    ldr     \dst, [\dst, :got_lo12:FNCALL_CONTEXTS]
.endm


.macro LOAD_HOST_THREAD_KEY dst, scratch
    mov     \scratch, #178
    svc     #0
.endm

.global syscall_fn_entry
.global syscall_fn_return
.global syscall_fn_return_extended
"#
);

#[cfg(target_os = "macos")]
global_asm!(
    r#"
.macro INIT_USER_TP dst, kernel_tp
    // Darwin's TSD base is in TPIDRRO_EL0. Use TSD slot 30 as the
    // backing storage for the initial synthetic user thread pointer.
    mrs     \dst, tpidrro_el0
    add     \dst, \dst, #240
.endm


.macro LOAD_CONTEXT_TABLE dst
    adrp    \dst, _FNCALL_CONTEXTS@GOTPAGE
    ldr     \dst, [\dst, _FNCALL_CONTEXTS@GOTPAGEOFF]
.endm


.macro LOAD_HOST_THREAD_KEY dst, scratch
    mrs     \dst, tpidrro_el0
.endm

.global _syscall_fn_entry
.global _syscall_fn_return
.global _syscall_fn_return_extended
.set _syscall_fn_entry, syscall_fn_entry
.set _syscall_fn_return, syscall_fn_return
.set _syscall_fn_return_extended, syscall_fn_return_extended
"#
);

global_asm!(include_str!("fncall.S"));

unsafe extern "C" {
    /// The syscall entry of function call.
    ///
    /// # Usage
    ///
    /// Replace `svc` instruction by a `bl` instruction.
    ///
    /// ```asm
    /// svc #0
    /// bl syscall_fn_entry
    /// ```
    pub fn syscall_fn_entry();

    fn syscall_fn_return(regs: &mut UserContext);
    fn syscall_fn_return_extended(regs: &mut UserContextWithExtensions);
}

impl UserContext {
    /// Go to user context by function return, within the same privilege level.
    ///
    /// User program should call `syscall_fn_entry()` to return back.
    pub fn run_fncall(&mut self) {
        prepare_fncall_context(self);
        unsafe { syscall_fn_return(self) }
    }
}

impl UserContextWithExtensions {
    /// Goes to user context while preserving floating-point and SIMD state.
    pub fn run_fncall(&mut self) {
        prepare_fncall_context(self);
        unsafe { syscall_fn_return_extended(self) }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use core::arch::{asm, global_asm};

    #[cfg(target_os = "linux")]
    global_asm!(
        r#"
.macro LOAD_ADDRESS reg, symbol
    adrp    \reg, \symbol
    add     \reg, \reg, :lo12:\symbol
.endm

.global test_preserve_host_state
.global observe_guest_x18
"#
    );

    #[cfg(target_os = "macos")]
    global_asm!(
        r#"
.macro LOAD_ADDRESS reg, symbol
    adrp    \reg, \symbol@GOTPAGE
    ldr     \reg, [\reg, \symbol@GOTPAGEOFF]
.endm

.set _dump_registers, dump_registers
.set _elr_location, elr_location
.set _test_preserve_host_state, test_preserve_host_state
.set _observe_guest_x18, observe_guest_x18
.set _observe_guest_x18_return, observe_guest_x18_return
.set RESTORED_Q0, _RESTORED_Q0
.set UPDATED_Q0, _UPDATED_Q0
"#
    );

    #[unsafe(no_mangle)]
    static mut RESTORED_Q0: u128 = 0;
    #[unsafe(no_mangle)]
    static UPDATED_Q0: u128 = 0xa5a5_a5a5_a5a5_a5a5_5a5a_5a5a_5a5a_5a5a;

    // Mock user program to dump registers at stack.
    global_asm!(
        r#"
dump_registers:
    str     x9, [sp, #-16]!
    LOAD_ADDRESS x9, RESTORED_Q0
    str     q0, [x9]
    LOAD_ADDRESS x9, UPDATED_Q0
    ldr     q0, [x9]
    mrs     x9, tpidr_el0
    add     x9, x9, #8
    msr     tpidr_el0, x9
    str     xzr, [x9, #48]
    ldr     x9, [sp], #16
    stp     x30, x0, [sp, #-16]!
    str     x29, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!
    stp     x17, x18, [sp, #-16]!
    stp     x15, x16, [sp, #-16]!
    stp     x13, x14, [sp, #-16]!
    stp     x11, x12, [sp, #-16]!
    stp     x9, x10, [sp, #-16]!
    stp     x7, x8, [sp, #-16]!
    stp     x5, x6, [sp, #-16]!
    stp     x3, x4, [sp, #-16]!
    stp     x1, x2, [sp, #-16]!

    add     x0, x0, #100
    add     x1, x1, #100
    add     x2, x2, #100
    add     x3, x3, #100
    add     x4, x4, #100
    add     x5, x5, #100
    add     x6, x6, #100
    add     x7, x7, #100
    add     x8, x8, #100
    add     x9, x9, #100
    add     x10, x10, #100
    add     x11, x11, #100
    add     x12, x12, #100
    add     x13, x13, #100
    add     x14, x14, #100
    add     x15, x15, #100
    add     x16, x16, #100
    add     x17, x17, #100
    add     x18, x18, #100
    add     x19, x19, #100
    add     x20, x20, #100
    add     x21, x21, #100
    add     x22, x22, #100
    add     x23, x23, #100
    add     x24, x24, #100
    add     x25, x25, #100
    add     x26, x26, #100
    add     x27, x27, #100
    add     x28, x28, #100
    add     x29, x29, #100
    add     x30, x30, #100

    bl syscall_fn_entry

.global elr_location
elr_location:

// Call the extended entry point while holding sentinels in the AAPCS64
// callee-saved d8 register and the platform register x18. Return the observed
// values through x1 while preserving the real caller's registers.
test_preserve_host_state:
    stp     x19, x20, [sp, #-64]!
    stp     x21, x30, [sp, #16]
    str     d8, [sp, #32]
    str     x18, [sp, #40]
    mov     x19, x1
    mov     x20, x2
    fmov    d8, x20
    mov     x18, x3
    bl      syscall_fn_return_extended
    fmov    x9, d8
    str     x9, [x19]
    str     x18, [x19, #8]
    ldr     d8, [sp, #32]
    ldr     x18, [sp, #40]
    ldp     x21, x30, [sp, #16]
    ldp     x19, x20, [sp], #64
    ret

// Observe the guest x18 value restored by the direct Rust entry path.
observe_guest_x18:
    mov     x0, x18
    bl      syscall_fn_entry
.global observe_guest_x18_return
observe_guest_x18_return:
    brk     #0
"#
    );

    #[test]
    fn run_fncall() {
        unsafe extern "C" {
            fn dump_registers();
            fn elr_location();
            fn test_preserve_host_state(
                context: &mut UserContextWithExtensions,
                observed: &mut [u64; 2],
                d8_sentinel: u64,
                x18_sentinel: u64,
            );
        }
        #[repr(align(16))]
        struct AlignedStack([u8; 0x1000]);

        let mut stack = AlignedStack([0; 0x1000]);
        let mut guest_tls = [0usize; 32];
        let initial_guest_tp = unsafe { guest_tls.as_mut_ptr().add(8) } as usize;
        let general = GeneralRegs {
            x0: 0,
            x1: 1,
            x2: 2,
            x3: 3,
            x4: 4,
            x5: 5,
            x6: 6,
            x7: 7,
            x8: 8,
            x9: 9,
            x10: 10,
            x11: 11,
            x12: 12,
            x13: 13,
            x14: 14,
            x15: 15,
            x16: 16,
            x17: 17,
            x18: 18,
            x19: 19,
            x20: 20,
            x21: 21,
            x22: 22,
            x23: 23,
            x24: 24,
            x25: 25,
            x26: 26,
            x27: 27,
            x28: 28,
            x29: 29,
            x30: 30,
            ..Default::default()
        };
        let base = UserContext {
            general,
            sp: stack.0.as_mut_ptr() as usize + stack.0.len(),
            elr: dump_registers as *const () as usize,
            tpidr: initial_guest_tp,
            ..Default::default()
        };
        #[repr(C)]
        struct GuardedContext {
            context: UserContext,
            guard: [u8; 520],
        }
        let mut legacy = GuardedContext {
            context: base,
            guard: [0xa5; 520],
        };
        let mut legacy_q0 = 0;
        guest_tls[15] = usize::MAX;
        legacy.context.run_fncall();
        unsafe {
            asm!(
                "str q0, [{buffer}]",
                buffer = in(reg) &raw mut legacy_q0,
                options(nostack)
            );
        }
        assert_eq!(legacy.context.general.x0, 100);
        assert_eq!(legacy.context.tpidr, initial_guest_tp + 8);
        assert_eq!(guest_tls[15], 0);
        assert_eq!(legacy.guard, [0xa5; 520]);
        assert_eq!(legacy_q0, UPDATED_Q0);

        let mut cx = UserContextWithExtensions {
            trap_num: base.trap_num,
            __reserved: base.__reserved,
            elr: base.elr,
            spsr: base.spsr,
            sp: base.sp,
            tpidr: base.tpidr,
            general: base.general,
            ..Default::default()
        };
        let initial_q0 = 0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00;
        cx.fp_simd.registers[0] = initial_q0;
        let initial_host_d8 = 0x1357_9bdf_2468_ace0_u64;
        let initial_host_x18 = 0x1020_3040_5060_7080_u64;
        let mut restored_host_state = [0; 2];
        guest_tls[15] = usize::MAX;
        super::prepare_fncall_context(&mut cx);
        unsafe {
            test_preserve_host_state(
                &mut cx,
                &mut restored_host_state,
                initial_host_d8,
                initial_host_x18,
            )
        };
        let restored_q0 = unsafe { core::ptr::addr_of!(RESTORED_Q0).read_volatile() };
        assert_eq!(restored_q0, initial_q0);
        assert_eq!(cx.tpidr, initial_guest_tp + 8);
        assert_eq!(guest_tls[15], 0);
        assert_eq!(cx.fp_simd.registers[0], UPDATED_Q0);
        assert_eq!(restored_host_state, [initial_host_d8, initial_host_x18]);
        // check restored registers
        let general_dump = unsafe { *(cx.sp as *const GeneralRegs) };
        assert_eq!(
            general_dump,
            GeneralRegs {
                x30: dump_registers as *const () as usize,
                ..general
            }
        );
        // check saved registers
        assert_eq!(
            cx.general,
            GeneralRegs {
                x0: 100 + 0,
                x1: 100 + 1,
                x2: 100 + 2,
                x3: 100 + 3,
                x4: 100 + 4,
                x5: 100 + 5,
                x6: 100 + 6,
                x7: 100 + 7,
                x8: 100 + 8,
                x9: 100 + 9,
                x10: 100 + 10,
                x11: 100 + 11,
                x12: 100 + 12,
                x13: 100 + 13,
                x14: 100 + 14,
                x15: 100 + 15,
                x16: 100 + 16,
                x17: 100 + 17,
                x18: 100 + 18,
                x19: 100 + 19,
                x20: 100 + 20,
                x21: 100 + 21,
                x22: 100 + 22,
                x23: 100 + 23,
                x24: 100 + 24,
                x25: 100 + 25,
                x26: 100 + 26,
                x27: 100 + 27,
                x28: 100 + 28,
                x29: 100 + 29,
                x30: elr_location as *const () as usize,
                ..cx.general
            }
        );
        assert_eq!(cx.elr, elr_location as *const () as usize);
    }

    #[test]
    fn run_fncall_extended_restores_guest_x18() {
        unsafe extern "C" {
            fn observe_guest_x18();
            fn observe_guest_x18_return();
        }

        #[repr(align(16))]
        struct AlignedStack([u8; 0x1000]);

        let mut stack = AlignedStack([0; 0x1000]);
        let mut guest_tls = [0usize; 32];
        let guest_x18 = 0x1020_3040_5060_7080;
        let mut context = UserContextWithExtensions {
            elr: observe_guest_x18 as *const () as usize,
            sp: stack.0.as_mut_ptr() as usize + stack.0.len(),
            tpidr: unsafe { guest_tls.as_mut_ptr().add(8) } as usize,
            general: GeneralRegs {
                x18: guest_x18,
                ..Default::default()
            },
            ..Default::default()
        };

        context.run_fncall();

        assert_eq!(context.general.x0, guest_x18);
        assert_eq!(context.general.x18, guest_x18);
        assert_eq!(context.elr, observe_guest_x18_return as *const () as usize);
    }
}
