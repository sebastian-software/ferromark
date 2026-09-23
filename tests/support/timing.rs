//! Timing for the tests that pin a shape to linear cost.
//!
//! Each of those compares the time of a small input with the time of a
//! larger one. Timed one after the other, a burst of load from a neighboring
//! test, or from another job on a shared runner, lands on one side of the
//! ratio only; that pushed linear shapes past x8 on macOS runners. The two
//! inputs are therefore timed in alternation, with the order swapped every
//! round as `emphasis_pairing.rs` does, and only one measurement runs at a
//! time within a test binary.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// The harness runs a binary's tests in parallel, and a measurement that
/// shares the cores with another one measures the scheduler.
static MEASURING: Mutex<()> = Mutex::new(());

/// Holds off every other measurement in this test binary while it lives.
pub fn measuring() -> MutexGuard<'static, ()> {
    MEASURING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The best of `rounds` timings of each input, taken in alternation, so a
/// scheduling stall on a busy runner has to hit every repetition of a side
/// to move the ratio.
///
/// While `large / small` is still at or above `bound`, up to `rounds` more
/// rounds follow. Each one can only lower the two minimums toward the true
/// cost: a linear shape settles near its own ratio, and a quadratic one
/// stays past the bound however long it is timed.
pub fn best_of_pairs(
    rounds: usize,
    bound: f64,
    mut small: impl FnMut() -> Duration,
    mut large: impl FnMut() -> Duration,
) -> (Duration, Duration) {
    let _measuring = measuring();
    let (mut small_best, mut large_best) = (Duration::MAX, Duration::MAX);
    for round in 0..2 * rounds {
        if round >= rounds && ratio(large_best, small_best) < bound {
            break;
        }
        if round % 2 == 0 {
            small_best = small_best.min(small());
            large_best = large_best.min(large());
        } else {
            large_best = large_best.min(large());
            small_best = small_best.min(small());
        }
    }
    (small_best, large_best)
}

fn ratio(large: Duration, small: Duration) -> f64 {
    large.as_secs_f64() / small.as_secs_f64().max(1e-9)
}
