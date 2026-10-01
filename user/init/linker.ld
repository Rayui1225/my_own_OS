OUTPUT_ARCH(riscv)
ENTRY(_start)

USER_TEXT_BASE = 0x00010000;

SECTIONS
{
    . = USER_TEXT_BASE;

    .text : ALIGN(4K) {
        *(.text.entry)
        *(.text .text.*)
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.sdata .sdata.*)
        /* Keep the file-backed data size consistent with the BSS virtual start. */
        . = ALIGN(8);
    }

    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
