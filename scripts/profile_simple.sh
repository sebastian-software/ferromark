#!/usr/bin/env bash
set -euo pipefail

# Build bench binary if needed
cargo bench --bench comparison --no-run >/dev/null

bin=$(ls -1 target/release/deps/comparison-* | grep -v '\.dSYM' | head -n 1)
if [[ -z "$bin" ]]; then
  echo "comparison bench binary not found" >&2
  exit 1
fi

echo "Using bench binary: $bin"

echo "Available benches:"
"$bin" --list > /tmp/md-fast-bench.list || true
cat /tmp/md-fast-bench.list

if ! rg -q '^simple/' /tmp/md-fast-bench.list; then
  echo "No 'simple' benchmark found. Aborting." >&2
  exit 1
fi

echo "Starting benchmark (60s) and sampling for 10s..."
"$bin" --measurement-time 60 --warm-up-time 5 --sample-size 100 "^simple/" &
pid=$!

# Give it a moment to start
sleep 0.5

sudo sample "$pid" 10 -file /tmp/md-fast-simple.sample.txt

# Best-effort cleanup
kill "$pid" 2>/dev/null || true

echo "Sample saved to /tmp/md-fast-simple.sample.txt"
