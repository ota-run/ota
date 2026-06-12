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
status_file="${root}/docs/policy/skills-sync-status.yaml"

fail() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

trim() {
  printf '%s' "$1" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//'
}

load_changed_files() {
  if [ -n "${OTA_SKILLS_SYNC_CHANGED_FILES:-}" ]; then
    printf '%s\n' "${OTA_SKILLS_SYNC_CHANGED_FILES}"
    return
  fi

  if [ -n "${OTA_SKILLS_SYNC_BASE_SHA:-}" ] && git -C "${root}" rev-parse --verify "${OTA_SKILLS_SYNC_BASE_SHA}^{commit}" >/dev/null 2>&1; then
    git -C "${root}" diff --name-only "${OTA_SKILLS_SYNC_BASE_SHA}...HEAD"
    return
  fi

  if git -C "${root}" rev-parse --verify HEAD^ >/dev/null 2>&1; then
    git -C "${root}" diff --name-only HEAD^..HEAD
    return
  fi
}

changed_files="$(load_changed_files || true)"

triggered="false"
if printf '%s\n' "${changed_files}" | grep -Eq '^(src/schema\.rs|src/validator\.rs|docs/spec/contract-reference\.md|docs/spec/toolchains-runtimes-tools\.md|docs/spec/execution-topology\.md|docs/spec/local-service-topology\.md|docs/spec/command-reference\.md|examples/)'; then
  triggered="true"
fi

if [ "${triggered}" != "true" ]; then
  printf '%s\n' "skills-sync: no contract-surface diff detected"
  exit 0
fi

if ! printf '%s\n' "${changed_files}" | grep -Eq '^docs/policy/skills-sync-status\.yaml$'; then
  fail "contract-surface files changed, but docs/policy/skills-sync-status.yaml was not updated; record ota-run/skills sync or an explicit waiver"
fi

[ -f "${status_file}" ] || fail "missing docs/policy/skills-sync-status.yaml"

mode="$(sed -n 's/^mode:[[:space:]]*//p' "${status_file}" | head -n1 | tr -d '"')"
skills_commit="$(sed -n 's/^skills_commit:[[:space:]]*//p' "${status_file}" | head -n1 | tr -d '"')"
waiver_reason="$(sed -n 's/^waiver_reason:[[:space:]]*//p' "${status_file}" | head -n1 | tr -d '"')"
reviewed_ota_commit="$(sed -n 's/^reviewed_ota_commit:[[:space:]]*//p' "${status_file}" | head -n1 | tr -d '"')"

mode="$(trim "${mode}")"
skills_commit="$(trim "${skills_commit}")"
waiver_reason="$(trim "${waiver_reason}")"
reviewed_ota_commit="$(trim "${reviewed_ota_commit}")"

[ "${mode}" = "synced" ] || [ "${mode}" = "waived" ] || fail "skills-sync status mode must be `synced` or `waived`"
[ -n "${reviewed_ota_commit}" ] || fail "skills-sync status must record reviewed_ota_commit"

if [ "${mode}" = "synced" ]; then
  printf '%s' "${skills_commit}" | grep -Eq '^[0-9a-f]{40}$' \
    || fail "skills-sync status mode `synced` requires a 40-character skills_commit"
  printf '%s\n' "skills-sync: synced via ota-run/skills@${skills_commit}"
  exit 0
fi

[ -n "${waiver_reason}" ] || fail "skills-sync status mode `waived` requires waiver_reason"
printf '%s\n' "skills-sync: waived (${waiver_reason})"
