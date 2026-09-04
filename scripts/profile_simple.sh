#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
# shellcheck source=profile_common.sh
source "$script_dir/profile_common.sh"

mode="${1:-pgo}"
sample_secs="${2:-10}"
measure_secs="${3:-60}"

case "$mode" in
  pgo|non-pgo) ;;
  *)
    echo "Usage: $0 [pgo|non-pgo] [sample_seconds] [measurement_seconds]" >&2
    exit 1
    ;;
esac
profile_validate_duration "$sample_secs" sample_seconds
profile_validate_duration "$measure_secs" measurement_seconds
profile_validate_sample_budget "$sample_secs" "$measure_secs"
profile_normalize_target_dir
profile_configure_mode "$mode"
PROFILE_MODE="$mode"
export PROFILE_MODE PROFILE_RUSTFLAGS PROFILE_ENCODED_RUSTFLAGS

build_log="$(mktemp "${TMPDIR:-/tmp}/ferromark-profile-simple-build.XXXXXX")"
fixture="$(mktemp "${TMPDIR:-/tmp}/ferromark-profile-simple-fixture.md.XXXXXX")"
cleanup_files() {
  rm -f "$build_log" "$fixture"
}
trap cleanup_files EXIT

printf '%s\n' '# Profile sample' '' 'A simple paragraph with *emphasis* and **strong** text.' >"$fixture"
bin="$(profile_build_harness "$repo_root" "$build_log")"

echo "Mode: $mode"
if [[ "$mode" == pgo ]]; then
  echo "Using PGO profile: $PGO_PROFDATA"
fi
echo "Using profile harness: $bin"
echo "Workload: ferromark commonmark preset with a reused output buffer."
echo "Sampling ferromark simple fixture for ${sample_secs}s (within ${measure_secs}s budget)..."

profile_run_and_sample_supervised \
  "$bin" simple "$sample_secs" "$measure_secs" \
  "$fixture" commonmark 0 --forever
