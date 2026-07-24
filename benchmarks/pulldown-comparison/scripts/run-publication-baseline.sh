#!/usr/bin/env bash
set -euo pipefail

repetitions="${1:-3}"
if [[ "$repetitions" != "3" ]]; then
  echo "publication baseline requires exactly three repetitions" >&2
  exit 1
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd -- "$script_dir/.." && pwd)
lock_dir="$crate_dir/.publication-baseline.lock"

if [[ -n "$(git -C "$crate_dir/../.." status --porcelain)" ]]; then
  echo "publication baseline requires a clean checkout" >&2
  exit 1
fi

mkdir -p "$crate_dir/results"
if ! mkdir "$lock_dir" 2>/dev/null; then
  echo "another publication baseline is already running" >&2
  exit 1
fi
trap 'rmdir "$lock_dir"' EXIT

results_dir=$(mktemp -d "$crate_dir/results/publication-XXXXXXXX")
cd "$crate_dir"

targets=(
  '^profiling/commonmark-5k/commonmark/'
  '^profiling/commonmark-20k/commonmark/'
  '^profiling/commonmark-50k/commonmark/'
  '^profiling/mixed-250k/commonmark/'
  '^profiling/commonmark-5k/default-secure/ferromark$'
  '^profiling/commonmark-20k/default-secure/ferromark$'
  '^profiling/commonmark-50k/default-secure/ferromark$'
  '^profiling/commonmark-50k/minimal-secure/ferromark$'
  '^profiling/commonmark-50k/all-extensions-secure/ferromark$'
)

criterion_groups=(
  'profiling_commonmark-5k_commonmark'
  'profiling_commonmark-20k_commonmark'
  'profiling_commonmark-50k_commonmark'
  'profiling_mixed-250k_commonmark'
  'profiling_commonmark-5k_default-secure'
  'profiling_commonmark-20k_default-secure'
  'profiling_commonmark-50k_default-secure'
  'profiling_commonmark-50k_minimal-secure'
  'profiling_commonmark-50k_all-extensions-secure'
)

criterion_functions=(
  'ferromark pulldown-cmark'
  'ferromark pulldown-cmark'
  'ferromark pulldown-cmark'
  'ferromark pulldown-cmark'
  'ferromark'
  'ferromark'
  'ferromark'
  'ferromark'
  'ferromark'
)

orders=(ferromark-first pulldown-first ferromark-first)

for repetition in $(seq 1 "$repetitions"); do
  run_dir="$results_dir/run-$repetition"
  mkdir -p "$run_dir/criterion"

  for target in "${targets[@]}"; do
    FERROMARK_CPU_MODE=portable FERROMARK_PARITY_ORDER="${orders[$((repetition - 1))]}" \
      RUSTFLAGS='-C target-cpu=generic' \
      cargo bench --locked --bench profiling -- \
      "$target" --sample-size 80 --measurement-time 5 --warm-up-time 3 --noplot
  done

  FERROMARK_CPU_MODE=portable RUSTFLAGS='-C target-cpu=generic' \
    cargo run --locked --release --bin profile_driver -- \
    --parser ferromark \
    --config default-secure \
    --corpus commonmark-50k \
    --iterations 1 \
    --json "$run_dir/environment-probe.json" >/dev/null

  for index in "${!criterion_groups[@]}"; do
    criterion_group=${criterion_groups[$index]}
    target_path="$crate_dir/target/criterion/$criterion_group"
    if [[ ! -d "$target_path" ]]; then
      echo "Criterion result missing for $criterion_group" >&2
      exit 1
    fi
    for criterion_function in ${criterion_functions[$index]}; do
      result_path="$target_path/$criterion_function"
      if [[ ! -d "$result_path" ]]; then
        echo "Criterion result missing for $criterion_group/$criterion_function" >&2
        exit 1
      fi
      mkdir -p "$run_dir/criterion/$criterion_group"
      cp -R "$result_path" "$run_dir/criterion/$criterion_group/$criterion_function"
    done
  done
done

echo "Publication baseline artifacts: $results_dir"
