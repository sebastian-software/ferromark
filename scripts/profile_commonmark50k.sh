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
parser="${2:-md-fast}"
sample_secs="${3:-10}"
measure_secs="${4:-60}"

case "$size" in
  5k|20k|50k) ;;
  *)
    echo "Usage: $0 [5k|20k|50k] [md-fast|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds]" >&2
    exit 1
    ;;
esac

case "$parser" in
  md-fast|md4c|pulldown-cmark|comrak) ;;
  *)
    echo "Usage: $0 [5k|20k|50k] [md-fast|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds]" >&2
    exit 1
    ;;
esac

if rg -q "^commonmark${size}/${parser}:" /tmp/md-fast-bench.list; then
  filter="^commonmark${size}/${parser}$"
else
  echo "No 'commonmark${size}/${parser}' benchmark found. Aborting." >&2
  exit 1
fi

echo "Starting benchmark (${measure_secs}s) and sampling for ${sample_secs}s..."
out="/tmp/md-fast-commonmark${size}-${parser}.bench.out"
"$bin" --bench --measurement-time "$measure_secs" --warm-up-time 5 --sample-size 100 "$filter" > "$out" 2>&1 &
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

if ! sample "$pid" "$sample_secs" -mayDie -fullPaths -file "/tmp/md-fast-commonmark${size}-${parser}.sample.txt"; then
  echo "sample failed. If this requires elevated privileges, rerun in a terminal with sudo:" >&2
  echo "  sudo sample $pid $sample_secs -mayDie -fullPaths -file /tmp/md-fast-commonmark${size}-${parser}.sample.txt" >&2
  exit 1
fi

# Best-effort cleanup
kill "$pid" 2>/dev/null || true

echo "Sample saved to /tmp/md-fast-commonmark${size}-${parser}.sample.txt"
