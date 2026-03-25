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

ota_warn() {
  if supports_color; then
    printf '\033[1;33m%s\033[0m\n' "$1" >&2
  else
    printf '%s\n' "$1" >&2
  fi
}

ota_error() {
  if supports_color; then
    printf '\033[1;31m%s\033[0m\n' "$1" >&2
  else
    printf '%s\n' "$1" >&2
  fi
}

downloader() {
  if command -v curl >/dev/null 2>&1; then
    printf "curl"
    return 0
  fi
  if command -v wget >/dev/null 2>&1; then
    printf "wget"
    return 0
  fi
  return 1
}

download_to() {
  url="$1"
  out="$2"
  dl="$(downloader || true)"
  if [ "${dl}" = "curl" ]; then
    curl -fsSL "${url}" -o "${out}"
    return $?
  fi
  if [ "${dl}" = "wget" ]; then
    wget -qO "${out}" "${url}"
    return $?
  fi
  return 1
}

resolve_target() {
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) return 1 ;;
  esac

  case "${os}" in
    linux) printf "%s-unknown-linux-gnu" "${arch}" ;;
    darwin) printf "%s-apple-darwin" "${arch}" ;;
    *) return 1 ;;
  esac
}

install_release_binary() {
  target="$(resolve_target || true)"
  if [ -z "${target}" ]; then
    ota_warn "warning: unsupported OS/arch for release binaries; trying cargo fallback"
    return 1
  fi

  version="${OTA_VERSION:-latest}"
  release_base="${OTA_RELEASE_BASE:-https://github.com/ota-run/ota/releases}"
  bin_dir="${OTA_BIN_DIR:-$HOME/.local/bin}"
  asset="ota-${target}.tar.gz"
  checksum_asset="ota-checksums.txt"

  case "${version}" in
    latest) download_prefix="${release_base}/latest/download" ;;
    *) download_prefix="${release_base}/download/${version}" ;;
  esac

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "${tmpdir}"' EXIT INT TERM
  archive="${tmpdir}/${asset}"
  checksums="${tmpdir}/${checksum_asset}"

  ota_info "installing ota ${version} for ${target}..."
  if ! download_to "${download_prefix}/${asset}" "${archive}"; then
    ota_warn "warning: release artifact not available (${asset}); trying cargo fallback"
    return 1
  fi

  if download_to "${download_prefix}/${checksum_asset}" "${checksums}"; then
    if command -v shasum >/dev/null 2>&1; then
      expected="$(grep " ${asset}\$" "${checksums}" | awk '{print $1}' || true)"
      if [ -n "${expected}" ]; then
        actual="$(shasum -a 256 "${archive}" | awk '{print $1}')"
        if [ "${actual}" != "${expected}" ]; then
          ota_error "error: checksum verification failed for ${asset}"
          return 1
        fi
      fi
    elif command -v sha256sum >/dev/null 2>&1; then
      expected="$(grep " ${asset}\$" "${checksums}" | awk '{print $1}' || true)"
      if [ -n "${expected}" ]; then
        actual="$(sha256sum "${archive}" | awk '{print $1}')"
        if [ "${actual}" != "${expected}" ]; then
          ota_error "error: checksum verification failed for ${asset}"
          return 1
        fi
      fi
    else
      ota_warn "warning: no sha256 tool found; skipping checksum verification"
    fi
  else
    ota_warn "warning: checksums not found; skipping checksum verification"
  fi

  if ! command -v tar >/dev/null 2>&1; then
    ota_error "error: tar is required to unpack release artifacts"
    return 1
  fi

  tar -xzf "${archive}" -C "${tmpdir}"
  if [ ! -f "${tmpdir}/ota" ]; then
    ota_error "error: release artifact did not contain ota binary"
    return 1
  fi

  mkdir -p "${bin_dir}"
  install -m 0755 "${tmpdir}/ota" "${bin_dir}/ota"
  ota_info "installed ota to ${bin_dir}/ota"
  return 0
}

install_from_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    ota_error "error: cargo is required for source/git install fallback"
    return 1
  fi

  install_from_source="$1"
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
      ota_error "error: set only one of OTA_GIT_TAG, OTA_GIT_BRANCH, OTA_GIT_REV"
      return 1
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
}

install_from_source=false
install_mode="${OTA_INSTALL_MODE:-release}"
if [ "${1-}" = "--from-source" ]; then
  install_from_source=true
  install_mode="source"
elif [ "${1-}" = "--from-git" ]; then
  install_mode="git"
elif [ "${1-}" = "--from-release" ]; then
  install_mode="release"
fi

if [ -f "./Cargo.toml" ] && grep -q '^name = "ota"$' "./Cargo.toml"; then
  if [ "${install_mode}" = "release" ]; then
    install_mode="source"
  fi
  install_from_source=true
fi

ota_header

if [ "${install_mode}" = "source" ]; then
  install_from_cargo "true"
elif [ "${install_mode}" = "git" ]; then
  install_from_cargo "false"
else
  if ! install_release_binary; then
    ota_warn "warning: falling back to git install via cargo"
    install_from_cargo "false"
  fi
fi

if command -v ota >/dev/null 2>&1; then
  ota --version
elif [ -n "${OTA_BIN_DIR:-}" ] && [ -x "${OTA_BIN_DIR}/ota" ]; then
  "${OTA_BIN_DIR}/ota" --version
elif [ -x "$HOME/.local/bin/ota" ]; then
  "$HOME/.local/bin/ota" --version
  ota_warn "warning: add $HOME/.local/bin to PATH to run 'ota' directly"
elif [ -x "$HOME/.cargo/bin/ota" ]; then
  "$HOME/.cargo/bin/ota" --version
  ota_warn "warning: add $HOME/.cargo/bin to PATH to run 'ota' directly"
else
  ota_error "error: install completed but \`ota\` is not on PATH yet"
  if [ "${install_mode}" = "release" ]; then
    ota_warn "next: export PATH=\"\$HOME/.local/bin:\$PATH\""
  else
    ota_warn "next: ensure cargo bin path is on PATH"
  fi
  exit 1
fi
