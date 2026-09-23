.section .rodata.user_init, "a", @progbits
.balign 4
.globl __user_init_start
.globl __user_init_end

.equ SYS_WRITE, 1
.equ SYS_EXIT, 2
.equ SYS_YIELD, 3
.equ SYS_GETPID, 4

__user_init_start:
    li a0, 1
.Lmessage_address:
    auipc a1, %pcrel_hi(.Lmessage)
    addi a1, a1, %pcrel_lo(.Lmessage_address)
    li a2, .Lmessage_end - .Lmessage
    li a7, SYS_WRITE
    ecall
    li t0, .Lmessage_end - .Lmessage
    bne a0, t0, .Lwrite_failed

    # A supervisor-only kernel mapping must not pass user-pointer validation.
    li a0, 1
    li a1, 0x80200000
    li a2, 1
    li a7, SYS_WRITE
    ecall
    li t0, -14
    bne a0, t0, .Lpointer_check_failed

    li a7, SYS_GETPID
    ecall
    li t0, 1
    bne a0, t0, .Lgetpid_failed

    li a7, SYS_YIELD
    ecall
    bnez a0, .Lyield_failed

    li a0, 0
    li a7, SYS_EXIT
    ecall

.Lwrite_failed:
    li a0, 10
    j .Lexit_failed
.Lpointer_check_failed:
    li a0, 11
    j .Lexit_failed
.Lgetpid_failed:
    li a0, 12
    j .Lexit_failed
.Lyield_failed:
    li a0, 13
.Lexit_failed:
    li a7, SYS_EXIT
    ecall

.Lhang:
    j .Lhang

.Lmessage:
    .ascii "[user] hello from user\n"
.Lmessage_end:

__user_init_end:
