#!/usr/bin/env bash
set -euo pipefail

mutable_refs=$(rg --pcre2 --line-number \
  'uses:\s+(?!\./)[^@\s]+@(?![0-9a-f]{40}(?:\s|$))' \
  .github/workflows || true)

if [[ -n "$mutable_refs" ]]; then
  echo "Workflow actions must use full 40-character commit SHAs:" >&2
  echo "$mutable_refs" >&2
  exit 1
fi
