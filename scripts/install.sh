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

setup_path=false

supports_color() {
  [ -t 2 ] && [ -z "${NO_COLOR-}" ]
}

ota_header() {
  if supports_color; then
    printf '\033[1;38;2;214;161;95m                █████\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m               ░░███\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m       ██████  ███████    ██████\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m      ███░░███░░░███░    ░░░░░███\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m     ░███ ░███  ░███      ███████\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m     ░███ ░███  ░███ ███ ███░░███\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m     ░░██████   ░░█████ ░░████████\033[0m\n' >&2
    printf '\033[1;38;2;214;161;95m      ░░░░░░     ░░░░░   ░░░░░░░░\033[0m\n' >&2
    printf '\n' >&2
    printf '\033[1;38;2;214;161;95m     DOCTOR FIRST, CONTRACT SECOND\033[0m\n' >&2
    printf '\n' >&2
  else
    printf '                █████\n' >&2
    printf '               ░░███\n' >&2
    printf '       ██████  ███████    ██████\n' >&2
    printf '      ███░░███░░░███░    ░░░░░███\n' >&2
    printf '     ░███ ░███  ░███      ███████\n' >&2
    printf '     ░███ ░███  ░███ ███ ███░░███\n' >&2
    printf '     ░░██████   ░░█████ ░░████████\n' >&2
    printf '      ░░░░░░     ░░░░░   ░░░░░░░░\n' >&2
    printf '\n' >&2
    printf '      DOCTOR FIRST, CONTRACT SECOND\n' >&2
    printf '\n' >&2
  fi
}

ota_info() {
  if supports_color; then
    printf '\033[38;2;214;161;95m%s\033[0m\n' "$1" >&2
  else
    printf '%s\n' "$1" >&2
  fi
}

ota_receipt() {
  if supports_color; then
    printf '\033[1;38;2;214;161;95m%s\033[0m\n' "$1" >&2
  else
    printf '%s\n' "$1" >&2
  fi
}

ota_receipt_line() {
  if supports_color; then
    printf '\033[1;38;2;214;161;95m➤\033[0m \033[1;37m%s\033[0m\n' "$1" >&2
  else
    printf '➤ %s\n' "$1" >&2
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

detect_shell_rc_file() {
  shell_name="$(basename "${SHELL:-sh}")"
  case "${shell_name}" in
    zsh) printf "%s" "$HOME/.zshrc" ;;
    bash)
      if [ -f "$HOME/.bashrc" ] || [ ! -f "$HOME/.bash_profile" ]; then
        printf "%s" "$HOME/.bashrc"
      else
        printf "%s" "$HOME/.bash_profile"
      fi
      ;;
    fish) printf "%s" "$HOME/.config/fish/config.fish" ;;
    ksh) printf "%s" "$HOME/.kshrc" ;;
    *) printf "%s" "$HOME/.profile" ;;
  esac
}

persist_path_update() {
  dir="$1"
  rc_file="$(detect_shell_rc_file)"
  rc_dir="$(dirname "${rc_file}")"
  shell_name="$(basename "${SHELL:-sh}")"
  begin_marker="# >>> ota PATH >>>"
  end_marker="# <<< ota PATH <<<"

  mkdir -p "${rc_dir}"
  if [ ! -f "${rc_file}" ]; then
    : > "${rc_file}"
  fi

  if grep -F "${begin_marker}" "${rc_file}" >/dev/null 2>&1; then
    ota_info "PATH setup already exists in ${rc_file}"
    ota_warn "next: restart your shell or source ${rc_file}"
    return 0
  fi

  {
    printf '\n%s\n' "${begin_marker}"
    if [ "${shell_name}" = "fish" ]; then
      printf 'if not contains -- "%s" $PATH\n' "${dir}"
      printf '    set -gx PATH "%s" $PATH\n' "${dir}"
      printf 'end\n'
    else
      printf 'case ":$PATH:" in\n'
      printf '  *:"%s":*) ;;\n' "${dir}"
      printf '  *) export PATH="%s:$PATH" ;;\n' "${dir}"
      printf 'esac\n'
    fi
    printf '%s\n' "${end_marker}"
  } >> "${rc_file}"

  ota_info "added ${dir} to PATH in ${rc_file}"
  ota_warn "next: restart your shell or source ${rc_file}"
}

