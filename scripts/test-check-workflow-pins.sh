#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
checker="$repository_root/scripts/check-workflow-pins.sh"
fixtures="$repository_root/scripts/test-fixtures/workflow-pins"
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/ferromark-workflow-pins.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT

fail() {
  echo "test-check-workflow-pins: $*" >&2
  exit 1
}

run_fixture() {
  local fixture=$1
  local expected_status=$2
  local expected_output=${3:-}
  local case_directory="$temporary_directory/$fixture"
  local output="$case_directory/output"
  local actual_status

  mkdir -p "$case_directory"
  cp "$fixtures/$fixture" "$case_directory/workflow.yml"

  if "$checker" "$case_directory" >"$output" 2>&1; then
    actual_status=0
  else
    actual_status=$?
  fi

  [[ "$actual_status" == "$expected_status" ]] || \
    fail "$fixture exited $actual_status; expected $expected_status: $(<"$output")"

  if [[ -n "$expected_output" ]]; then
    grep --fixed-strings --quiet "$expected_output" "$output" || \
      fail "$fixture did not report $expected_output: $(<"$output")"
  fi
}

run_fixture full-sha.yml 0
run_fixture local-and-docker.yml 0
run_fixture mutable-ref.yml 1 'actions/checkout@v5'
run_fixture missing-ref.yml 1 'actions/checkout'
run_fixture no-matches.yml 0

scanner_directory="$temporary_directory/scanner-failure"
fake_bin="$temporary_directory/fake-bin"
mkdir -p "$scanner_directory" "$fake_bin"
cp "$fixtures/full-sha.yml" "$scanner_directory/workflow.yml"
cp "$fixtures/failing-grep" "$fake_bin/grep"
chmod +x "$fake_bin/grep"

if PATH="$fake_bin:$PATH" "$checker" "$scanner_directory" >"$temporary_directory/scanner-output" 2>&1; then
  fail "scanner failure unexpectedly succeeded"
else
  scanner_status=$?
fi

[[ "$scanner_status" == 2 ]] || \
  fail "scanner failure exited $scanner_status; expected 2: $(<"$temporary_directory/scanner-output")"
grep --fixed-strings --quiet 'Failed to scan workflow files' "$temporary_directory/scanner-output" || \
  fail "scanner failure did not report the scan error: $(<"$temporary_directory/scanner-output")"

echo "workflow pin checks passed"
