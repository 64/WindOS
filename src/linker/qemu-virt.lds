OUTPUT_ARCH(riscv64imac)

ENTRY(_entry);

SECTIONS {
    . = 0x80000000;

    .text : ALIGN(16) {
        *(.text.entry)
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
}
