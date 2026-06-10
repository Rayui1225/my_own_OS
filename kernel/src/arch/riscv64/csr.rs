use core::arch::asm;

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn write_stvec(addr: usize) {
    let direct_mode_addr = addr & !0b11; // stvec = Base Address + Mode (00 for direct mode 01 for vectored mode)  
    //Direct mode: all traps set stvec to the same address, and the hardware will jump to that address when a trap occurs. 
    //Vectored mode: the hardware will jump to an address calculated by adding an offset to the value in stvec. 
    //The offset is determined by the cause of the trap, allowing for different handlers for different traps.

    unsafe {
        asm!(
            "csrw stvec, {}",
            in(reg) direct_mode_addr,
            options(nostack, nomem, preserves_flags)
        );
    }
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn read_time() -> u64 {
    let value: u64;

    unsafe {
        asm!(
            "rdtime {}",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn enable_supervisor_interrupts() {
    const SSTATUS_SIE: usize = 1 << 1;

    unsafe {
        asm!(
            "csrs sstatus, {}",
            in(reg) SSTATUS_SIE,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn enable_supervisor_timer_interrupt() {
    const SIE_STIE: usize = 1 << 5;

    unsafe {
        asm!(
            "csrs sie, {}",
            in(reg) SIE_STIE,
            options(nomem, nostack, preserves_flags)
        );
    }
}
