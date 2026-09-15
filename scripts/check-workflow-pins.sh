#!/usr/bin/env bash
set -euo pipefail

workflow_directory=${1:-.github/workflows}
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

if [[ ! -d "$workflow_directory" ]]; then
  echo "Workflow directory does not exist or is not a directory: $workflow_directory" >&2
  exit 2
fi

node "$script_directory/check-workflow-pins.mjs" "$workflow_directory"
