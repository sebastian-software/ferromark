//! Timing for tests that pin a shape to a relative cost.
//!
//! Each guard compares two inputs whose expected relative costs are known. A
//! burst of runner load can distort a single wall-clock sample, so each size is
//! measured in five alternating pairs and the median paired ratio is used. For
//! a four-times-larger input, linear work stays near x4 and quadratic work near
//! x16.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// The harness runs a binary's tests in parallel, and a measurement that
/// shares the cores with another one measures the scheduler.
static MEASURING: Mutex<()> = Mutex::new(());

/// Holds off every other measurement in this test binary while it lives.
pub fn measuring() -> MutexGuard<'static, ()> {
    MEASURING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Measures each input in alternating order and returns median durations plus
/// the median of the paired large-to-small ratios. An odd number of samples is
/// required so the ratio comes from one observed pair rather than an average
/// across samples that may have seen different runner load.
pub fn median_of_pairs(
    rounds: usize,
    mut small: impl FnMut() -> Duration,
    mut large: impl FnMut() -> Duration,
) -> TimingEstimate {
    assert!(rounds > 0, "at least one timing pair is required");
    assert_eq!(rounds % 2, 1, "the number of timing pairs must be odd");

    let _measuring = measuring();
    let mut samples = Vec::with_capacity(rounds);
    for round in 0..rounds {
        let pair = if round % 2 == 0 {
            let small = small();
            let large = large();
            (small, large)
        } else {
            let large = large();
            let small = small();
            (small, large)
        };
        samples.push(pair);
    }

    let mut small_durations: Vec<_> = samples.iter().map(|(small, _)| *small).collect();
    let mut large_durations: Vec<_> = samples.iter().map(|(_, large)| *large).collect();
    let mut ratios: Vec<_> = samples
        .iter()
        .map(|(small, large)| ratio(*large, *small))
        .collect();
    small_durations.sort_unstable();
    large_durations.sort_unstable();
    ratios.sort_unstable_by(f64::total_cmp);
    let middle = rounds / 2;

    TimingEstimate {
        small: small_durations[middle],
        large: large_durations[middle],
        ratio: ratios[middle],
    }
}

/// Median timings for each input and the median ratio from paired samples.
#[derive(Debug, Clone, Copy)]
pub struct TimingEstimate {
    pub small: Duration,
    pub large: Duration,
    pub ratio: f64,
}

fn ratio(large: Duration, small: Duration) -> f64 {
    large.as_secs_f64() / small.as_secs_f64().max(1e-9)
}
