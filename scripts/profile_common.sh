#!/usr/bin/env bash

# Shared helpers for the macOS profiling entry points. This file is sourced by
# profile_simple.sh and profile_commonmark50k.sh; it is not a standalone CLI.

profile_validate_duration() {
  local value="$1"
  local label="$2"
  if ! awk -v value="$value" 'BEGIN { exit !(value ~ /^[0-9]+([.][0-9]+)?$/ && value > 0) }'; then
    echo "$label must be a positive number of seconds: $value" >&2
    return 1
  fi
}

profile_validate_sample_budget() {
  local sample_secs="$1"
  local measure_secs="$2"
  if ! awk -v sample="$sample_secs" -v measure="$measure_secs" 'BEGIN { exit !(sample <= measure) }'; then
    echo "sample_seconds must not exceed measurement_seconds ($sample_secs > $measure_secs)" >&2
    return 1
  fi
}

profile_configure_mode() {
  local mode="$1"
  case "$mode" in
    pgo)
      if [[ -z "${PGO_PROFDATA:-}" ]]; then
        echo "PGO mode requires PGO_PROFDATA to point to a .profdata file." >&2
        return 1
      fi
      if [[ ! -f "$PGO_PROFDATA" ]]; then
        echo "PGO profile data not found: $PGO_PROFDATA" >&2
        return 1
      fi
      PGO_PROFDATA="$(cd "$(dirname "$PGO_PROFDATA")" && pwd)/$(basename "$PGO_PROFDATA")"
      PROFILE_RUSTFLAGS=""
      PROFILE_ENCODED_RUSTFLAGS="$(printf '%s\037%s' "-Cprofile-use=${PGO_PROFDATA}" '-Cllvm-args=-pgo-warn-missing-function')"
      ;;
    non-pgo)
      PROFILE_RUSTFLAGS=""
      PROFILE_ENCODED_RUSTFLAGS=""
      ;;
    *)
      echo "Unknown profiling mode: $mode (expected pgo or non-pgo)" >&2
      return 1
      ;;
  esac
}

