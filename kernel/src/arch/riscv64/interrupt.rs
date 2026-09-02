use super::csr;

/// Keeps a single-hart critical section from being interrupted.
#[must_use]
pub struct InterruptGuard {
    was_enabled: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        Self {
            was_enabled: csr::disable_supervisor_interrupts(),
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.was_enabled {
            csr::enable_supervisor_interrupts();
        }
    }
}
