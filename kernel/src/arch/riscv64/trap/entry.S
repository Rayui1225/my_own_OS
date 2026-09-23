.section .text.trap
.balign 4
.globl __trap_entry
.globl __enter_user

.equ TRAP_FRAME_SIZE, 288
.equ SSTATUS_SPP, 0x100

.macro SAVE_REGISTERS
    sd ra, 0(sp)
    sd gp, 16(sp)
    sd tp, 24(sp)
    sd t0, 32(sp)
    sd t1, 40(sp)
    sd t2, 48(sp)
    sd s0, 56(sp)
    sd s1, 64(sp)
    sd a0, 72(sp)
    sd a1, 80(sp)
    sd a2, 88(sp)
    sd a3, 96(sp)
    sd a4, 104(sp)
    sd a5, 112(sp)
    sd a6, 120(sp)
    sd a7, 128(sp)
    sd s2, 136(sp)
    sd s3, 144(sp)
    sd s4, 152(sp)
    sd s5, 160(sp)
    sd s6, 168(sp)
    sd s7, 176(sp)
    sd s8, 184(sp)
    sd s9, 192(sp)
    sd s10, 200(sp)
    sd s11, 208(sp)
    sd t3, 216(sp)
    sd t4, 224(sp)
    sd t5, 232(sp)
    sd t6, 240(sp)
.endm

.macro RESTORE_REGISTERS
    ld ra, 0(sp)
    ld gp, 16(sp)
    ld tp, 24(sp)
    ld t0, 32(sp)
    ld t1, 40(sp)
    ld t2, 48(sp)
    ld s0, 56(sp)
    ld s1, 64(sp)
    ld a0, 72(sp)
    ld a1, 80(sp)
    ld a2, 88(sp)
    ld a3, 96(sp)
    ld a4, 104(sp)
    ld a5, 112(sp)
    ld a6, 120(sp)
    ld a7, 128(sp)
    ld s2, 136(sp)
    ld s3, 144(sp)
    ld s4, 152(sp)
    ld s5, 160(sp)
    ld s6, 168(sp)
    ld s7, 176(sp)
    ld s8, 184(sp)
    ld s9, 192(sp)
    ld s10, 200(sp)
    ld s11, 208(sp)
    ld t3, 216(sp)
    ld t4, 224(sp)
    ld t5, 232(sp)
    ld t6, 240(sp)
.endm

__trap_entry:
    # U-mode keeps its kernel stack pointer in sscratch. S-mode keeps sscratch zero.
    csrrw sp, sscratch, sp
    bnez sp, .Ltrap_from_user

    # A supervisor trap swapped its kernel sp into sscratch; swap it back.
    csrrw sp, sscratch, sp
    addi sp, sp, -TRAP_FRAME_SIZE
    SAVE_REGISTERS
    addi t0, sp, TRAP_FRAME_SIZE
    sd t0, 8(sp)
    j .Lsave_csrs

.Ltrap_from_user:
    addi sp, sp, -TRAP_FRAME_SIZE
    SAVE_REGISTERS
    csrr t0, sscratch
    sd t0, 8(sp)
    csrw sscratch, zero

.Lsave_csrs:
    csrr t0, sepc
    sd t0, 248(sp)
    csrr t0, sstatus
    sd t0, 256(sp)
    csrr t0, scause
    sd t0, 264(sp)
    csrr t0, stval
    sd t0, 272(sp)
    sd zero, 280(sp)

    mv a0, sp
    call trap_entry_rust

    ld t0, 248(sp)
    csrw sepc, t0
    ld t0, 256(sp)
    csrw sstatus, t0
    andi t0, t0, SSTATUS_SPP
    bnez t0, .Lreturn_to_supervisor

    # Preserve both stack pointers across the final register restore.
    ld t0, 8(sp)
    csrw sscratch, t0
    RESTORE_REGISTERS
    addi sp, sp, TRAP_FRAME_SIZE
    csrrw sp, sscratch, sp
    sret

.Lreturn_to_supervisor:
    csrw sscratch, zero
    RESTORE_REGISTERS
    addi sp, sp, TRAP_FRAME_SIZE
    sret

# Restores a prepared TrapFrame and performs the first S-mode to U-mode transition.
# a0 points at the frame; the current kernel sp becomes the U-mode sscratch value.
__enter_user:
    mv t6, a0
    csrw sscratch, sp

    ld t0, 248(t6)
    csrw sepc, t0
    ld t0, 256(t6)
    csrw sstatus, t0

    ld ra, 0(t6)
    ld sp, 8(t6)
    ld gp, 16(t6)
    ld tp, 24(t6)
    ld t0, 32(t6)
    ld t1, 40(t6)
    ld t2, 48(t6)
    ld s0, 56(t6)
    ld s1, 64(t6)
    ld a0, 72(t6)
    ld a1, 80(t6)
    ld a2, 88(t6)
    ld a3, 96(t6)
    ld a4, 104(t6)
    ld a5, 112(t6)
    ld a6, 120(t6)
    ld a7, 128(t6)
    ld s2, 136(t6)
    ld s3, 144(t6)
    ld s4, 152(t6)
    ld s5, 160(t6)
    ld s6, 168(t6)
    ld s7, 176(t6)
    ld s8, 184(t6)
    ld s9, 192(t6)
    ld s10, 200(t6)
    ld s11, 208(t6)
    ld t3, 216(t6)
    ld t4, 224(t6)
    ld t5, 232(t6)
    ld t6, 240(t6)
    sret
