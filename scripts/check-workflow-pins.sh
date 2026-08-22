#!/usr/bin/env bash
set -euo pipefail

workflow_directory=${1:-.github/workflows}

if [[ ! -d "$workflow_directory" ]]; then
  echo "Workflow directory does not exist or is not a directory: $workflow_directory" >&2
  exit 2
fi

set +e
uses_entries=$(grep --recursive --line-number --extended-regexp \
  --include='*.yml' --include='*.yaml' \
  '^[[:space:]]*(-[[:space:]]*)?uses:[[:space:]]*' "$workflow_directory")
grep_status=$?
set -e

case "$grep_status" in
  0) ;;
  1) exit 0 ;;
  *)
    echo "Failed to scan workflow files for uses: entries." >&2
    exit "$grep_status"
    ;;
esac

invalid_refs=()
while IFS= read -r entry; do
  [[ "$entry" =~ uses:[[:space:]]*(.*)$ ]] || continue
  value=${BASH_REMATCH[1]}
  value=${value%%#*}
  value=$(sed 's/^[[:space:]]*//; s/[[:space:]]*$//' <<< "$value")

  # Local actions and Docker container actions are not GitHub Action refs.
  if [[ "$value" == ./* || "$value" == docker://* ]]; then
    continue
  fi

  if [[ ! "$value" =~ ^[^[:space:]@]+@[0-9a-fA-F]{40}$ ]]; then
    invalid_refs+=("$entry")
  fi
done <<< "$uses_entries"

if ((${#invalid_refs[@]})); then
  echo "Workflow actions must use full 40-character commit SHAs:" >&2
  printf '%s\n' "${invalid_refs[@]}" >&2
  exit 1
fi
