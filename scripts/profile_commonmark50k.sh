#!/usr/bin/env bash
set -euo pipefail

size="${1:-50k}"
parser="${2:-ferromark}"
sample_secs="${3:-10}"
measure_secs="${4:-60}"
mode="${5:-pgo}"

case "$mode" in
  pgo|non-pgo) ;;
  *)
    echo "Usage: $0 [5k|20k|50k] [ferromark|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds] [pgo|non-pgo]" >&2
    exit 1
    ;;
esac

case "$size" in
  5k|20k|50k) ;;
  *)
    echo "Usage: $0 [5k|20k|50k] [ferromark|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds] [pgo|non-pgo]" >&2
    exit 1
    ;;
esac

case "$parser" in
  ferromark|md4c|pulldown-cmark|comrak) ;;
  *)
    echo "Usage: $0 [5k|20k|50k] [ferromark|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds] [pgo|non-pgo]" >&2
    exit 1
    ;;
esac

if [[ "$mode" == "pgo" ]]; then
  if [[ -z "${PGO_PROFDATA:-}" ]]; then
    echo "PGO mode requires PGO_PROFDATA to point to a .profdata file." >&2
    exit 1
  fi
  if [[ ! -f "$PGO_PROFDATA" ]]; then
    echo "PGO profile data not found: $PGO_PROFDATA" >&2
    exit 1
  fi
  rustflags="-Cprofile-use=${PGO_PROFDATA} -Cllvm-args=-pgo-warn-missing-function"
else
  rustflags=""
fi

# Build bench binary with symbols (avoid stripping) and parse exact binary path.
build_output=$(
  CARGO_PROFILE_BENCH_STRIP=false RUSTFLAGS="$rustflags" \
    cargo bench --bench comparison --no-run 2>&1
)
bin=$(printf '%s\n' "$build_output" | sed -nE 's|.*Executable benches/comparison\.rs \((target/release/deps/comparison-[^)]+)\).*|\1|p' | tail -n 1)
if [[ -z "$bin" || ! -x "$bin" ]]; then
  echo "Could not resolve comparison bench binary from cargo output." >&2
  printf '%s\n' "$build_output" >&2
  exit 1
fi

echo "Mode: $mode"
if [[ "$mode" == "pgo" ]]; then
  echo "Using PGO profile: $PGO_PROFDATA"
fi
echo "Using bench binary: $bin"

echo "Available benches:"
"$bin" --list > /tmp/ferromark-bench.list || true
cat /tmp/ferromark-bench.list

if rg -q "^commonmark${size}/${parser}:" /tmp/ferromark-bench.list; then
  filter="^commonmark${size}/${parser}$"
else
  echo "No 'commonmark${size}/${parser}' benchmark found. Aborting." >&2
  exit 1
fi

echo "Starting benchmark (${measure_secs}s) and sampling for ${sample_secs}s..."
out="/tmp/ferromark-commonmark${size}-${parser}-${mode}.bench.out"
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

sample_out="/tmp/ferromark-commonmark${size}-${parser}-${mode}.sample.txt"
if ! sample "$pid" "$sample_secs" -mayDie -fullPaths -file "$sample_out"; then
  echo "sample failed. If this requires elevated privileges, rerun in a terminal with sudo:" >&2
  echo "  sudo sample $pid $sample_secs -mayDie -fullPaths -file $sample_out" >&2
  exit 1
fi

# Best-effort cleanup
kill "$pid" 2>/dev/null || true

echo "Sample saved to $sample_out"
