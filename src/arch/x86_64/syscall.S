.intel_syntax noprefix

.text
    # extern "sysv64" fn run_user(&mut GeneralRegs)
.global run_user
run_user:
    # disable interrupt
    cli

    # save callee-saved registers
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx

    push rdi
    mov gs:4, rsp           # store kernel rsp -> TSS.sp0
    mov rsp, rdi            # set rsp = bottom of trap frame

    # pop vector registers
    # fxrstor [rsp]
    .byte 0x0f
    .byte 0xae
    .byte 0x0c
    .byte 0x24
    add rsp, 512

    # pop fsbase gsbase
    swapgs                  # store kernel gsbase
    mov rax, [rsp + 18*8]   # rax = fsbase
    wrfsbase rax
    mov rax, [rsp + 19*8]   # rax = gsbase
    wrgsbase rax

    pop rax
    pop rbx
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rbp
    pop r8                  # skip rsp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    pop rcx                 # rcx = rip
    pop r11                 # r11 = rflags
    mov rsp, [rsp - 11*8]   # load rsp

    sysretq

    # sysretq instruction do:
    # - load cs, ss
    # - load rflags <- r11
    # - load rip <- rcx


.global syscall_entry
syscall_entry:
    # syscall instruction do:
    # - load cs
    # - store rflags -> r11
    # - mask rflags
    # - store rip -> rcx
    # - load rip

    swapgs                  # swap in kernel gs
    mov gs:12, rsp          # store user rsp -> scratch at TSS.sp1
    mov rsp, gs:4           # load kernel rsp <- TSS.sp0
    pop rsp                 # load rsp = bottom of trap frame
    add rsp, 512+18*8       # rsp = top of trap frame

    # push trap frame (struct GeneralRegs)
    push r11                # push rflags
    push rcx                # push rip

.global trap_syscall_entry
trap_syscall_entry:
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push gs:12              # push rsp
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push rax

    # push fsbase gsbase
    swapgs
    rdfsbase rax
    mov [rsp + 18*8], rax   # store fsbase
    rdgsbase rax
    mov [rsp + 19*8], rax   # store gsbase
    swapgs

    # save vector registers
    sub rsp, 512
    # fxsave [rsp]
    .byte 0x0f
    .byte 0xae
    .byte 0x04
    .byte 0x24

    # restore callee-saved registers
    mov rsp, gs:4           # load kernel rsp <- TSS.sp0
    pop rbx
    pop rbx
    pop rbp
    pop r12
    pop r13
    pop r14
    pop r15

    # enable interrupt
    # sti

    # go back to Rust
    ret
