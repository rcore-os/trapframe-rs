# User contexts are published in FNCALL_CONTEXTS under the thread pointer that
# will be loaded before entering user code.  This keeps trapframe metadata out
# of the guest libc's TLS block.

syscall_fn_entry:
    # Save the registers used to find the context in FNCALL_CONTEXTS.
    sub     sp, sp, #48
    stp     x0, x1, [sp]
    stp     x2, x3, [sp, #16]
    str     x30, [sp, #32]

    # Take the context registered for this user thread pointer.  The acquire
    # load pairs with register_fncall_context's release publication.
    mrs     x0, tpidr_el0
    LOAD_CONTEXT_TABLE x1
    mov     x2, #256
1:  ldar    x3, [x1]
    cmp     x3, x0
    b.eq    2f
    add     x1, x1, #16
    subs    x2, x2, #1
    b.ne    1b
    brk     #0
2:  ldr     x0, [x1, #8]
    stlr    xzr, [x1]
    cbz     x0, 3f

    # Switch to the context stack and restore the lookup scratch registers.
    add     x30, sp, #48         // x30 = user stack
    str     x30, [x0, #4 * 8]   // save user stack
    add     sp, x0, #38 * 8     // sp = top of user context
    ldp     x2, x3, [x30, #-32]
    ldp     x0, x1, [x30, #-48]
    ldr     x30, [x30, #-16]
    b       4f
3:  brk     #0
4:

    # save general registers
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

    # save floating-point and Advanced SIMD state
    sub     x4, sp, #48
    ldr     x5, [x4, #8]
    tbz     x5, #0, 9f
    stp     q0, q1, [x4, #304]
    stp     q2, q3, [x4, #336]
    stp     q4, q5, [x4, #368]
    stp     q6, q7, [x4, #400]
    stp     q8, q9, [x4, #432]
    stp     q10, q11, [x4, #464]
    stp     q12, q13, [x4, #496]
    stp     q14, q15, [x4, #528]
    stp     q16, q17, [x4, #560]
    stp     q18, q19, [x4, #592]
    stp     q20, q21, [x4, #624]
    stp     q22, q23, [x4, #656]
    stp     q24, q25, [x4, #688]
    stp     q26, q27, [x4, #720]
    stp     q28, q29, [x4, #752]
    stp     q30, q31, [x4, #784]
    mrs     x2, fpcr
    mrs     x3, fpsr
    add     x5, x4, #816
    stp     w2, w3, [x5]

9:
    # skip sp and save tpidr
    mrs     x1, tpidr_el0
    str     x1, [sp, #-8]
    add     sp, sp, #-16

    # skip spsr and save elr(lr)
    ldr     x1, [sp, #32*8]
    str     x1, [sp, #-16]!

    # skip trap num and read kernel sp
    ldr     x1, [sp, #-8]
    bic     x1, x1, #1
    str     x1, [sp, #-8]
    mov     sp, x1

    # load kernel tp
    ldr     x1, [sp], #16
    msr     tpidr_el0, x1

    # restore the host floating-point environment and the SIMD registers that
    # are callee-saved by AAPCS64.  The user context may freely modify these,
    # but syscall_fn_return is still an ordinary host function call as far as
    # its Rust caller is concerned.
    ldp     x2, x3, [sp], #16
    msr     fpcr, x2
    msr     fpsr, x3
    ldp     d8, d9, [sp], #16
    ldp     d10, d11, [sp], #16
    ldp     d12, d13, [sp], #16
    ldp     d14, d15, [sp], #16

    # load callee-saved registers
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16

    # x18 is the platform register and must survive calls on Darwin. Saving it
    # unconditionally also keeps this entry path ABI-safe on other hosts.
    ldr     x18, [sp], #16

    ret

    # extern "C" fn syscall_fn_return(&mut UserContext)
syscall_fn_return:
    mov     x10, #0
    b       8f
syscall_fn_return_extended:
    mov     x10, #1
8:
    # Preserve the host platform register before loading the user register
    # set. Apple reserves x18 for use by the operating system.
    str     x18, [sp, #-16]!

    # save callee-saved registers
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!

    # Preserve the host floating-point environment and the low 64 bits of
    # v8-v15, which AAPCS64 requires a function to keep intact.
    stp     d14, d15, [sp, #-16]!
    stp     d12, d13, [sp, #-16]!
    stp     d10, d11, [sp, #-16]!
    stp     d8, d9, [sp, #-16]!
    mrs     x11, fpcr
    mrs     x12, fpsr
    stp     x11, x12, [sp, #-16]!

    # save kernel tp
    mrs     x8, tpidr_el0   // x8 = kernel tp
    str     x8, [sp, #-16]!

    # save kernel sp to UserContext
    mov     x9, sp
    orr     x9, x9, x10
    str     x9, [x0, #8]

    cbz     x10, 7f
    # restore floating-point and Advanced SIMD state
    ldp     q0, q1, [x0, #304]
    ldp     q2, q3, [x0, #336]
    ldp     q4, q5, [x0, #368]
    ldp     q6, q7, [x0, #400]
    ldp     q8, q9, [x0, #432]
    ldp     q10, q11, [x0, #464]
    ldp     q12, q13, [x0, #496]
    ldp     q14, q15, [x0, #528]
    ldp     q16, q17, [x0, #560]
    ldp     q18, q19, [x0, #592]
    ldp     q20, q21, [x0, #624]
    ldp     q22, q23, [x0, #656]
    ldp     q24, q25, [x0, #688]
    ldp     q26, q27, [x0, #720]
    ldp     q28, q29, [x0, #752]
    ldp     q30, q31, [x0, #784]
    add     x3, x0, #816
    ldp     w1, w2, [x3]
    msr     fpcr, x1
    msr     fpsr, x2

7:
    # pop tpidr
    ldr     x9, [x0, #5*8]  // x9 = user tp
    cbnz    x9, 1f          // if not 0, goto set
    INIT_USER_TP x9, x8     // x9 = init user tp
1:  msr     tpidr_el0, x9   // tp = x9

    # pop elr, sp
    ldr     x30, [x0, #2*8]
    ldr     x8, [x0, #4*8]
    mov     sp, x8

    # pop general registers
    add     x0, x0, #6*8
    ldp     x1, x2, [x0], #16
    ldp     x3, x4, [x0], #16
    ldp     x5, x6, [x0], #16
    ldp     x7, x8, [x0], #16
    ldp     x9, x10, [x0], #16
    ldp     x11, x12, [x0], #16
    ldp     x13, x14, [x0], #16
    ldp     x15, x16, [x0], #16
    ldp     x17, x18, [x0], #16
    ldp     x19, x20, [x0], #16
    ldp     x21, x22, [x0], #16
    ldp     x23, x24, [x0], #16
    ldp     x25, x26, [x0], #16
    ldp     x27, x28, [x0], #16
    ldr     x29, [x0], #16
    ldr     x0, [x0, #8]
    ret
