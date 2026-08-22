# Fuzzing

The `fuzz` crate is deliberately its own workspace, so it is not a member of
the release workspace or the published `ferromark` package.

Install [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) and run either
target with nightly Rust:

```sh
cargo +nightly fuzz run parse_bytes -- -dict=fuzz/ferromark.dict -max_len=65536
cargo +nightly fuzz run render_lossy -- -dict=fuzz/ferromark.dict -max_len=65536
```

`parse_bytes` sends arbitrary bytes through the public block and inline parsing
APIs. `render_lossy` converts arbitrary bytes with `String::from_utf8_lossy`
and checks that the public rendering APIs agree. Both targets reject inputs over
64 KiB, and the scheduled workflow also applies libFuzzer time and RSS limits.

Crash artifacts and local corpora live under `fuzz/artifacts` and `fuzz/corpus`;
they are intentionally untracked. The small dictionary is tracked because it
helps mutations reach Markdown delimiters without tying the project to a large
or stale corpus.
