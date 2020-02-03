use alloc::boxed::Box;
use log::*;
use x86_64::structures::idt::*;
use x86_64::structures::DescriptorTablePointer;
use x86_64::PrivilegeLevel;

pub fn init() {
    extern "C" {
        #[link_name = "__vectors"]
        static VECTORS: [extern "C" fn(); 256];
    }

    // FIXME:
    //    let idt = Box::leak(Box::new(InterruptDescriptorTable::new()));
    let idt = sidt().base;
    let entries: &'static mut [Entry<HandlerFunc>; 256] =
        unsafe { core::mem::transmute_copy(&idt) };
    for i in 0..256 {
        if i == 0x68 {
            continue;
        }
        let _opt = entries[i].set_handler_fn(unsafe { core::mem::transmute(VECTORS[i]) });
    }
    //    idt.load();
}

/// Get current IDT register
#[inline]
fn sidt() -> DescriptorTablePointer {
    let mut dtp = DescriptorTablePointer { limit: 0, base: 0 };
    unsafe {
        asm!("sidt ($0)" :: "r" (&mut dtp) : "memory" : "volatile");
    }
    dtp
}
