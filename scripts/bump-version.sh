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

usage() {
  cat <<'EOF'
Usage:
  ./scripts/bump-version.sh <new-version>

Example:
  ./scripts/bump-version.sh 0.2.0
  ./scripts/bump-version.sh 0.2.0-rc.1
EOF
}

if [ "${1-}" = "-h" ] || [ "${1-}" = "--help" ]; then
  usage
  exit 0
fi

new_version="${OTA_INPUT_VERSION-${1-}}"
if [ -z "${new_version}" ]; then
  usage >&2
  exit 2
fi

if ! printf '%s' "$new_version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$'; then
  echo "error: version must look like semver (for example 0.2.0 or 0.2.0-rc.1)" >&2
  exit 2
fi

script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cargo_toml="$repo_root/Cargo.toml"

if [ ! -f "$cargo_toml" ]; then
  echo "error: Cargo.toml not found at $cargo_toml" >&2
  exit 1
fi

current_version=$(awk '
  BEGIN { in_package=0 }
  /^\[package\]/ { in_package=1; next }
  /^\[/ && $0 !~ /^\[package\]/ { in_package=0 }
  in_package && $0 ~ /^version = "/ {
    match($0, /"[^"]+"/);
    v=substr($0, RSTART+1, RLENGTH-2);
    print v;
    exit
  }
' "$cargo_toml")

if [ -z "$current_version" ]; then
  echo "error: could not locate [package] version in Cargo.toml" >&2
  exit 1
fi

tmp_file="$cargo_toml.tmp.$$"
awk -v new_version="$new_version" '
  BEGIN { in_package=0; replaced=0 }
  /^\[package\]/ { in_package=1; print; next }
  /^\[/ && $0 !~ /^\[package\]/ { in_package=0 }
  in_package && !replaced && $0 ~ /^version = "/ {
    print "version = \"" new_version "\""
    replaced=1
    next
  }
  { print }
  END {
    if (!replaced) {
      exit 3
    }
  }
' "$cargo_toml" > "$tmp_file" || {
  rc=$?
  rm -f "$tmp_file"
  if [ "$rc" -eq 3 ]; then
    echo "error: failed to update [package] version in Cargo.toml" >&2
    exit 1
  fi
  exit "$rc"
}

mv "$tmp_file" "$cargo_toml"

# Update VERSION file
version_file="$repo_root/VERSION"
printf '%s\n' "$new_version" > "$version_file"

printf '🦦 VERSION BUMP\n'
printf 'Updated: Cargo.toml, VERSION\n'
printf 'From: %s\n' "$current_version"
printf 'To:   %s\n' "$new_version"
printf '\nNext:\n'
printf '▸  run `cargo test`\n'
printf '▸  commit with message like `release: v%s`\n' "$new_version"
printf '▸  push to `main`; GitHub Actions will create `v%s` after the gate passes\n' "$new_version"
