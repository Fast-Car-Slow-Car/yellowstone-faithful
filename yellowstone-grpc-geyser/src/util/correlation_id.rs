use std::sync::atomic::{AtomicU64, Ordering};

pub fn next_correlation_id(slot: u64) -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    (slot << 32) | (counter & 0xFFFF_FFFF)
}