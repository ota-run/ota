#!/bin/sh
# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
#
# DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
#
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
# You may not use this file except in compliance with that License.
# Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
# either express or implied. See the License for the specific language governing permissions
# and limitations under the License.
#
# If you need additional information or have any questions, please email: os@ota.run

set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
consumer_filter=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --consumer)
      [ "$#" -ge 2 ] || {
        printf '%s\n' "error: --consumer requires a value" >&2
        exit 1
      }
      consumer_filter="$2"
      shift 2
      ;;
    *)
      printf '%s\n' "error: unsupported argument: $1" >&2
      exit 1
      ;;
  esac
done

fail() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

trim() {
  printf '%s' "$1" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//'
}

load_committed_changed_files() {
  if [ -n "${OTA_FIRST_PARTY_SYNC_CHANGED_FILES:-}" ]; then
    printf '%s\n' "${OTA_FIRST_PARTY_SYNC_CHANGED_FILES}"
    return
  fi

  if [ -n "${OTA_SKILLS_SYNC_CHANGED_FILES:-}" ]; then
    printf '%s\n' "${OTA_SKILLS_SYNC_CHANGED_FILES}"
    return
  fi

  if [ -n "${OTA_FIRST_PARTY_SYNC_BASE_SHA:-}" ] && git -C "${root}" rev-parse --verify "${OTA_FIRST_PARTY_SYNC_BASE_SHA}^{commit}" >/dev/null 2>&1; then
    git -C "${root}" diff --name-only "${OTA_FIRST_PARTY_SYNC_BASE_SHA}...HEAD"
    return
  fi

  if [ -n "${OTA_SKILLS_SYNC_BASE_SHA:-}" ] && git -C "${root}" rev-parse --verify "${OTA_SKILLS_SYNC_BASE_SHA}^{commit}" >/dev/null 2>&1; then
    git -C "${root}" diff --name-only "${OTA_SKILLS_SYNC_BASE_SHA}...HEAD"
    return
  fi

  if git -C "${root}" rev-parse --verify HEAD^ >/dev/null 2>&1; then
    {
      git -C "${root}" diff --name-only HEAD^..HEAD
      git -C "${root}" diff --name-only --cached
      git -C "${root}" diff --name-only
    } | sort -u
    return
  fi

  return 0
}

load_worktree_changed_files() {
  {
    git -C "${root}" diff --name-only --cached
    git -C "${root}" diff --name-only
  } | sort -u
}

read_status_field() {
  status_file="$1"
  field="$2"
  sed -n "s/^${field}:[[:space:]]*//p" "${status_file}" | head -n1 | tr -d '"'
}

check_consumer() {
  consumer_name="$1"
  repository="$2"
  status_relative_path="$3"
  trigger_regex="$4"
  legacy_commit_field="${5:-}"

  if [ -n "${consumer_filter}" ] && [ "${consumer_filter}" != "${consumer_name}" ]; then
    return
  fi

  status_file="${root}/${status_relative_path}"
  if ! printf '%s\n' "${changed_files}" | grep -Eq "${trigger_regex}"; then
    if [ -n "${consumer_filter}" ] && [ "${consumer_filter}" = "${consumer_name}" ]; then
      printf '%s\n' "first-party-sync: ${consumer_name} has no governed diff"
    fi
    return
  fi

  triggered_any="true"

  if ! printf '%s\n' "${changed_files}" | grep -Eq "^${status_relative_path}\$"; then
    fail "${consumer_name} governed surfaces changed, but ${status_relative_path} was not updated; record ${repository} sync or an explicit waiver"
  fi

  [ -f "${status_file}" ] || fail "missing ${status_relative_path}"

  mode="$(read_status_field "${status_file}" mode)"
  consumer_commit="$(read_status_field "${status_file}" consumer_commit)"
  if [ -z "${consumer_commit}" ] && [ -n "${legacy_commit_field}" ]; then
    consumer_commit="$(read_status_field "${status_file}" "${legacy_commit_field}")"
  fi
  waiver_reason="$(read_status_field "${status_file}" waiver_reason)"
  reviewed_ota_commit="$(read_status_field "${status_file}" reviewed_ota_commit)"

  mode="$(trim "${mode}")"
  consumer_commit="$(trim "${consumer_commit}")"
  waiver_reason="$(trim "${waiver_reason}")"
  reviewed_ota_commit="$(trim "${reviewed_ota_commit}")"

  [ "${mode}" = "synced" ] || [ "${mode}" = "waived" ] || fail "${consumer_name} sync status mode must be \`synced\` or \`waived\`"
  [ -n "${reviewed_ota_commit}" ] || fail "${consumer_name} sync status must record reviewed_ota_commit"

  if [ "${mode}" = "synced" ]; then
    printf '%s' "${consumer_commit}" | grep -Eq '^[0-9a-f]{40}$' \
      || fail "${consumer_name} sync status mode \`synced\` requires a 40-character consumer_commit"
    printf '%s\n' "first-party-sync: ${consumer_name} synced via ${repository}@${consumer_commit}"
    return
  fi

  [ -n "${waiver_reason}" ] || fail "${consumer_name} sync status mode \`waived\` requires waiver_reason"
  printf '%s\n' "first-party-sync: ${consumer_name} waived (${waiver_reason})"
}

check_batch() {
  changed_files="$1"
  triggered_any="false"
  check_consumer \
    "skills" \
    "ota-run/skills" \
    "docs/policy/skills-sync-status.yaml" \
    '^(src/schema\.rs|src/validator\.rs|docs/spec/contract-reference\.md|docs/spec/toolchains-runtimes-tools\.md|docs/spec/execution-topology\.md|docs/spec/local-service-topology\.md|docs/spec/command-reference\.md|examples/)' \
    "skills_commit"
  check_consumer \
    "ota-site" \
    "ota-run/ota-site" \
    "docs/policy/ota-site-sync-status.yaml" \
    '^(src/published_docs_manifest\.rs|docs/spec/published-docs\.md|docs/spec/published-docs/canonical-docs\.json|docs/spec/contract-reference\.md|docs/spec/workspace-reference\.md|docs/spec/command-reference\.md|docs/spec/json-output-reference\.md|docs/spec/execution-topology\.md|docs/spec/local-service-topology\.md|docs/spec/toolchains-runtimes-tools\.md)'
  if [ "${triggered_any}" = "true" ]; then
    batches_triggered="true"
  fi
}

batches_triggered="false"
if [ -n "${OTA_FIRST_PARTY_SYNC_CHANGED_FILES:-}" ] || [ -n "${OTA_SKILLS_SYNC_CHANGED_FILES:-}" ] || [ -n "${OTA_FIRST_PARTY_SYNC_BASE_SHA:-}" ] || [ -n "${OTA_SKILLS_SYNC_BASE_SHA:-}" ]; then
  check_batch "$(load_committed_changed_files || true)"
else
  check_batch "$(load_committed_changed_files || true)"
  check_batch "$(load_worktree_changed_files || true)"
fi

if [ "${batches_triggered}" != "true" ]; then
  printf '%s\n' "first-party-sync: no governed consumer diff detected"
fi