setup_path_rerun_command() {
  release_base="${OTA_RELEASE_BASE:-https://dist.ota.run}"
  case "${install_mode}" in
    source)
      printf "./scripts/install.sh --from-source --setup-path"
      ;;
    git)
      printf "curl -fsSL %s/install.sh | sh -s -- --from-git --setup-path" "${release_base}"
      ;;
    *)
      printf "curl -fsSL %s/install.sh | sh -s -- --setup-path" "${release_base}"
      ;;
  esac
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

extract_zip_to() {
  archive="$1"
  dest="$2"

  if command -v unzip >/dev/null 2>&1; then
    unzip -oq "${archive}" -d "${dest}"
    return $?
  fi

  if command -v pwsh >/dev/null 2>&1; then
    pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -Command \
      "Expand-Archive -Path '${archive}' -DestinationPath '${dest}' -Force" >/dev/null
    return $?
  fi

  if command -v powershell >/dev/null 2>&1; then
    powershell -NoLogo -NoProfile -ExecutionPolicy Bypass -Command \
      "Expand-Archive -Path '${archive}' -DestinationPath '${dest}' -Force" >/dev/null
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
    msys*|mingw*|cygwin*) printf "%s-pc-windows-msvc" "${arch}" ;;
    *) return 1 ;;
  esac
}

checksum_expected_value() {
  checksums_path="$1"
  asset_name="$2"
  awk -v asset="${asset_name}" '
    $2 == asset || $2 == ("dist/" asset) { print $1; exit }
  ' "${checksums_path}"
}

install_release_binary() {
  target="$(resolve_target || true)"
  if [ -z "${target}" ]; then
    ota_warn "warning: no published prebuilt ota release is configured for this OS/arch"
    return 1
  fi

  version="${OTA_VERSION:-latest}"
  release_base="${OTA_RELEASE_BASE:-https://github.com/ota-run/ota/releases}"
  bin_dir="${OTA_BIN_DIR:-$HOME/.local/bin}"
  case "${target}" in
    *-pc-windows-msvc) asset="ota-${target}.zip" ;;
    *) asset="ota-${target}.tar.gz" ;;
  esac
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
    ota_warn "warning: could not download prebuilt ota release asset for ${target} at ${version} (${asset})"
    return 1
  fi

  if download_to "${download_prefix}/${checksum_asset}" "${checksums}"; then
    if command -v shasum >/dev/null 2>&1; then
      expected="$(checksum_expected_value "${checksums}" "${asset}" || true)"
      if [ -n "${expected}" ]; then
        actual="$(shasum -a 256 "${archive}" | awk '{print $1}')"
        if [ "${actual}" != "${expected}" ]; then
          ota_error "error: checksum verification failed for ${asset}"
          return 1
        fi
      fi
    elif command -v sha256sum >/dev/null 2>&1; then
      expected="$(checksum_expected_value "${checksums}" "${asset}" || true)"
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

  case "${asset}" in
    *.zip)
      if ! extract_zip_to "${archive}" "${tmpdir}"; then
        ota_error "error: unzip or PowerShell Expand-Archive is required to unpack Windows release artifacts"
        return 1
      fi
      ;;
    *)
      if ! command -v tar >/dev/null 2>&1; then
        ota_error "error: tar is required to unpack release artifacts"
        return 1
      fi
      tar -xzf "${archive}" -C "${tmpdir}"
      ;;
  esac
  mkdir -p "${bin_dir}"
  if printf '%s' "${target}" | grep -q 'windows-msvc$'; then
    if [ ! -f "${tmpdir}/ota.exe" ]; then
      ota_error "error: release artifact did not contain ota.exe"
      return 1
    fi
    staged="${bin_dir}/ota.exe.new"
    install -m 0755 "${tmpdir}/ota.exe" "${staged}"
    mv -f "${staged}" "${bin_dir}/ota.exe"
    ota_info "installed ota to ${bin_dir}/ota.exe"
  else
    if [ ! -f "${tmpdir}/ota" ]; then
      ota_error "error: release artifact did not contain ota binary"
      return 1
    fi
    staged="${bin_dir}/ota.new"
    install -m 0755 "${tmpdir}/ota" "${staged}"
    mv -f "${staged}" "${bin_dir}/ota"
    ota_info "installed ota to ${bin_dir}/ota"
  fi
  return 0
}

install_from_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    ota_error "error: cargo is required for source/git install fallback"
    ota_error "install Rust/cargo or use a published prebuilt ota release for your target"
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
install_mode_forced=false
if [ "${OTA_INSTALL_MODE+x}" = "x" ]; then
  install_mode_forced=true
