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

supports_color() {
  [ -t 2 ] && [ -z "${NO_COLOR-}" ]
}

ota_header() {
  if supports_color; then
    printf '\033[1;36m🦦  INSTALL\033[0m\n' >&2
    printf '\033[38;2;180;223;255m◉ doctor first, contract second\033[0m\n' >&2
  else
    printf ' INSTALL\n' >&2
    printf 'Signature: doctor first, contract second\n' >&2
  fi
}

ota_info() {
  if supports_color; then
    printf '\033[38;2;214;161;95m%s\033[0m\n' "$1" >&2
  else
    printf '%s\n' "$1" >&2
  fi
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to install ota" >&2
  exit 1
fi

install_from_source=false
if [ "${1-}" = "--from-source" ]; then
  install_from_source=true
fi

if [ -f "./Cargo.toml" ] && grep -q '^name = "ota"$' "./Cargo.toml"; then
  install_from_source=true
fi

ota_header

if [ "${install_from_source}" = "true" ]; then
  ota_info "installing ota from local source (cargo install --path .)..."
  cargo install --path . --locked --force
else
  git_url="${OTA_GIT_URL:-https://github.com/ota-run/ota.git}"
  tag="${OTA_GIT_TAG:-}"
  branch="${OTA_GIT_BRANCH:-}"
  rev="${OTA_GIT_REV:-}"
  refs_set=0

  if [ -n "${tag}" ]; then
    refs_set=$((refs_set + 1))
  fi
  if [ -n "${branch}" ]; then
    refs_set=$((refs_set + 1))
  fi
  if [ -n "${rev}" ]; then
    refs_set=$((refs_set + 1))
  fi

  if [ "${refs_set}" -gt 1 ]; then
    echo "error: set only one of OTA_GIT_TAG, OTA_GIT_BRANCH, OTA_GIT_REV" >&2
    exit 1
  fi

  ota_info "installing ota from ${git_url}..."
  if [ -n "${tag}" ]; then
    cargo install --git "${git_url}" --tag "${tag}" ota --locked --force
  elif [ -n "${branch}" ]; then
    cargo install --git "${git_url}" --branch "${branch}" ota --locked --force
  elif [ -n "${rev}" ]; then
    cargo install --git "${git_url}" --rev "${rev}" ota --locked --force
  else
    cargo install --git "${git_url}" ota --locked --force
  fi
fi

if command -v ota >/dev/null 2>&1; then
  ota --version
else
  echo "warning: install completed but \`ota\` is not on PATH yet" >&2
  exit 1
fi
