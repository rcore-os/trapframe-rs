#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use riscv::register::scause::{Exception as E, Trap};
use riscv::register::{scause, stval};
#[cfg(target_pointer_width = "64")]
use trapframe::{FloatRegs, UserContextWithExtensions, VectorRegs};
use trapframe::{GeneralRegs, TrapFrame, UserContext};

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    la sp, bootstacktop
    call main

    .section .bss.stack
    .align 12
bootstack:
    .space 4096 * 16
bootstacktop:
"#
);

#[cfg(target_pointer_width = "64")]
global_asm!(
    r#"
    .section .text
    .globl user_entry
user_entry:
    .option push
    .option arch, +v, +d
    la t0, RESTORED_VECTOR
    vs8r.v v0, (t0)
    addi t0, t0, 128
    vs8r.v v8, (t0)
    addi t0, t0, 128
    vs8r.v v16, (t0)
    addi t0, t0, 128
    vs8r.v v24, (t0)

    la t0, RESTORED_FLOAT
    fsd f0, 0(t0)
    fsd f1, 8(t0)
    fsd f2, 16(t0)
    fsd f3, 24(t0)
    fsd f4, 32(t0)
    fsd f5, 40(t0)
    fsd f6, 48(t0)
    fsd f7, 56(t0)
    fsd f8, 64(t0)
    fsd f9, 72(t0)
    fsd f10, 80(t0)
    fsd f11, 88(t0)
    fsd f12, 96(t0)
    fsd f13, 104(t0)
    fsd f14, 112(t0)
    fsd f15, 120(t0)
    fsd f16, 128(t0)
    fsd f17, 136(t0)
    fsd f18, 144(t0)
    fsd f19, 152(t0)
    fsd f20, 160(t0)
    fsd f21, 168(t0)
    fsd f22, 176(t0)
    fsd f23, 184(t0)
    fsd f24, 192(t0)
    fsd f25, 200(t0)
    fsd f26, 208(t0)
    fsd f27, 216(t0)
    fsd f28, 224(t0)
    fsd f29, 232(t0)
    fsd f30, 240(t0)
    fsd f31, 248(t0)

    la t0, UPDATED_VECTOR
    vl8re8.v v0, (t0)
    addi t0, t0, 128
    vl8re8.v v8, (t0)
    addi t0, t0, 128
    vl8re8.v v16, (t0)
    addi t0, t0, 128
    vl8re8.v v24, (t0)

    la t0, UPDATED_FLOAT
    fld f0, 0(t0)
    fld f1, 8(t0)
    fld f2, 16(t0)
    fld f3, 24(t0)
    fld f4, 32(t0)
    fld f5, 40(t0)
    fld f6, 48(t0)
    fld f7, 56(t0)
    fld f8, 64(t0)
    fld f9, 72(t0)
    fld f10, 80(t0)
    fld f11, 88(t0)
    fld f12, 96(t0)
    fld f13, 104(t0)
    fld f14, 112(t0)
    fld f15, 120(t0)
    fld f16, 128(t0)
    fld f17, 136(t0)
    fld f18, 144(t0)
    fld f19, 152(t0)
    fld f20, 160(t0)
    fld f21, 168(t0)
    fld f22, 176(t0)
    fld f23, 184(t0)
    fld f24, 192(t0)
    fld f25, 200(t0)
    fld f26, 208(t0)
    fld f27, 216(t0)
    fld f28, 224(t0)
    fld f29, 232(t0)
    fld f30, 240(t0)
    fld f31, 248(t0)
    csrwi fcsr, 0x1f
    vsetivli zero, 8, e16, m2, ta, ma
    csrwi vcsr, 3
    ecall
    .option pop
"#
);

#[cfg(target_pointer_width = "64")]
#[unsafe(no_mangle)]
static mut RESTORED_VECTOR: [u128; 32] = [0; 32];
#[cfg(target_pointer_width = "64")]
#[unsafe(no_mangle)]
static mut RESTORED_FLOAT: [u64; 32] = [0; 32];
#[cfg(target_pointer_width = "64")]
#[unsafe(no_mangle)]
static UPDATED_VECTOR: [u128; 32] = sequence_u128(0x2000);
#[cfg(target_pointer_width = "64")]
#[unsafe(no_mangle)]
static UPDATED_FLOAT: [u64; 32] = sequence_u64(0x4000);

#[cfg(target_pointer_width = "64")]
unsafe extern "C" {
    fn user_entry();
}

#[cfg(target_pointer_width = "64")]
const fn sequence_u128(base: u128) -> [u128; 32] {
    let mut values = [0; 32];
    let mut i = 0;
    while i < values.len() {
        values[i] = base + i as u128;
        i += 1;
    }
    values
}

