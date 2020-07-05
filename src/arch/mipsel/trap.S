#define zero    $0
#define AT	$1
#define v0	$2
#define v1	$3
#define a0	$4
#define a1	$5
#define a2	$6
#define a3	$7
#define t0	$8
#define t1	$9
#define t2	$10
#define t3	$11
#define t4	$12
#define t5	$13
#define t6	$14
#define t7	$15
#define s0	$16
#define s1	$17
#define s2	$18
#define s3	$19
#define s4	$20
#define s5	$21
#define s6	$22
#define s7	$23
#define t8	$24
#define t9	$25
#define jp	$25
#define k0	$26
#define k1	$27
#define gp	$28
#define sp	$29
#define fp	$30
#define ra	$31

	.set noat
	.set noreorder
    .section .text
    .balign 4096
    .global trap_entry

trap_entry:
    b general_trap_vec

.org 0x180
general_trap_vec:
    move k1, sp
    # read cp0.status
    mfc0 k0, $12
    # cp0.status.ksu
    andi k0, k0, 0x10
    beq k0, zero, trap_from_kernel
    nop

trap_from_user:

trap_from_kernel: