#!/bin/sh
#
#                █████
#               ░░███
#       ██████  ███████    ██████
#      ███░░███░░░███░    ░░░░░███
#     ░███ ░███  ░███      ███████
#     ░███ ░███  ░███ ███ ███░░███
#     ░░██████   ░░█████ ░░████████
#      ░░░░░░     ░░░░░   ░░░░░░░░
#
#   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
#
#   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
#
#   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
#   You may not use this file except in compliance with that License.
#   Unless required by applicable law or agreed to in writing, software distributed under the
#   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
#   either express or implied. See the License for the specific language governing permissions
#   and limitations under the License.
#
#   If you need additional information or have any questions, please email: os@ota.run

set -eu

format="plain"
mode=""
title=""
input=""

usage() {
  cat <<'EOF'
usage: emit-ota-findings.sh --mode doctor|workspace-doctor [--format plain|github] [--title TEXT] --input FILE

Reads ota doctor JSON and emits portable finding lines. Use --format github for GitHub Actions
annotations, or --format plain for CI-agnostic log output.
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    --format)
      format="${2:-}"
      shift 2
      ;;
    --title)
      title="${2:-}"
      shift 2
      ;;
    --input)
      input="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'error: unexpected argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$mode" ]; then
  printf 'error: --mode is required\n' >&2
  usage >&2
  exit 2
fi

if [ -z "$input" ]; then
  printf 'error: --input is required\n' >&2
  usage >&2
  exit 2
fi

if [ -z "$title" ]; then
  case "$mode" in
    doctor) title='ota doctor' ;;
    workspace-doctor) title='ota workspace doctor' ;;
    *)
      printf 'error: unsupported mode: %s\n' "$mode" >&2
      exit 2
      ;;
  esac
fi

emit_line() {
  printf '%s\n' "$1"
}

emit_finding() {
  severity="$1"
  heading="$2"
  body="$3"
  next="$4"

  case "$format" in
    github)
      if [ "$severity" = "error" ]; then
        emit_line "::error title=${heading}::${body} | ${next}"
      else
        emit_line "::warning title=${heading}::${body} | ${next}"
      fi
      ;;
    plain)
      if [ "$severity" = "error" ]; then
        emit_line "ERROR: ${heading}: ${body} | ${next}"
      else
        emit_line "WARNING: ${heading}: ${body} | ${next}"
      fi
      ;;
    *)
      printf 'error: unsupported format: %s\n' "$format" >&2
      exit 2
      ;;
  esac
}

emit_primary_blocker() {
  blocker_title="$1"
  blocker_body="$2"
  blocker_next="$3"

  case "$format" in
    github)
      emit_line "::notice title=${blocker_title}::${blocker_body} | ${blocker_next}"
      ;;
    plain)
      emit_line "NOTICE: ${blocker_title}: ${blocker_body} | ${blocker_next}"
      ;;
    *)
      printf 'error: unsupported format: %s\n' "$format" >&2
      exit 2
      ;;
  esac
}

case "$mode" in
  doctor)
    primary_blocker="$(jq -r '.summary.primary_blocker? | select(. != null) | [.summary, .next] | @tsv' "$input" | head -n 1 || true)"
    if [ -n "$primary_blocker" ]; then
      IFS="$(printf '\t')" read -r blocker_summary blocker_next <<EOF
$primary_blocker
EOF
      emit_primary_blocker "${title} primary blocker" "$blocker_summary" "$blocker_next"
    fi

    jq -r '.findings[] | [.severity, .summary, .next] | @tsv' "$input" \
      | while IFS="$(printf '\t')" read -r severity summary next; do
          [ -n "$severity" ] || continue
          emit_finding "$severity" "${title} finding" "$summary" "$next"
        done
    ;;
  workspace-doctor)
    primary_blocker="$(jq -r '.summary.primary_blocker? | select(. != null) | [.repo, .summary, .next] | @tsv' "$input" | head -n 1 || true)"
    if [ -n "$primary_blocker" ]; then
      IFS="$(printf '\t')" read -r blocker_repo blocker_summary blocker_next <<EOF
$primary_blocker
EOF
      emit_primary_blocker "${title} primary blocker [${blocker_repo}]" "$blocker_summary" "$blocker_next"
    fi

    jq -r '.repos[] | .name as $name | .path as $path | .findings[]? | [$name, $path, .severity, .summary, .next] | @tsv' "$input" \
      | while IFS="$(printf '\t')" read -r repo path severity summary next; do
          [ -n "$severity" ] || continue
          emit_finding "$severity" "${title} finding [${repo}]" "${path}: ${summary}" "$next"
        done
    ;;
  *)
    printf 'error: unsupported mode: %s\n' "$mode" >&2
    exit 2
    ;;
esac
