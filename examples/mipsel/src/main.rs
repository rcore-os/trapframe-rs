#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use trapframe::{GeneralRegs, TrapFrame, UserContext};

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
    unsafe {
        trapframe::init();
    }

    let mut context = UserContext {
        general: GeneralRegs {
            sp: 0x8100_0000,
            ..Default::default()
        },
        epc: user_entry as *const () as usize,
        ..Default::default()
    };
    context.run();

    halt()
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(_tf: &mut TrapFrame) {}

unsafe extern "C" fn user_entry() {
    unsafe {
        asm!("break");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt()
}

fn halt() -> ! {
    loop {
        unsafe {
            asm!("wait");
        }
    }
}
