#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
# shellcheck source=profile_common.sh
source "$script_dir/profile_common.sh"

size="${1:-50k}"
parser="${2:-ferromark}"
sample_secs="${3:-10}"
measure_secs="${4:-60}"
mode="${5:-pgo}"

usage="Usage: $0 [5k|20k|50k] [ferromark|md4c|pulldown-cmark|comrak] [sample_seconds] [measurement_seconds] [pgo|non-pgo]"
case "$size" in
  5k|20k|50k) ;;
  *) echo "$usage" >&2; exit 1 ;;
esac
case "$parser" in
  ferromark|md4c|pulldown-cmark|comrak) ;;
  *) echo "$usage" >&2; exit 1 ;;
esac
case "$mode" in
  pgo|non-pgo) ;;
  *) echo "$usage" >&2; exit 1 ;;
esac
profile_validate_duration "$sample_secs" sample_seconds
profile_validate_duration "$measure_secs" measurement_seconds
profile_validate_sample_budget "$sample_secs" "$measure_secs"
profile_normalize_target_dir
profile_configure_mode "$mode"
PROFILE_MODE="$mode"
export PROFILE_MODE PROFILE_RUSTFLAGS PROFILE_ENCODED_RUSTFLAGS

build_log="$(mktemp "${TMPDIR:-/tmp}/ferromark-profile-commonmark-build.XXXXXX")"
cleanup_files() {
  rm -f "$build_log"
}
trap cleanup_files EXIT

fixture="$repo_root/benches/fixtures/commonmark-${size}.md"
if [[ ! -f "$fixture" ]]; then
  echo "CommonMark fixture not found: $fixture" >&2
  exit 1
fi

if [[ "$parser" == ferromark ]]; then
  bin="$(profile_build_harness "$repo_root" "$build_log")"
  args=("$fixture" commonmark 0 --forever)
else
  bin="$(profile_build_comparison "$repo_root" "$build_log")"
  args=(--bench --measurement-time "$measure_secs" --warm-up-time 5 --sample-size 100 "^commonmark${size}/${parser}$")
fi

echo "Mode: $mode"
if [[ "$mode" == pgo ]]; then
  echo "Using PGO profile: $PGO_PROFDATA"
fi
echo "Parser: $parser"
echo "Using executable: $bin"
if [[ "$parser" == ferromark ]]; then
  echo "Workload: ferromark commonmark preset with a reused output buffer."
else
  echo "Workload: isolated comparison bench with Criterion's fresh benchmark inputs."
fi
echo "Sampling commonmark${size}/${parser} for ${sample_secs}s (budget ${measure_secs}s)..."

profile_run_and_sample_supervised \
  "$bin" "commonmark${size}-${parser}-${mode}" "$sample_secs" "$measure_secs" \
  "${args[@]}"
