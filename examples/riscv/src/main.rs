#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use]
extern crate opensbi_rt;

use riscv::register::scause::{Exception as E, Scause, Trap};
use trapframe::{GeneralRegs, TrapFrame, UserContext};

#[no_mangle]
extern "C" fn main() {
    unsafe {
        trapframe::init();
    }
    println!("Hello, OpenSBI!");

    let mut regs = UserContext {
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
        sstatus: 0xdead_beaf,
        sepc: user_entry as usize,
    };
    println!("Go to user: {:#x?}", regs);
    let (scause, stval) = regs.run();
    println!(
        "Back from user: {:?}, stval={:#x}\n{:#x?}",
        scause.cause(),
        stval,
        regs
    );

    unsafe {
        asm!("ebreak");
    }

    println!("Exit...");
}

#[no_mangle]
extern "C" fn trap_handler(scause: Scause, stval: usize, tf: &mut TrapFrame) {
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

unsafe extern "C" fn user_entry() {
    opensbi_rt::sbi::console_putchar(1);
}
