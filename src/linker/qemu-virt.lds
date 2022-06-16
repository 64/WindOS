OUTPUT_ARCH("riscv")
OUTPUT_FORMAT("elf64-littleriscv")
ENTRY(_start);

SECTIONS {
    /* Firmware is linked to 0x80000000. */
    . = 0x80100000;

    .text : ALIGN(16) {
        *(.text.start)
        *(.text .text.*)
    }

    PROVIDE(__global_pointer$ = .);

    .sdata : ALIGN(16) {
        *(.sdata .sdata.*)
    }

    .srodata : ALIGN(16) {
        *(.srodata .srodata.*)
    }

    PROVIDE(__bss_start = .);

    .sbss : ALIGN(16) {
        *(.sbss .sbss.*)
    }

    .bss : ALIGN(16) {
        *(.bss .bss.*)
    }

    PROVIDE(__bss_end = .);

    .data : ALIGN(16) {
        *(.data .data.*)
    }

    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
    }

    .eh_frame : ALIGN(16) {
        *(.eh_frame)
    }
}
