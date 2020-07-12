//! Configure Global Descriptor Table (GDT)

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use core::slice::from_raw_parts_mut;

use x86_64::instructions::tables::{lgdt, load_tss};
use x86_64::registers::model_specific::{GsBase, Star};
use x86_64::structures::gdt::{Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::DescriptorTablePointer;
use x86_64::{PrivilegeLevel, VirtAddr};

/// TSS with port bitmap, allocated consecutively.
///
/// Bit in port_bitmap: 0 indicates accessible, 1 indicated inaccessible.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegmentPortBitmap {
    pub tss: TaskStateSegment,
    // Follow linux, add one extra element.
    pub port_bitmap: [u8; 1 + Self::BITMAP_VALID_SIZE],
}

impl TaskStateSegmentPortBitmap {
    pub const BITMAP_VALID_SIZE: usize = 8192;
    pub const PORT_BITMAP_OFFSET: usize = size_of::<TaskStateSegment>();

    fn new() -> Self {
        const ALLOW_ALL: u8 = 0;
        Self {
            tss: TaskStateSegment::new(),
            port_bitmap: [ALLOW_ALL; 1 + Self::BITMAP_VALID_SIZE]
        }
    }

}

pub fn gsbase_to_bitmap<'a>(gsbase: u64) -> &'a mut [u8] {
    unsafe {
        from_raw_parts_mut(
            (gsbase as usize + TaskStateSegmentPortBitmap::PORT_BITMAP_OFFSET) as *mut u8,
            TaskStateSegmentPortBitmap::BITMAP_VALID_SIZE)
    }
}

/// Init TSS & GDT.
pub fn init() {
    // allocate stack for trap from user
    // set the stack top to TSS
    // so that when trap from ring3 to ring0, CPU can switch stack correctly
    let mut tss = Box::new(TaskStateSegmentPortBitmap::new());
    let trap_stack_top = Box::leak(Box::new([0u8; 0x1000])).as_ptr() as u64 + 0x1000;
    tss.tss.privilege_stack_table[0] = VirtAddr::new(trap_stack_top);
    tss.tss.iomap_base = TaskStateSegmentPortBitmap::PORT_BITMAP_OFFSET as u16;
    let tss: &'static _ = Box::leak(tss);
    let (tss0, tss1) = match Descriptor::tss_segment(&tss.tss) {
        // Extreme hack: the segment limit assumed by x86_64 does not include the port bitmap.
        Descriptor::SystemSegment(tss0, tss1) => {
            let tss0  = tss0 & !0xFFFF;
            let tss0 = tss0 | (size_of::<TaskStateSegmentPortBitmap>() as u64);
            (tss0, tss1)
        },
        _ => unreachable!(),
    };

    unsafe {
        // get current GDT
        let gdtp = sgdt();
        let entry_count = (gdtp.limit + 1) as usize / size_of::<u64>();
        let old_gdt = core::slice::from_raw_parts(gdtp.base as *const u64, entry_count);

        // allocate new GDT with 7 more entries
        //
        // NOTICE: for fast syscall:
        //   STAR[47:32] = K_CS   = K_SS - 8
        //   STAR[63:48] = U_CS32 = U_SS32 - 8 = U_CS - 16
        let mut gdt = Vec::from(old_gdt);
        gdt.extend([tss0, tss1, KCODE64, KDATA64, UCODE32, UDATA32, UCODE64].iter());
        let gdt = Vec::leak(gdt);

        // load new GDT and TSS
        lgdt(&DescriptorTablePointer {
            limit: gdt.len() as u16 * 8 - 1,
            base: gdt.as_ptr() as _,
        });
        load_tss(SegmentSelector::new(
            entry_count as u16,
            PrivilegeLevel::Ring0,
        ));

        // for fast syscall:
        // store address of TSS to kernel_gsbase
        GsBase::MSR.write(tss as *const _ as u64);

        Star::write_raw(
            SegmentSelector::new(entry_count as u16 + 4, PrivilegeLevel::Ring3).0,
            SegmentSelector::new(entry_count as u16 + 2, PrivilegeLevel::Ring0).0,
        );
    }
}

/// Get current GDT register
#[inline]
unsafe fn sgdt() -> DescriptorTablePointer {
    let mut gdt = DescriptorTablePointer { limit: 0, base: 0 };
    asm!("sgdt [{}]", in(reg) &mut gdt);
    gdt
}

const KCODE64: u64 = 0x00209800_00000000; // EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE
const UCODE64: u64 = 0x0020F800_00000000; // EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT | LONG_MODE
const KDATA64: u64 = 0x00009200_00000000; // DATA_WRITABLE | USER_SEGMENT | PRESENT
#[allow(dead_code)]
const UDATA64: u64 = 0x0000F200_00000000; // DATA_WRITABLE | USER_SEGMENT | USER_MODE | PRESENT
const UCODE32: u64 = 0x00cffa00_0000ffff; // EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT
const UDATA32: u64 = 0x00cff200_0000ffff; // EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT
