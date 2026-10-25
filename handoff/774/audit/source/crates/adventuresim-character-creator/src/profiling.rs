//! Opt-in wall-clock timings for generation and export diagnostics.

use serde::Serialize;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

static ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
struct Timing<'a> {
    label: &'a str,
    milliseconds: f64,
}

pub fn enable(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn measure<T>(label: &str, operation: impl FnOnce() -> T) -> T {
    if !ENABLED.load(Ordering::Relaxed) {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    record(label, started.elapsed());
    result
}

pub fn record(label: &str, elapsed: Duration) {
    if ENABLED.load(Ordering::Relaxed) {
        let event = Timing {
            label,
            milliseconds: elapsed.as_secs_f64() * 1_000.0,
        };
        eprintln!(
            "PROFILE {}",
            serde_json::to_string(&event).expect("timing event is serializable")
        );
    }
}
