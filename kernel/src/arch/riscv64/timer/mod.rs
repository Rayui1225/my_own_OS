use core::sync::atomic::{AtomicUsize, Ordering};

use super::{csr, sbi};

const TICK_INTERVAL: u64 = 1_000_000;
static TICKS: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub fn init() {
    schedule_next_tick();
    csr::enable_supervisor_timer_interrupt();
    csr::enable_supervisor_interrupts();
}

pub fn stop() {
    csr::disable_supervisor_timer_interrupt();
    sbi::set_timer(u64::MAX);
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn handle_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    // The current handler may be suspended by the switch, so arm first.
    schedule_next_tick();
    crate::task::on_timer_tick();
}

#[allow(dead_code)]
pub fn ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

fn schedule_next_tick() {
    let deadline = csr::read_time().wrapping_add(TICK_INTERVAL);
    sbi::set_timer(deadline);
}