fi

while [ "$#" -gt 0 ]; do
  case "$1" in
    --from-source)
      install_from_source=true
      install_mode="source"
      install_mode_forced=true
      ;;
    --from-git)
      install_mode="git"
      install_mode_forced=true
      ;;
    --from-release)
      install_mode="release"
      install_mode_forced=true
      ;;
    --setup-path)
      setup_path=true
      ;;
    *)
      ota_error "error: unknown install flag: $1"
      exit 1
      ;;
  esac
  shift
done

if [ "${install_mode_forced}" != "true" ] && [ -f "./Cargo.toml" ] && grep -q '^name = "ota"$' "./Cargo.toml"; then
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
    if [ "${install_mode}" = "release" ] && [ "${install_mode_forced}" = "true" ]; then
      ota_error "error: prebuilt release install failed; refusing cargo fallback in explicit release mode"
      exit 1
    fi
    ota_warn "warning: falling back to git install via cargo"
    install_from_cargo "false"
  fi
fi

version_output=""
binary_path=""
binary_name="ota"
case "$(resolve_target || true)" in
  *-pc-windows-msvc) binary_name="ota.exe" ;;
esac

if command -v ota >/dev/null 2>&1; then
  binary_path="$(command -v ota)"
  version_output="$(ota --version 2>/dev/null || true)"
elif [ -n "${OTA_BIN_DIR:-}" ] && [ -x "${OTA_BIN_DIR}/${binary_name}" ]; then
  binary_path="${OTA_BIN_DIR}/${binary_name}"
  version_output="$("${OTA_BIN_DIR}/${binary_name}" --version 2>/dev/null || true)"
elif [ -x "$HOME/.local/bin/${binary_name}" ]; then
  binary_path="$HOME/.local/bin/${binary_name}"
  version_output="$("$HOME/.local/bin/${binary_name}" --version 2>/dev/null || true)"
  if [ "${setup_path}" = "true" ]; then
    persist_path_update "$HOME/.local/bin"
  else
    ota_warn "warning: add $HOME/.local/bin to PATH to run 'ota' directly"
    ota_warn "next: rerun \`$(setup_path_rerun_command)\` to persist it automatically"
  fi
elif [ -x "$HOME/.cargo/bin/${binary_name}" ]; then
  binary_path="$HOME/.cargo/bin/${binary_name}"
  version_output="$("$HOME/.cargo/bin/${binary_name}" --version 2>/dev/null || true)"
  if [ "${setup_path}" = "true" ]; then
    persist_path_update "$HOME/.cargo/bin"
  else
    ota_warn "warning: add $HOME/.cargo/bin to PATH to run 'ota' directly"
    ota_warn "next: rerun \`$(setup_path_rerun_command)\` to persist it automatically"
  fi
else
  ota_error "error: install completed but \`ota\` is not on PATH yet"
  if [ "${install_mode}" = "release" ]; then
    ota_warn "next: export PATH=\"\$HOME/.local/bin:\$PATH\""
  else
    ota_warn "next: ensure cargo bin path is on PATH"
  fi
  if [ "${setup_path}" = "true" ]; then
    ota_warn "next: rerun after confirming the installed binary location"
  else
    ota_warn "next: rerun \`$(setup_path_rerun_command)\` to persist PATH automatically"
  fi
  exit 1
fi

version_text="${version_output#🦦 }"
version_text="${version_text#ota }"

duplicate_paths=""
if [ -x "$HOME/.local/bin/${binary_name}" ] && [ "$binary_path" != "$HOME/.local/bin/${binary_name}" ]; then
  duplicate_paths="${duplicate_paths}${duplicate_paths:+, }$HOME/.local/bin/${binary_name}"
fi
if [ -x "$HOME/.cargo/bin/${binary_name}" ] && [ "$binary_path" != "$HOME/.cargo/bin/${binary_name}" ]; then
  duplicate_paths="${duplicate_paths}${duplicate_paths:+, }$HOME/.cargo/bin/${binary_name}"
fi
if [ -n "$duplicate_paths" ]; then
  ota_warn "warning: multiple ota binaries were found; PATH is using $binary_path"
  ota_warn "warning: remove or de-prioritize the other copy/copies: $duplicate_paths"
fi

ota_receipt "🦦 READY"
ota_receipt_line "${version_text}"
