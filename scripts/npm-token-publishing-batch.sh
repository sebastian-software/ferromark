#!/usr/bin/env bash
# Preview or apply npm's mfa=publish policy for packages with configured OIDC publishers.
set -uo pipefail

REGISTRY=https://registry.npmjs.org
owners_file=$(mktemp /tmp/npm-publishing-owners.XXXXXX) || exit 1
tmp_dir=$(mktemp -d /tmp/npm-publishing-batch.XXXXXX) || exit 1
cleanup() {
  rm -f "$owners_file"
  rm -rf "$tmp_dir"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
: >"$owners_file"

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

command -v npm >/dev/null 2>&1 || fail 'npm was not found on PATH'
command -v jq >/dev/null 2>&1 || fail 'jq is required (https://jqlang.org/download/)'
command -v shasum >/dev/null 2>&1 || fail 'shasum is required'

apply=0
plan_path=
state_path=
packages_path=
acknowledged_path=
while (($#)); do
  case "$1" in
    --owner)
      (($# >= 2)) || fail '--owner requires a user or scope'
      [[ "$2" != -* && "$2" != *[$'\t\r\n ']* ]] || fail "invalid owner or scope: $2"
      printf '%s\n' "$2" >>"$owners_file"
      shift 2
      ;;
    --packages-file)
      (($# >= 2)) || fail '--packages-file requires a path'
      packages_path=$2
      shift 2
      ;;
    --plan)
      (($# >= 2)) || fail '--plan requires a path'
      plan_path=$2
      shift 2
      ;;
    --state)
      (($# >= 2)) || fail '--state requires a path'
      state_path=$2
      shift 2
      ;;
    --acknowledged-packages)
      (($# >= 2)) || fail '--acknowledged-packages requires a path'
      acknowledged_path=$2
      shift 2
      ;;
    --apply) apply=1; shift ;;
    --help|-h)
      printf '%s\n' \
        'Usage: npm-token-publishing-batch.sh [--owner USER_OR_SCOPE ...] [--packages-file FILE] [--plan FILE]' \
        '       npm-token-publishing-batch.sh --apply --plan FILE --acknowledged-packages FILE --state FILE' \
        'Preview is read-only. Apply requires explicit acknowledgement of unknown npm UI status.'
      exit 0
      ;;
    *) fail "unknown option: $1" ;;
  esac
done

if ((apply)); then
  [[ -n "$plan_path" && -n "$state_path" && -n "$acknowledged_path" ]] || fail '--apply requires --plan FILE, --acknowledged-packages FILE, and --state FILE'
  [[ -f "$plan_path" ]] || fail "plan file does not exist: $plan_path"
  [[ -f "$acknowledged_path" ]] || fail "acknowledged package list does not exist: $acknowledged_path"
  [[ -z "$packages_path" && ! -s "$owners_file" ]] || fail '--apply reads its package inventory and owner selectors from the plan'
  [[ ! -d "$state_path" ]] || fail "state path is a directory: $state_path"
  if [[ -e "$state_path" ]]; then
    [[ -f "$state_path" ]] || fail "state path is not a file: $state_path"
    IFS= read -r state_header <"$state_path" || fail "state file is empty or malformed: $state_path"
    [[ "$state_header" == $'package\tfingerprint\tresult' ]] || fail "state file has an unsupported header: $state_path"
    awk -F '\t' 'NR == 1 { next } NF == 0 { next } NF != 3 || $1 == "" || length($2) != 64 || $2 ~ /[^0-9a-f]/ || $3 != "COMMAND_OK" { exit 1 }' "$state_path" || fail "state file has a malformed row: $state_path"
  fi
else
  [[ -z "$state_path" ]] || fail '--state is only used with --apply'
  [[ -z "$acknowledged_path" ]] || fail '--acknowledged-packages is only used with --apply'
  [[ -z "$plan_path" || ! -e "$plan_path" ]] || fail "refusing to overwrite plan file: $plan_path"
  [[ -z "$packages_path" || -f "$packages_path" ]] || fail "package inventory does not exist: $packages_path"
fi

umask 077
whoami_output=$(npm --registry="$REGISTRY" whoami) || fail 'npm whoami failed; authenticate with npm login and retry'
account=$(printf '%s\n' "$whoami_output" | tr -d '\r\n')
[[ -n "$account" && "$account" != *[$'\t\n ']* ]] || fail 'npm whoami did not return one account name'

if ((apply)); then
  jq -e '.schema == 1 and .registry == "https://registry.npmjs.org" and (.packages | type == "array") and (.owners | type == "array")' "$plan_path" >/dev/null || fail 'unsupported or malformed plan'
  plan_account=$(jq -r '.account // empty' "$plan_path")
  [[ "$plan_account" == "$account" ]] || fail "plan account '$plan_account' does not match npm account '$account'"
  jq -r '.owners[]' "$plan_path" >"$tmp_dir/plan-owners"
  cat "$tmp_dir/plan-owners" >>"$owners_file"
fi

read_package_map() {
  if [[ -n "$1" ]]; then
    npm --registry="$REGISTRY" access list packages "$1" --json >"$2" || return 1
  else
    npm --registry="$REGISTRY" access list packages --json >"$2" || return 1
  fi
  jq -e 'type == "object" and all(to_entries[]; (.value | type) == "string")' "$2" >/dev/null
}

load_package_map() {
  local maps="$tmp_dir/maps.jsonl"
  : >"$maps"
  read_package_map '' "$tmp_dir/account.json" || return 1
  cat "$tmp_dir/account.json" >>"$maps"
  printf '\n' >>"$maps"
  while IFS= read -r owner; do
    [[ -n "$owner" ]] || continue
    read_package_map "$owner" "$tmp_dir/owner.json" || return 1
    cat "$tmp_dir/owner.json" >>"$maps"
    printf '\n' >>"$maps"
  done <"$owners_file"
  package_map=$(jq -s 'reduce .[] as $map ({}; reduce ($map | to_entries[]) as $e (. ; .[$e.key] = if .[$e.key] == "read-write" or $e.value == "read-write" then "read-write" else $e.value end))' "$maps") || return 1
}

load_package_map || fail 'could not read or combine current package permissions'

valid_package_name() {
  [[ "$1" =~ ^(@[a-z0-9][a-z0-9._-]*/)?[a-z0-9][a-z0-9._-]*$ ]]
}

if ((apply)); then
  : >"$tmp_dir/selected"
  while IFS= read -r package || [[ -n "$package" ]]; do
    package=$(printf '%s' "$package" | tr -d '\r')
    [[ -z "$package" || "$package" == \#* ]] && continue
    valid_package_name "$package" || fail "invalid package name in acknowledged list: $package"
    printf '%s\n' "$package" >>"$tmp_dir/selected"
  done <"$acknowledged_path"
else
  : >"$tmp_dir/selected"
  if [[ -n "$packages_path" ]]; then
    while IFS= read -r package || [[ -n "$package" ]]; do
      package=$(printf '%s' "$package" | tr -d '\r')
      [[ -z "$package" || "$package" == \#* ]] && continue
      valid_package_name "$package" || fail "invalid package name: $package"
      printf '%s\n' "$package" >>"$tmp_dir/selected"
    done <"$packages_path"
  else
    jq -r 'to_entries[] | select(.value == "read-write") | .key' <<<"$package_map" >>"$tmp_dir/selected"
  fi
fi
sort -u "$tmp_dir/selected" >"$tmp_dir/selected.sorted"
mv "$tmp_dir/selected.sorted" "$tmp_dir/selected"
if [[ ! -s "$tmp_dir/selected" ]]; then
  if ((apply)); then
    printf 'No packages were acknowledged for update; no changes made.\n'
    exit 0
  fi
  fail 'selected inventory is empty'
fi

read_trust() {
  npm --registry="$REGISTRY" trust list "$1" --json >"$2" || return 1
  if [[ ! -s "$2" ]]; then
    printf '[]\n' >"$2"
    return 0
  fi
  # npm emits one JSON object for each relationship; jq -s accepts that JSON stream.
  jq -s -e 'if all(.[]; type == "object") then . else error("invalid trust JSON") end' "$2" >"$tmp_dir/trust-array.json" || return 2
  mv "$tmp_dir/trust-array.json" "$2"
}

fingerprint() {
  jq -S -c 'sort_by(tojson)' <<<"$1" | shasum -a 256 | awk '{print $1}'
}

permissions() {
  jq -r '[.[] | (.permissions // [])[]? | select(. == "createPackage" or . == "createStagedPackage")] | unique | map(if . == "createPackage" then "direct" else "staged" end) | join(",")' <<<"$1"
}

if ((apply)); then
  # Preflight every eligible package before the first mutation.
  : >"$tmp_dir/preflight"
  while IFS= read -r package; do
    valid_package_name "$package" || fail "invalid package name in plan: $package"
    planned=$(jq -c --arg name "$package" '[.packages[] | select(.name == $name)] | if length == 1 then .[0] else empty end' "$plan_path")
    [[ -n "$planned" ]] || fail "package is missing or duplicated in plan: $package"
    access=$(jq -r '.access // ""' <<<"$planned")
    old_permissions=$(jq -r '.permission_summary // ""' <<<"$planned")
    if [[ "$access" != read-write || -z "$old_permissions" ]]; then
      printf 'SKIP %s: no read-write access or publish-capable Trusted Publisher\n' "$package"
      continue
    fi
    jq -e '.trust_records | type == "array"' <<<"$planned" >/dev/null || fail "malformed Trusted Publisher records in plan for $package"
    planned_records=$(jq -c '.trust_records' <<<"$planned")
    old_fingerprint=$(jq -r '.trust_fingerprint' <<<"$planned")
    old_fingerprint_check=$(fingerprint "$planned_records") || fail "could not fingerprint planned publishers for $package"
    planned_permissions=$(permissions "$planned_records") || fail "could not read planned publisher permissions for $package"
    if [[ "$old_fingerprint_check" != "$old_fingerprint" || "$planned_permissions" != "$old_permissions" ]]; then
      printf 'SKIP %s: plan records, permissions, and fingerprint do not match; create a new plan\n' "$package"
      continue
    fi
    current_access=$(jq -r --arg name "$package" '.[$name] // ""' <<<"$package_map")
    if [[ "$current_access" != read-write ]]; then
      printf 'SKIP %s: current npm account no longer has read-write access\n' "$package"
      continue
    fi
    read_trust "$package" "$tmp_dir/current-trust.json" || fail "could not revalidate Trusted Publisher records for $package; no changes made"
    current_records=$(cat "$tmp_dir/current-trust.json")
    current_fingerprint=$(fingerprint "$current_records") || fail "could not fingerprint Trusted Publisher records for $package"
    if [[ "$current_fingerprint" != "$old_fingerprint" ]]; then
      printf 'SKIP %s: Trusted Publisher records changed after review; create and review a new plan\n' "$package"
      continue
    fi
    jq -cn --arg package "$package" --arg fingerprint "$current_fingerprint" --arg permissions "$old_permissions" \
      '{package:$package,fingerprint:$fingerprint,permissions:$permissions}' >>"$tmp_dir/preflight"
  done <"$tmp_dir/selected"

  [[ -s "$tmp_dir/preflight" ]] || { printf 'No acknowledged, current packages are ready to update.\n'; exit 0; }
  state_dir=$(dirname "$state_path")
  [[ -d "$state_dir" ]] || fail "state directory does not exist: $state_dir"
  if [[ -e "$state_path" && ! -f "$state_path" ]]; then
    fail "state path is not a file: $state_path"
  fi
  printf 'Ready to set npm mfa=publish for:\n'
  jq -r '[.package, .permissions] | @tsv' "$tmp_dir/preflight" | while IFS=$'\t' read -r package summary; do
    printf '  %s (%s OIDC publish)\n' "$package" "$summary"
  done
  printf 'npm CLI cannot report UI Valid status, publisher expiry, or current Publishing access. Type APPLY UNKNOWN to continue: '
  IFS= read -r confirmation || fail 'confirmation input ended; no changes made'
  [[ "$confirmation" == 'APPLY UNKNOWN' ]] || fail 'confirmation did not match APPLY UNKNOWN; no changes made'
  if [[ ! -e "$state_path" ]]; then
    printf 'package\tfingerprint\tresult\n' >"$state_path" || fail "could not create state file: $state_path"
  fi

  while IFS=$'\t' read -r package expected_fingerprint summary; do
    [[ -n "$package" ]] || continue
    if awk -F '\t' -v package="$package" -v fp="$expected_fingerprint" '$1 == package && $2 == fp && $3 == "COMMAND_OK" { found=1 } END { exit !found }' "$state_path"; then
      printf 'SKIP %s: command already succeeded for this fingerprint; verify npm settings\n' "$package"
      continue
    fi
    load_package_map || fail "could not refresh package permissions before $package; stopped before this write"
    if [[ $(jq -r --arg name "$package" '.[$name] // ""' <<<"$package_map") != read-write ]]; then
      fail "read-write access changed for $package; stopped before this write"
    fi
    read_trust "$package" "$tmp_dir/current-trust.json" || fail "could not refresh Trusted Publisher records for $package; stopped before this write"
    current_records=$(cat "$tmp_dir/current-trust.json")
    current_fingerprint=$(fingerprint "$current_records") || fail "could not fingerprint current publishers for $package"
    current_permissions=$(permissions "$current_records") || fail "could not read current publisher permissions for $package"
    if [[ "$current_fingerprint" != "$expected_fingerprint" || "$current_permissions" != "$summary" ]]; then
      fail "Trusted Publisher records or permissions changed for $package; stopped before this write"
    fi
    printf 'Applying mfa=publish to %s. npm may prompt for MFA; enter it only at the npm prompt.\n' "$package"
    if npm --registry="$REGISTRY" access set mfa=publish "$package"; then
      printf '%s\t%s\tCOMMAND_OK\n' "$package" "$expected_fingerprint" >>"$state_path" || fail "command succeeded for $package but state could not be saved; verify npm settings"
      printf 'COMMAND_OK %s: verify saved policy in npm settings\n' "$package"
    else
      fail "npm command failed for $package; stopped here. Check npm settings and state before resuming"
    fi
  done < <(jq -r '[.package, .fingerprint, .permissions] | @tsv' "$tmp_dir/preflight")
  exit 0
fi

: >"$tmp_dir/plan-packages"
while IFS= read -r package; do
  valid_package_name "$package" || fail "invalid package name: $package"
  access=$(jq -r --arg name "$package" '.[$name] // "not-writable"' <<<"$package_map")
  trust_records='[]'
  trust_fingerprint=
  permission_summary=
  reason=
  if [[ "$access" == read-write ]]; then
    read_trust "$package" "$tmp_dir/trust.json" || fail "could not inspect Trusted Publisher records for $package"
    trust_records=$(cat "$tmp_dir/trust.json")
    trust_fingerprint=$(fingerprint "$trust_records") || fail "could not fingerprint Trusted Publisher records for $package"
    permission_summary=$(permissions "$trust_records") || fail "could not read publisher permissions for $package"
    [[ -n "$permission_summary" ]] || reason='no Trusted Publisher permits direct or staged publish'
  else
    reason='npm account does not have read-write access'
  fi
  if [[ -z "$reason" ]]; then
    printf 'REVIEW %s: %s OIDC publish; fingerprint %s\n' "$package" "$permission_summary" "$trust_fingerprint"
    jq -c '.[]' <<<"$trust_records" | while IFS= read -r record; do printf '  %s\n' "$record"; done
  else
    printf 'SKIP %s: %s\n' "$package" "$reason"
  fi
  jq -cn --arg name "$package" --arg access "$access" --arg fp "$trust_fingerprint" \
    --arg permissions "$permission_summary" --arg reason "$reason" --argjson trust "$trust_records" \
    '{name:$name,access:$access,trust_records:$trust,trust_fingerprint:$fp,permission_summary:$permissions,reason:$reason}' >>"$tmp_dir/plan-packages"
done <"$tmp_dir/selected"

if [[ -n "$plan_path" ]]; then
  plan_dir=$(dirname "$plan_path")
  [[ -d "$plan_dir" ]] || fail "plan directory does not exist: $plan_dir"
  jq -s -n --arg account "$account" --arg registry "$REGISTRY" --slurpfile packages "$tmp_dir/plan-packages" --rawfile owner_lines "$owners_file" \
    '{schema:1,account:$account,registry:$registry,owners:($owner_lines | split("\n") | map(select(length > 0))),packages:$packages}' >"$tmp_dir/plan.json" || fail 'could not build plan'
  (set -C; cat "$tmp_dir/plan.json" >"$plan_path") 2>/dev/null || fail "could not create plan: $plan_path"
  printf 'Review plan written to %s. Use the interactive TUI to review packages and apply safely.\n' "$plan_path"
fi