profile_normalize_target_dir() {
  if [[ -n "${CARGO_TARGET_DIR:-}" && "$CARGO_TARGET_DIR" != /* ]]; then
    local target_parent="$(dirname "$CARGO_TARGET_DIR")"
    mkdir -p "$target_parent"
    target_parent="$(cd "$target_parent" && pwd)"
    CARGO_TARGET_DIR="$target_parent/$(basename "$CARGO_TARGET_DIR")"
    export CARGO_TARGET_DIR
  fi
}

profile_json_executable() {
  local log="$1"
  local target_name="$2"
  local target_kind="$3"
  python3 - "$log" "$target_name" "$target_kind" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as stream:
    for line in stream:
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target", {})
        if (message.get("reason") == "compiler-artifact"
                and target.get("name") == sys.argv[2]
                and sys.argv[3] in target.get("kind", [])
                and message.get("executable")):
            print(message["executable"])
PY
}

profile_build_harness() {
  local repo_root="$1"
  local log="$2"
  local executable

  if ! (
    cd "$repo_root"
    CARGO_ENCODED_RUSTFLAGS="$PROFILE_ENCODED_RUSTFLAGS" RUSTFLAGS= \
      cargo build --locked --manifest-path "$repo_root/Cargo.toml" \
        --profile release-debug --example profile_harness \
        --message-format=json-render-diagnostics
  ) >"$log" 2>&1; then
    echo "Failed to build ferromark profile_harness." >&2
    echo "Manifest: $repo_root/Cargo.toml" >&2
    echo "Mode: ${PROFILE_MODE:-unknown}; CARGO_TARGET_DIR: ${CARGO_TARGET_DIR:-<default>}" >&2
    cat "$log" >&2
    return 1
  fi

  executable="$(profile_json_executable "$log" profile_harness example)"
  if [[ -z "$executable" ]]; then
    echo "Cargo succeeded but did not report the profile_harness executable in JSON output." >&2
    cat "$log" >&2
    return 1
  fi
  if [[ "$executable" != /* ]]; then
    executable="$repo_root/$executable"
  fi
  if [[ ! -x "$executable" ]]; then
    echo "Cargo reported a non-executable profile_harness path: $executable" >&2
    return 1
  fi
  printf '%s\n' "$executable"
}

profile_build_comparison() {
  local repo_root="$1"
  local log="$2"
  local manifest="$repo_root/benchmarks/md4c-comparison/Cargo.toml"
  local executable

  if [[ -z "${MD4C_DIR:-}" ]]; then
    echo "Cross-parser profiling requires MD4C_DIR to point to the pinned md4c checkout." >&2
    return 1
  fi
  if [[ ! -d "$MD4C_DIR" ]]; then
    echo "MD4C_DIR does not exist: $MD4C_DIR" >&2
    return 1
  fi
  local md4c_dir
  md4c_dir="$(cd "$MD4C_DIR" && pwd)"

  if ! (
    cd "$repo_root"
    CARGO_PROFILE_BENCH_STRIP=false CARGO_ENCODED_RUSTFLAGS="$PROFILE_ENCODED_RUSTFLAGS" RUSTFLAGS= MD4C_DIR="$md4c_dir" \
      cargo bench --locked --manifest-path "$manifest" --bench comparison --no-run \
        --message-format=json-render-diagnostics
  ) >"$log" 2>&1; then
    echo "Failed to build the isolated cross-parser comparison bench." >&2
    echo "Manifest: $manifest" >&2
    echo "MD4C_DIR: $MD4C_DIR" >&2
    echo "Mode: ${PROFILE_MODE:-unknown}; CARGO_TARGET_DIR: ${CARGO_TARGET_DIR:-<default>}" >&2
    cat "$log" >&2
    return 1
  fi

  executable="$(profile_json_executable "$log" comparison bench)"
  if [[ -z "$executable" ]]; then
    echo "Cargo succeeded but did not report the comparison executable in JSON output." >&2
    cat "$log" >&2
    return 1
  fi
  if [[ "$executable" != /* ]]; then
    executable="$repo_root/$executable"
  fi
  if [[ ! -x "$executable" ]]; then
    echo "Cargo reported a non-executable comparison path: $executable" >&2
    return 1
  fi
  printf '%s\n' "$executable"
}

# The fourth argument is the caller's measurement budget. Criterion consumes it
# for comparison benches; the ferromark harness runs until sample returns, and
# callers validate that the requested sample interval fits within that budget.
profile_run_and_sample() {
  local executable="$1"
  local label="$2"
  local sample_secs="$3"
  shift 4

  local output_dir="${PROFILE_OUTPUT_DIR:-${TMPDIR:-/tmp}}"
  local bench_output="$output_dir/ferromark-${label}.bench.out"
  local sample_output="$output_dir/ferromark-${label}.sample.txt"
  # These variables intentionally remain child globals: the EXIT trap runs
  # after this function returns, when function locals are out of scope.
  PROFILE_RUN_CHILD_PID=""
  PROFILE_RUN_SAMPLE_PID=""

  "$executable" "$@" >"$bench_output" 2>&1 &
  PROFILE_RUN_CHILD_PID=$!

  cleanup_profile_child() {
    if [[ -n "$PROFILE_RUN_SAMPLE_PID" ]]; then
      kill "$PROFILE_RUN_SAMPLE_PID" 2>/dev/null || true
      wait "$PROFILE_RUN_SAMPLE_PID" 2>/dev/null || true
      PROFILE_RUN_SAMPLE_PID=""
    fi
    if [[ -n "$PROFILE_RUN_CHILD_PID" ]]; then
      kill "$PROFILE_RUN_CHILD_PID" 2>/dev/null || true
      wait "$PROFILE_RUN_CHILD_PID" 2>/dev/null || true
      PROFILE_RUN_CHILD_PID=""
    fi
  }
  trap cleanup_profile_child EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  trap 'exit 129' HUP

  sleep 0.1
  if ! kill -0 "$PROFILE_RUN_CHILD_PID" 2>/dev/null; then
    echo "Profiling child exited before sampling. Output:" >&2
    cat "$bench_output" >&2
    cleanup_profile_child
    return 1
  fi

  if ! command -v sample >/dev/null 2>&1; then
    echo "The macOS 'sample' command is required for CPU profiling." >&2
    echo "Run this script on macOS, or use profile_harness directly under perf/valgrind." >&2
    cleanup_profile_child
    return 1
  fi
  sample "$PROFILE_RUN_CHILD_PID" "$sample_secs" -mayDie -fullPaths -file "$sample_output" &
  PROFILE_RUN_SAMPLE_PID=$!
  if ! wait "$PROFILE_RUN_SAMPLE_PID"; then
    echo "sample failed for PID $PROFILE_RUN_CHILD_PID; benchmark output:" >&2
    cat "$bench_output" >&2
    echo "If this requires elevated privileges, rerun in a terminal with:" >&2
    echo "  sudo sample $PROFILE_RUN_CHILD_PID $sample_secs -mayDie -fullPaths -file $sample_output" >&2
    cleanup_profile_child
    return 1
  fi

  echo "Sample saved to $sample_output"
  trap - EXIT HUP INT TERM
  cleanup_profile_child
}

# Run the scoped helper as a child so its local state and EXIT trap cannot leak
# into the caller. Forward signals from the script process to that child so a
# caller killing the script cannot leave the benchmark behind.
profile_run_and_sample_supervised() {
  local helper_pid
  local status=0

  profile_run_and_sample "$@" &
  helper_pid=$!
  profile_forward_signal() {
    kill "$helper_pid" 2>/dev/null || true
    wait "$helper_pid" 2>/dev/null || true
    exit "$1"
  }
  trap 'profile_forward_signal 130' INT
  trap 'profile_forward_signal 143' TERM
  trap 'profile_forward_signal 129' HUP
  wait "$helper_pid" || status=$?
  trap - INT TERM HUP
  return "$status"
}
