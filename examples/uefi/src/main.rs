#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(core_intrinsics)]
#![deny(warnings)]

extern crate alloc;

use core::intrinsics::breakpoint;
use log::*;
use trapframe::{GeneralRegs, TrapFrame, UserContext};
use uefi::prelude::*;
use x86_64::registers::control::*;
use x86_64::structures::paging::{PageTable, PageTableFlags};

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> uefi::Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    check_and_set_cpu_features();
    init_user_code();
    unsafe {
        trapframe::init();
    }

    let mut context = UserContext {
        general: GeneralRegs {
            rax: 0,
            rbx: 1,
            rcx: 2,
            rdx: 3,
            rsi: 4,
            rdi: 5,
            rbp: 6,
            rsp: 7,
            r8: 8,
            r9: 9,
            r10: 10,
            r11: 11,
            r12: 12,
            r13: 13,
            r14: 14,
            r15: 15,
            rip: 0x1000,
            rflags: 0x202,
            fsbase: 18,
            gsbase: 19,
        },
        ..Default::default()
    };

    info!("go to user");
    context.run();
    info!("back from user: {:#x?}", context); // syscall

    info!("go to user");
    context.run();
    info!("back from user: {:#x?}", context); // int3

    // trap from kernel
    unsafe {
        breakpoint();
    }
    unimplemented!()
}

#[no_mangle]
extern "sysv64" fn rust_trap(tf: &mut TrapFrame) {
    match tf.trap_num {
        3 => {
            info!("TRAP: BreakPoint");
            tf.rip += 1;
        }
        0x68 => {} // UEFI timer
        _ => panic!("TRAP: {:#x?}", tf),
    }
}

/// Initialize user code at 0x1000.
fn init_user_code() {
    allow_user_access(USER_CODE_ADDR);
    const USER_CODE_ADDR: usize = 0x1000;
    const SYSCALL_OPCODE: u16 = 0x05_0f;
    const INT3_OPCODE: u8 = 0xcc;
    unsafe {
        (USER_CODE_ADDR as *mut u16).write(SYSCALL_OPCODE);
        ((USER_CODE_ADDR + 2) as *mut u8).write(INT3_OPCODE);
    }
}

/// Set user bit for 4-level PDEs of the `page`.
/// This is a workaround since `x86_64` crate does not set user bit for PDEs.
fn allow_user_access(vaddr: usize) {
    let mut page_table = Cr3::read().0.start_address().as_u64() as *mut PageTable;
    for level in 0..4 {
        let index = (vaddr >> (12 + (3 - level) * 9)) & 0o777;
        let entry = unsafe { &mut (&mut *page_table)[index] };
        let flags = entry.flags();
        entry.set_flags(flags | PageTableFlags::USER_ACCESSIBLE);
        if level == 3 || flags.contains(PageTableFlags::HUGE_PAGE) {
            return;
        }
        page_table = entry.frame().unwrap().start_address().as_u64() as *mut PageTable;
    }
}

fn check_and_set_cpu_features() {
    assert!(raw_cpuid::CpuId::new()
        .get_extended_feature_info()
        .unwrap()
        .has_fsgsbase());
    unsafe {
        // Enable NX bit.
        Efer::update(|f| f.insert(EferFlags::NO_EXECUTE_ENABLE));

        // By default the page of CR3 have write protect.
        // We have to remove that before editing page table.
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
    }
}
