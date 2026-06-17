use core::sync::atomic::{AtomicUsize, Ordering};

use crate::println;

use super::{csr, sbi};

const TICK_INTERVAL: u64 = 1_000_000;
static TICKS: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub fn init() {
    csr::enable_supervisor_timer_interrupt();
    csr::enable_supervisor_interrupts();
    schedule_next_tick();
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn handle_interrupt() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    println!("[timer] tick = {}", tick);
    schedule_next_tick();
}

#[allow(dead_code)]
pub fn ticks() -> usize {
    TICKS.load(Ordering::Relaxed)
}

fn schedule_next_tick() {
    let deadline = csr::read_time().wrapping_add(TICK_INTERVAL);
    sbi::set_timer(deadline);
}
