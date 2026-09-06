use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ferro_byte_search::ByteSet;
use std::hint::black_box;
use std::time::Duration;

// Compare the exact same implementation across the external crate boundary
// and the embedded-module arrangement used by Ferromark's published package.
#[path = "../../../src/byte_search.rs"]
mod embedded;

fn bench_set<const N: usize>(c: &mut Criterion, name: &str, bytes: &[u8; N]) {
    let set = ByteSet::new(bytes);
    let embedded_set = embedded::ByteSet::new(bytes);
    for len in [0, 8, 15, 16, 17, 64, 128, 1024, 65536] {
        let mut group = c.benchmark_group(format!("{name}/{len}"));
        group.throughput(Throughput::Bytes(len as u64));
        for pattern in ["absent", "first", "last", "dense"] {
            // Offset the slice to also exercise unaligned loads.
            let mut storage = vec![b'a'; len + 1];
            let input = &mut storage[1..];
            if len > 0 {
                match pattern {
                    "first" => input[0] = bytes[0],
                    "last" => input[len - 1] = bytes[0],
                    "dense" => input.fill(bytes[0]),
                    _ => {}
                }
            }
            let expected = input.iter().position(|b| bytes.contains(b));
            assert_eq!(set.find(input), expected);
            assert_eq!(set.contains_any(input), expected.is_some());
            assert_eq!(embedded_set.find(input), expected);
            assert_eq!(embedded_set.contains_any(input), expected.is_some());
            group.bench_with_input(BenchmarkId::new("find", pattern), &input, |b, input| {
                b.iter(|| black_box(set.find(black_box(input))))
            });
            group.bench_with_input(BenchmarkId::new("contains", pattern), &input, |b, input| {
                b.iter(|| black_box(set.contains_any(black_box(input))))
            });
            group.bench_with_input(
                BenchmarkId::new("embedded_find", pattern),
                &input,
                |b, input| b.iter(|| black_box(embedded_set.find(black_box(input)))),
            );
        }
        group.finish();
    }
}

fn search(c: &mut Criterion) {
    bench_set(c, "four", b"&<>\"");
    bench_set(c, "twelve", b"*_`[]<\\\n~$=^");
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(300));
    targets = search
}
criterion_main!(benches);
