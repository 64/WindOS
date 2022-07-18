OUTPUT_ARCH("riscv")
OUTPUT_FORMAT("elf64-littleriscv")
ENTRY(kernel_main);

SECTIONS {
    . = 0xffffffff80000000;
    .text : ALIGN(0x1000) {
        *(.text.start)
        *(.text .text.*)
    }

    .rodata : ALIGN(0x1000) {
        *(.rodata .rodata.*)
    }

    .srodata : ALIGN(16) {
        *(.srodata .srodata.*)
    }

    .sdata : ALIGN(0x1000) {
        *(.sdata .sdata.*)
    }

    PROVIDE(__global_pointer$ = .);
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

    .eh_frame : ALIGN(0x1000) {
        *(.eh_frame)
    }

    . = ALIGN(4k);
    PROVIDE(__kernel_end_phys = .);

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
