#!/usr/bin/env bash
set -euo pipefail

workflow_directory=${1:-.github/workflows}
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

if [[ ! -d "$workflow_directory" ]]; then
  echo "Workflow directory does not exist or is not a directory: $workflow_directory" >&2
  exit 2
fi

ruby "$script_directory/check-workflow-pins.rb" "$workflow_directory"
