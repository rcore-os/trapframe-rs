	.set noat
	.set noreorder
    .section .text
    .balign 4096
    .global trap_entry

trap_entry:
    b general_trap_vec

.org 0x180
general_trap_vec:
    move $k1, $sp
    # read cp0.status
    mfc0 $k0, $12
    # cp0.status.ksu
    andi $k0, $k0, 0x10
    beq $k0, $zero, trap_from_kernel
    nop

trap_from_user:

trap_from_kernel: