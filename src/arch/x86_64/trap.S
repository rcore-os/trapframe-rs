.intel_syntax noprefix

.text
.global __alltraps
__alltraps:
    swapgs                  # swap in kernel gs
    push rax

    /*
    kernel stack:
    - ptr to GeneralRegs
    - ss
    - rsp
    - rflags
    - cs
    - rip
    - error code
    - trap num
    - rax
    */
    mov ax, [rsp + 4*8]     # load cs
    and ax, 0x3             # test
    jz __from_kernel        # continue trap

    mov rax, [rsp + 6*8]    # rax = user rsp
    mov gs:12, rax          # store user rsp -> scratch at TSS.sp1

    mov rsp, [rsp + 8*8]    # load rsp = bottom of trap frame
    add rsp, 512+18*8       # rsp = top of trap frame
    mov rax, gs:4           # rax = kernel stack

    # push trap frame (struct GeneralRegs)
    push [rax - 3*8]        # push rflags
    push [rax - 5*8]        # push rip
    mov rax, [rax - 8*8]    # pop rax
    jmp trap_syscall_entry

__from_kernel:
    push rcx
    push rdx
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11

    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    # push fsbase gsbase
    swapgs
    rdgsbase rdx
    push rdx
    rdfsbase rdx
    push rdx
    swapgs

    # save fp registers
    # align to 16 byte boundary
    sub rsp, 512
    mov rax, rsp
    and rax, 0xFFFFFFFFFFFFFFF0
    # fxsave (rax)
    .byte 0x0f
    .byte 0xae
    .byte 0x00
    mov rcx, rsp
    sub rcx, rax
    # push fp state offset
    sub rsp, 16
    push rcx

    mov rdi, rsp
    call rust_trap

.global trap_ret
trap_ret:

    mov gs:4, rsp           # store kernel rsp -> TSS.sp0

    # pop fp state offset
    pop rcx
    cmp rcx, 16 # only 0-15 are valid
    jge skip_fxrstor
    mov rax, rsp
    add rax, 16
    sub rax, rcx
    # fxrstor (rax)
    .byte 0x0f
    .byte 0xae
    .byte 0x08
skip_fxrstor:
    add rsp, 16+512

    # pop fsbase gsbase
    swapgs
    pop rdx
    wrfsbase rdx
    pop rdx
    wrgsbase rdx

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx

    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rdx
    pop rcx
    pop rax

    # pop trap_num, error_code
    add rsp, 16

    iretq
