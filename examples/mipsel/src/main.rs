#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use trapframe::TrapFrame;

global_asm!(
    r#"
    .set noat
    .set noreorder
    .section .text.entry
    .global _start
_start:
    la $sp, bootstacktop
    jal main
    nop

1:
    wait
    b 1b
    nop

    .section .bss.stack
    .align 12
bootstack:
    .space 4096 * 4
bootstacktop:
"#
);

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    write_str("MIPS trap test start\n");

    unsafe {
        trapframe::init();
    }
    write_str("MIPS trap initialized\n");
    write_str("MIPS triggering breakpoint\n");
    unsafe {
        asm!("break");
    }

    halt()
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(tf: &mut TrapFrame) {
    const BREAKPOINT_EXCEPTION: usize = 9;
    if (tf.cause >> 2) & 0x1f == BREAKPOINT_EXCEPTION {
        write_str("MIPS trap round-trip passed\n");
    } else {
        write_str("MIPS unexpected kernel trap: 0x");
        write_hex(tf.cause);
        write_str("\n");
    }
    halt()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    write_str("MIPS panic\n");
    halt()
}

fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

fn write_hex(mut value: usize) {
    let mut digits = [b'0'; 8];
    for digit in digits.iter_mut().rev() {
        let nibble = (value & 0xf) as u8;
        *digit = if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        };
        value >>= 4;
    }
    for digit in digits {
        write_byte(digit);
    }
}

fn write_byte(byte: u8) {
    const UART_BASE: usize = 0xb800_03f8;
    const LINE_STATUS: usize = 5;
    const TRANSMIT_READY: u8 = 1 << 6;

    unsafe {
        while read_volatile((UART_BASE + LINE_STATUS) as *const u8) & TRANSMIT_READY == 0 {}
        write_volatile(UART_BASE as *mut u8, byte);
    }
}

fn halt() -> ! {
    loop {
        unsafe {
            asm!("wait");
        }
    }
}
