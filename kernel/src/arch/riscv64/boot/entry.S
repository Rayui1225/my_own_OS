.section .text.entry
.globl _start

_start:
    la sp, boot_stack_top
    call kernel_main

1:
    wfi
    j 1b

.section .bss.stack, "aw", @nobits
.align 12
.globl boot_stack
boot_stack:
    .space 16384
.globl boot_stack_top
boot_stack_top:

