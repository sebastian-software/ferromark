#!/usr/bin/env bash
set -euo pipefail

parser="${1:-ferromark}"
config="${2:-commonmark}"
corpus="${3:-commonmark-50k}"
seconds="${4:-10}"
cpu_mode="${5:-portable}"
instrumentation="${6:-release}"

case "$cpu_mode" in
  portable)
    rustflags="-C target-cpu=generic"
    ;;
  apple-m1)
    rustflags="-C target-cpu=apple-m1 -C target-feature=+neon"
    ;;
  native)
    rustflags="-C target-cpu=native"
    ;;
  pgo)
    if [[ -z "${PGO_PROFDATA:-}" || ! -f "${PGO_PROFDATA:-}" ]]; then
      echo "pgo mode requires PGO_PROFDATA to name an existing .profdata file" >&2
      exit 1
    fi
    rustflags="-C profile-use=${PGO_PROFDATA} -C llvm-args=-pgo-warn-missing-function"
    ;;
  *)
    echo "cpu mode must be portable, apple-m1, native, or pgo" >&2
    exit 1
    ;;
esac

case "$instrumentation" in
  release)
    ;;
  counters)
    ;;
  *)
    echo "instrumentation must be release or counters" >&2
    exit 1
    ;;
esac

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd -- "$script_dir/.." && pwd)
results_dir="$crate_dir/results"
mkdir -p "$results_dir"

timestamp=$(date -u +%Y%m%dT%H%M%SZ)
result="$results_dir/${timestamp}-${parser}-${config}-${corpus}-${cpu_mode}-${instrumentation}.json"

cd "$crate_dir"
if [[ "$instrumentation" == "counters" ]]; then
  FERROMARK_CPU_MODE="$cpu_mode" RUSTFLAGS="$rustflags" \
    cargo run --locked --release --features profiling --bin profile_driver -- \
    --parser "$parser" \
    --config "$config" \
    --corpus "$corpus" \
    --seconds "$seconds" \
    --json "$result"
else
  FERROMARK_CPU_MODE="$cpu_mode" RUSTFLAGS="$rustflags" \
    cargo run --locked --release --bin profile_driver -- \
    --parser "$parser" \
    --config "$config" \
    --corpus "$corpus" \
    --seconds "$seconds" \
    --json "$result"
fi

echo "Result: $result"
