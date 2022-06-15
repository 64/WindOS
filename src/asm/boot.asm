.section .text.entry
.global _entry
_entry:
    # Only the bootstrap hart should continue.
    csrr t0, mhartid
    bnez t0, .wait_for_interrupts

    csrw satp, zero

    la gp, __global_pointer$

    # Zero out the BSS sections.
    la a0, __bss_start
    la a1, __bss_end
    bgeu a0, a1, .bss_zero_loop_end
.bss_zero_loop:
    sd zero, (a0)
    addi a0, a0, 8
    bltu a0, a1, .bss_zero_loop
.bss_zero_loop_end:

.wait_for_interrupts:
    wfi
    j .wait_for_interrupts