#[cfg(target_pointer_width = "64")]
const fn sequence_u64(base: u64) -> [u64; 32] {
    let mut values = [0; 32];
    let mut i = 0;
    while i < values.len() {
        values[i] = base + i as u64;
        i += 1;
    }
    values
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            console_putchar(byte);
        }
        Ok(())
    }
}

macro_rules! println {
    () => {
        writeln!(Stdout).unwrap()
    };
    ($($arg:tt)*) => {{
        writeln!(Stdout, $($arg)*).unwrap()
    }};
}

fn console_putchar(byte: u8) {
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") byte as usize => _,
            in("a7") 1,
        );
    }
}

fn shutdown() -> ! {
    unsafe {
        asm!("ecall", in("a7") 8, options(noreturn));
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    shutdown()
}

#[unsafe(no_mangle)]
extern "C" fn main() {
    unsafe {
        trapframe::init();
    }
    println!("Hello, OpenSBI!");

    #[cfg(target_pointer_width = "64")]
    let initial_vector = sequence_u128(0x1000);
    #[cfg(target_pointer_width = "64")]
    let initial_float = sequence_u64(0x3000);
    let context = UserContext {
        general: GeneralRegs {
            zero: 0,
            ra: 1,
            sp: 0x8080_0000,
            gp: 3,
            tp: 4,
            t0: 5,
            t1: 6,
            t2: 7,
            s0: 8,
            s1: 9,
            a0: 10,
            a1: 11,
            a2: 12,
            a3: 13,
            a4: 14,
            a5: 15,
            a6: 16,
            a7: 17,
            s2: 18,
            s3: 19,
            s4: 20,
            s5: 21,
            s6: 22,
            s7: 23,
            s8: 24,
            s9: 25,
            s10: 26,
            s11: 27,
            t3: 28,
            t4: 29,
            t5: 30,
            t6: 31,
        },
        #[cfg(target_pointer_width = "64")]
        // Enable floating-point and vector state for user mode (FS/VS=Dirty).
        sstatus: (3 << 13) | (3 << 9) | (1 << 5),
        #[cfg(target_pointer_width = "32")]
        sstatus: 1 << 5,
        sepc: user_entry as *const () as usize,
    };
    #[cfg(target_pointer_width = "32")]
    let mut regs = context;
    #[cfg(target_pointer_width = "64")]
    let mut legacy = context;
    #[cfg(target_pointer_width = "64")]
    {
        legacy.run();
        assert_eq!(scause::read().cause(), Trap::Exception(E::UserEnvCall));
        println!("RISC-V base context round-trip passed");
    }
    #[cfg(target_pointer_width = "64")]
    let mut regs = UserContextWithExtensions {
        general: context.general,
        sstatus: context.sstatus,
        sepc: context.sepc,
        vector: VectorRegs {
            registers: initial_vector,
            vstart: 0,
            vl: 16,
            vtype: 0,
            vcsr: 5,
        },
        float: FloatRegs {
            registers: initial_float,
            fcsr: 0,
        },
    };
    println!("Go to user: {:#x?}", regs);
    regs.run();
    let scause = scause::read();
    let stval = stval::read();
    println!(
        "Back from user: {:?}, stval={:#x}\n{:#x?}",
        scause.cause(),
        stval,
        regs
    );

    #[cfg(target_pointer_width = "64")]
    unsafe {
        let restored_vector = core::ptr::addr_of!(RESTORED_VECTOR).read_volatile();
        let restored_float = core::ptr::addr_of!(RESTORED_FLOAT).read_volatile();
        assert_eq!(restored_vector, initial_vector);
        assert_eq!(restored_float, initial_float);
        assert_eq!(regs.vector.registers, UPDATED_VECTOR);
        assert_eq!(regs.float.registers, UPDATED_FLOAT);
        assert_eq!(regs.vector.vl, 8);
        assert_eq!(regs.vector.vcsr, 3);
        assert_eq!(regs.float.fcsr, 0x1f);
        println!("RISC-V vector/FPU context round-trip passed");
    }

    unsafe {
        asm!("ebreak");
    }

    println!("Exit...");
    shutdown();
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(tf: &mut TrapFrame) {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => {
            println!("TRAP: Breakpoint");
            tf.sepc += 2;
        }
        _ => panic!(
            "TRAP: scause={:?}, stval={:#x}, tf={:#x?}",
            scause.cause(),
            stval,
            tf
        ),
    }
}

#[cfg(target_pointer_width = "32")]
unsafe extern "C" fn user_entry() {
    console_putchar(1);
}
