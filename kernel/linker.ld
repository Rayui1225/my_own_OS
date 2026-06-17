OUTPUT_ARCH(riscv)
ENTRY(_start)

KERNEL_BASE = 0x80200000;
BOOT_STACK_SIZE = 16K;

SECTIONS
{
    . = KERNEL_BASE;
    PROVIDE(skernel = .);

    .text : ALIGN(4K) {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .boot_stack (NOLOAD) : ALIGN(4K) {
        *(.bss.stack)
    }

    .bss : ALIGN(4K) {
        PROVIDE(sbss = .);
        *(.bss .bss.*)
        *(COMMON)
        PROVIDE(ebss = .);
    }

    PROVIDE(ekernel = .);

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
