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
ota_bin="${OTA_BIN:-}"
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

usage() {
  cat <<'EOF'
usage: emit-ota-findings.sh --mode doctor|workspace-doctor|receipt-diff [--format plain|github|markdown] [--title TEXT] [--ota-bin PATH] --input FILE

Delegates to `ota annotations` so wrapper paths reuse the canonical CI and markdown renderers.
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
    --ota-bin)
      ota_bin="${2:-}"
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
    receipt-diff) title='ota receipt diff' ;;
    *)
      printf 'error: unsupported mode: %s\n' "$mode" >&2
      exit 2
      ;;
  esac
fi

if [ -z "$ota_bin" ]; then
  for candidate in \
    "$repo_root/target/debug/ota" \
    "$repo_root/target/release/ota"
  do
    if [ -x "$candidate" ]; then
      ota_bin="$candidate"
      break
    fi
  done
fi

if [ -z "$ota_bin" ] && [ -f "$repo_root/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
  exec cargo run --quiet --manifest-path "$repo_root/Cargo.toml" -- annotations --mode "$mode" --format "$format" --title "$title" --input "$input"
fi

if [ -z "$ota_bin" ] && command -v ota >/dev/null 2>&1; then
  ota_bin="ota"
fi

if [ -z "$ota_bin" ]; then
  printf 'error: could not resolve an ota binary; set OTA_BIN, pass --ota-bin, build the checkout, or install ota on PATH\n' >&2
  exit 2
fi

exec "$ota_bin" annotations --mode "$mode" --format "$format" --title "$title" --input "$input"
