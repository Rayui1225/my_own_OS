const INTERRUPT_BIT: usize = 1usize << (usize::BITS - 1);

#[repr(C)]
pub struct TrapFrame {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub sepc: usize,
    pub sstatus: usize,
    pub scause: usize,
    pub stval: usize,
    pub _reserved: usize,
}

impl TrapFrame {
    pub fn cause(&self) -> TrapCause {
        TrapCause::from_scause(self.scause)
    }

    pub fn advance_sepc_by_4(&mut self) {
        self.sepc = self.sepc.wrapping_add(4);
    }
}

#[derive(Clone, Copy)]
pub enum TrapCause {
    Breakpoint,
    IllegalInstruction,
    LoadPageFault,
    StorePageFault,
    UserEcall,
    SupervisorTimerInterrupt,
    Unknown { interrupt: bool, code: usize },
}

impl TrapCause {
    fn from_scause(scause: usize) -> Self {
        let interrupt = (scause & INTERRUPT_BIT) != 0;
        let code = scause & !INTERRUPT_BIT;

        match (interrupt, code) {
            (false, 3) => Self::Breakpoint,
            (false, 2) => Self::IllegalInstruction,
            (false, 13) => Self::LoadPageFault,
            (false, 15) => Self::StorePageFault,
            (false, 8) => Self::UserEcall,
            (true, 5) => Self::SupervisorTimerInterrupt,
            _ => Self::Unknown { interrupt, code },
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Breakpoint => "breakpoint",
            Self::IllegalInstruction => "illegal instruction",
            Self::LoadPageFault => "load page fault",
            Self::StorePageFault => "store page fault",
            Self::UserEcall => "ecall from user mode",
            Self::SupervisorTimerInterrupt => "timer interrupt",
            Self::Unknown { interrupt: true, .. } => "unknown interrupt",
            Self::Unknown { interrupt: false, .. } => "unknown exception",
        }
    }
}
