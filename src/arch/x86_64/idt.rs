use alloc::boxed::Box;
use x86_64::structures::idt::*;
use x86_64::structures::DescriptorTablePointer;

pub fn init() {
    extern "C" {
        #[link_name = "__vectors"]
        static VECTORS: [extern "C" fn(); 256];
    }

    let idt = Box::leak(Box::new(InterruptDescriptorTable::new()));
    // let idt = sidt().base;
    let entries: &'static mut [Entry<HandlerFunc>; 256] =
        unsafe { core::mem::transmute_copy(&idt) };
    for i in 0..256 {
        let _opt = entries[i].set_handler_fn(unsafe { core::mem::transmute(VECTORS[i]) });
    }
    idt.load();
}

/// Get current IDT register
#[allow(dead_code)]
#[inline]
fn sidt() -> DescriptorTablePointer {
    let mut dtp = DescriptorTablePointer { limit: 0, base: 0 };
    unsafe {
        asm!("sidt [{}]", in(reg) &mut dtp);
    }
    dtp
}
