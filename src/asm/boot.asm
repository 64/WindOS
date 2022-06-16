.section .text.start
.global _start
_start:
    # a0 contains hartid
    # a1 contains dtb

    # Zero out the BSS sections.
#     la t0, __bss_start
#     la t1, __bss_end
#     bgeu t0, t1, .bss_zero_loop_end
# .bss_zero_loop:
#     sd zero, (t0)
#     addi t0, t0, 8
#     bltu t0, t1, .bss_zero_loop
# .bss_zero_loop_end:

    la sp, stack_top

    # Jump to Rust.
    jal kernel_main
    unimp

.section .bss
    .skip 0x10000
stack_top:
