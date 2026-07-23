OUTPUT_ARCH(riscv)
ENTRY(_start)

KERNEL_BASE = 0x80200000;
BOOT_STACK_SIZE = 16K;

SECTIONS
{
    . = KERNEL_BASE;
    PROVIDE(skernel = .);

    .text : ALIGN(4K) {
        PROVIDE(stext = .);
        *(.text.entry)
        *(.text .text.*)
        PROVIDE(etext = .);
    }

    .rodata : ALIGN(4K) {
        PROVIDE(srodata = .);
        *(.rodata .rodata.*)
        PROVIDE(erodata = .);
    }

    .data : ALIGN(4K) {
        PROVIDE(sdata = .);
        *(.data .data.*)
        PROVIDE(edata = .);
    }

    .boot_stack (NOLOAD) : ALIGN(4K) {
        PROVIDE(sboot_stack = .);
        *(.bss.stack)
        PROVIDE(eboot_stack = .);
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
