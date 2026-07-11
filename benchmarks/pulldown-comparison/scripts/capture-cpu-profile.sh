#!/usr/bin/env bash
set -euo pipefail

tool="${1:-sample}"
config="${2:-extended-secure}"
corpus="${3:-commonmark-50k}"
seconds="${4:-30}"
cpu_mode="${5:-portable}"

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
  *)
    echo "cpu mode must be portable, apple-m1, or native" >&2
    exit 1
    ;;
esac

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd -- "$script_dir/.." && pwd)
results_dir="$crate_dir/results"
mkdir -p "$results_dir"
cd "$crate_dir"

RUSTFLAGS="$rustflags" CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=false \
  cargo build --locked --release --bin profile_driver

binary="$crate_dir/target/release/profile_driver"
timestamp=$(date -u +%Y%m%dT%H%M%SZ)
base="$results_dir/${timestamp}-ferromark-${config}-${corpus}-${cpu_mode}"
arguments=(
  --parser ferromark
  --config "$config"
  --corpus "$corpus"
  --seconds "$seconds"
  --json "$base.json"
)

case "$tool" in
  sample)
    "$binary" "${arguments[@]}" > "$base.stdout" &
    pid=$!
    sleep 1
    sample "$pid" "$((seconds - 1))" -mayDie -fullPaths -file "$base.sample.txt"
    wait "$pid"
    ;;
  samply)
    samply record --save-only --output "$base.samply.json.gz" -- \
      "$binary" "${arguments[@]}"
    ;;
  xctrace)
    xctrace record --template "Time Profiler" --output "$base.trace" --launch -- \
      "$binary" "${arguments[@]}"
    ;;
  *)
    echo "tool must be sample, samply, or xctrace" >&2
    exit 1
    ;;
esac

echo "Profile base: $base"
