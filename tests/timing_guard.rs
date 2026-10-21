#[path = "support/timing.rs"]
mod timing;

use std::time::Duration;

fn estimate(small_samples: [u64; 5], large_samples: [u64; 5]) -> timing::TimingEstimate {
    let mut small_samples = small_samples.into_iter();
    let mut large_samples = large_samples.into_iter();
    timing::median_of_pairs(
        5,
        || Duration::from_micros(small_samples.next().expect("five small samples")),
        || Duration::from_micros(large_samples.next().expect("five large samples")),
    )
}

#[test]
fn median_pairs_ignore_timing_outliers_and_keep_quadratic_regressions_visible() {
    let linear = estimate([10; 5], [40, 800, 40, 40, 40]);
    assert!((linear.ratio - 4.0).abs() < f64::EPSILON);
    assert_eq!(linear.small, Duration::from_micros(10));
    assert_eq!(linear.large, Duration::from_micros(40));

    let quadratic = estimate([10; 5], [160; 5]);
    assert!((quadratic.ratio - 16.0).abs() < f64::EPSILON);
    assert!(quadratic.ratio >= 8.0);
}
