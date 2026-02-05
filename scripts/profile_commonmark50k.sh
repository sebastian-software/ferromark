#!/usr/bin/env bash
set -euo pipefail

# Build bench binary with symbols (avoid stripping)
CARGO_PROFILE_BENCH_STRIP=false cargo bench --bench comparison --no-run >/dev/null

bin=$(ls -t target/release/deps/comparison-* | grep -v '\.dSYM' | head -n 1)
if [[ -z "$bin" ]]; then
  echo "comparison bench binary not found" >&2
  exit 1
fi

echo "Using bench binary: $bin"

echo "Available benches:"
"$bin" --list > /tmp/md-fast-bench.list || true
cat /tmp/md-fast-bench.list

size="${1:-50k}"
case "$size" in
  5k|20k|50k) ;;
  *)
    echo "Usage: $0 [5k|20k|50k]" >&2
    exit 1
    ;;
esac

if rg -q "^commonmark${size}/md-fast$" /tmp/md-fast-bench.list; then
  filter="^commonmark${size}/md-fast$"
else
  echo "No 'commonmark${size}' benchmark found. Aborting." >&2
  exit 1
fi

echo "Starting benchmark (60s) and sampling for 10s..."
out="/tmp/md-fast-commonmark${size}.bench.out"
"$bin" --bench --measurement-time 60 --warm-up-time 5 --sample-size 100 "$filter" > "$out" 2>&1 &
pid=$!

for i in $(seq 1 50); do
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "Benchmark exited early. Output:" >&2
    cat "$out" >&2
    exit 1
  fi
  if rg -q "Benchmarking" "$out"; then
    break
  fi
  sleep 0.1
done

sudo sample "$pid" 10 -mayDie -fullPaths -file "/tmp/md-fast-commonmark${size}.sample.txt"

# Best-effort cleanup
kill "$pid" 2>/dev/null || true

echo "Sample saved to /tmp/md-fast-commonmark${size}.sample.txt"
