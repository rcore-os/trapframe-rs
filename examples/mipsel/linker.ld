OUTPUT_ARCH(mips)
ENTRY(_start)

SECTIONS
{
    . = 0x80100000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        *(.bss.stack)
        *(.bss .bss.*)
    }
}
