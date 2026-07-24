#!/usr/bin/env bash
set -euo pipefail

seconds="${1:-5}"
cpu_mode="${2:-portable}"
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

for corpus in commonmark-5k commonmark-20k commonmark-50k mixed-250k; do
  "$script_dir/run-diagnostic.sh" ferromark commonmark "$corpus" "$seconds" "$cpu_mode" release
  "$script_dir/run-diagnostic.sh" pulldown-cmark commonmark "$corpus" "$seconds" "$cpu_mode" release
done

for config in minimal-secure default-secure all-extensions-secure; do
  "$script_dir/run-diagnostic.sh" ferromark "$config" commonmark-50k "$seconds" "$cpu_mode" release
done

for corpus in simple code safe-urls unsafe-urls references tables containers delimiters html unicode-entities; do
  "$script_dir/run-diagnostic.sh" ferromark default-secure "$corpus" "$seconds" "$cpu_mode" release
done

"$script_dir/run-diagnostic.sh" ferromark default-secure commonmark-50k "$seconds" "$cpu_mode" counters
