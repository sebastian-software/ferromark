use std::{
    hint::black_box,
    time::{Duration, Instant},
};
#[macro_export]
macro_rules! profile_span_detail {
    ($($t:tt)*) => {};
}
#[path = "@FERROMARK@/src/byte_search.rs"]
#[allow(dead_code)]
mod ferro_scan;
#[path = "../../ox-current/crates/ox_content_parser/src/parser/inline/scan.rs"]
mod ox_scan;
const SET: ferro_scan::ByteSet<11> = ferro_scan::ByteSet::new(b"*_`[!~\\<\n\r&");
fn ferro(s: &[u8]) -> usize {
    SET.find(s).unwrap_or(s.len())
}
fn ox(s: &[u8]) -> usize {
    ox_scan::next_inline_special(s, 0)
}
fn main() {
    // Verify identical classification for every byte at every offset, including tails.
    for len in 1..65 {
        for pos in 0..len {
            for b in 0..=255 {
                let mut s = vec![b'a'; len];
                s[pos] = b;
                assert_eq!(ferro(&s), ox(&s));
            }
        }
    }
    for len in [8, 16, 32, 128, 1024, 65536] {
        for shape in ["absent", "last", "first"] {
            let mut s = vec![b'a'; len];
            if shape == "last" {
                s[len - 1] = b'*';
            }
            if shape == "first" {
                s[0] = b'*';
            }
            for (name, f) in [("ferro", ferro as fn(&[u8]) -> usize), ("ox", ox)] {
                let mut windows = vec![];
                for _ in 0..11 {
                    let start = Instant::now();
                    let mut n = 0;
                    while start.elapsed() < Duration::from_millis(20) {
                        for _ in 0..256 {
                            black_box(f(black_box(&s)));
                            n += 1;
                        }
                    }
                    windows.push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
                }
                println!(
                    "{}",
                    serde_json::json!({"bytes":len,"shape":shape,"engine":name,"windows_ns":windows})
                );
            }
        }
    }
}
