# Constants / Macros defined in Rust code:
#   XLENB
#   LOAD
#   STORE

.macro SAVE_ALL
    # If coming from userspace, preserve the user stack pointer and load
    # the kernel stack pointer. If we came from the kernel, sscratch
    # will contain 0, and we should continue on the current stack.
    csrrw sp, sscratch, sp
    bnez sp, trap_from_user
trap_from_kernel:
    csrr sp, sscratch
    STORE_TF gp, -36
    # sscratch = previous-sp, sp = kernel-sp
trap_from_user:
    # provide room for trap frame
    addi sp, sp, -36 * XLENB
    # save x registers except x2 (sp)
    STORE_TF x1, 1
    STORE_TF x3, 3
    STORE_TF x4, 4
    STORE_TF x5, 5
    STORE_TF x6, 6
    STORE_TF x7, 7
    STORE_TF x8, 8
    STORE_TF x9, 9
    STORE_TF x10, 10
    STORE_TF x11, 11
    STORE_TF x12, 12
    STORE_TF x13, 13
    STORE_TF x14, 14
    STORE_TF x15, 15
    STORE_TF x16, 16
    STORE_TF x17, 17
    STORE_TF x18, 18
    STORE_TF x19, 19
    STORE_TF x20, 20
    STORE_TF x21, 21
    STORE_TF x22, 22
    STORE_TF x23, 23
    STORE_TF x24, 24
    STORE_TF x25, 25
    STORE_TF x26, 26
    STORE_TF x27, 27
    STORE_TF x28, 28
    STORE_TF x29, 29
    STORE_TF x30, 30
    STORE_TF x31, 31

    # load hartid to gp from sp[0]
    LOAD_TF gp, 0

    # get sp, sstatus, sepc, stval, scause
    # set sscratch = 0
    csrrw s0, sscratch, zero
    csrr s1, sstatus
    csrr s2, sepc
    csrr a1, scause
    csrr a2, stval
    addi s3, sp, 36 * XLENB
    # store sp, sstatus, sepc, kernel-sp
    STORE_TF s0, 2
    STORE_TF s1, 32
    STORE_TF s2, 33
    STORE_TF s3, 34
.endm

.macro RESTORE_ALL
    LOAD_TF s1, 32          # s1 = sstatus
    LOAD_TF s2, 33          # s2 = sepc
    LOAD_TF s3, 34          # s3 = kernel-sp
    andi s0, s1, 1 << 8     # sstatus.SPP = 1
    bnez s0, back_to_kernel # s0 = back to kernel?
back_to_user:
    csrw sscratch, s3       # sscratch = kernel-sp
    STORE gp, -36*XLENB(s3) # store hartid from gp to kernel-sp[0]
back_to_kernel:
    # restore sstatus, sepc
    csrw sstatus, s1
    csrw sepc, s2

    # restore x registers except x2 (sp)
    LOAD_TF x1, 1
    LOAD_TF x3, 3
    LOAD_TF x4, 4
    LOAD_TF x5, 5
    LOAD_TF x6, 6
    LOAD_TF x7, 7
    LOAD_TF x8, 8
    LOAD_TF x9, 9
    LOAD_TF x10, 10
    LOAD_TF x11, 11
    LOAD_TF x12, 12
    LOAD_TF x13, 13
    LOAD_TF x14, 14
    LOAD_TF x15, 15
    LOAD_TF x16, 16
    LOAD_TF x17, 17
    LOAD_TF x18, 18
    LOAD_TF x19, 19
    LOAD_TF x20, 20
    LOAD_TF x21, 21
    LOAD_TF x22, 22
    LOAD_TF x23, 23
    LOAD_TF x24, 24
    LOAD_TF x25, 25
    LOAD_TF x26, 26
    LOAD_TF x27, 27
    LOAD_TF x28, 28
    LOAD_TF x29, 29
    LOAD_TF x30, 30
    LOAD_TF x31, 31
    # restore sp last
    LOAD_TF x2, 2
.endm

    .section .text
    .globl trap_entry
trap_entry:
    SAVE_ALL
    mv a0, sp
    call trap_handler
    .globl trap_return
trap_return:
    RESTORE_ALL
    # return from supervisor call
    sret
