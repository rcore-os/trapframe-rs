.intel_syntax noprefix

.text
.global __alltraps
__alltraps:
    push rax
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
    rdgsbase rdx
    push rdx
    rdfsbase rdx
    push rdx

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

    mov rdi, rsp
    call set_return_rsp

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
