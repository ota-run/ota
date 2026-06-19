#!/usr/bin/env bash
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

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: install.sh [path-to-ota-yaml-or-repo] [--resolve-only]
EOF
}

strip_wrapping_quotes() {
  local value="$1"
  case "$value" in
    \"*\") printf '%s\n' "${value:1:${#value}-2}" ;;
    \'*\') printf '%s\n' "${value:1:${#value}-2}" ;;
    *) printf '%s\n' "$value" ;;
  esac
}

resolve_contract_path() {
  local input="$1"
  if [ -d "$input" ]; then
    input="${input%/}/ota.yaml"
  fi
  if [ ! -f "$input" ]; then
    echo "error: contract path '$input' was not found" >&2
    exit 1
  fi
  printf '%s\n' "$input"
}

parse_bootstrap_fields() {
  local contract_path="$1"
  awk '
    function ltrim(s) { sub(/^[[:space:]]+/, "", s); return s }
    function rtrim(s) { sub(/[[:space:]]+$/, "", s); return s }
    function trim(s) { return rtrim(ltrim(s)) }
    function strip_inline_comment(s,    i, c, out, in_single, in_double, prev) {
      out = ""
      in_single = 0
      in_double = 0
      prev = ""
      for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "\"" && !in_single && prev != "\\") {
          in_double = !in_double
        } else if (c == "'"'"'" && !in_double) {
          in_single = !in_single
        }
        if (c == "#" && !in_single && !in_double) {
          break
        }
        out = out c
        prev = c
      }
      return out
    }

    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }

    {
      line = $0
      indent = 0
      while (substr(line, indent + 1, 1) == " ") {
        indent++
      }
      trimmed = substr(line, indent + 1)
      colon = index(trimmed, ":")
      if (colon == 0) {
        next
      }
      key = trim(substr(trimmed, 1, colon - 1))
      if (key == "" || key ~ /^[#-]/) {
        next
      }
      value = trim(strip_inline_comment(substr(trimmed, colon + 1)))

      while (depth > 0 && indents[depth] >= indent) {
        delete path_keys[depth]
        delete indents[depth]
        depth--
      }

      depth++
      path_keys[depth] = key
      indents[depth] = indent

      if (value == "") {
        next
      }

      path = path_keys[1]
      for (i = 2; i <= depth; i++) {
        path = path "." path_keys[i]
      }

      if (path == "agent.bootstrap.ota.source.kind" || path == "agent.bootstrap.ota.source.version" || path == "agent.bootstrap.ota.source.rev" || path == "agent.bootstrap.ota.source.branch" || path == "agent.bootstrap.ota.sh" || path == "agent.bootstrap.ota.powershell") {
        print path "\t" value
      }
    }
  ' "$contract_path"
}

extract_marker_value() {
  local command="$1"
  shift
  local pattern regex
  for pattern in "$@"; do
    regex="${pattern}[[:space:]]*=[[:space:]]*['\"]?([^'\"[:space:];|]+)['\"]?"
    if [[ "$command" =~ $regex ]]; then
      printf '%s\n' "${BASH_REMATCH[1]}"
      return 0
    fi
  done
  return 1
}

infer_source_from_command() {
  local command="$1"
  local branch rev version
  branch="$(extract_marker_value "$command" 'OTA_GIT_BRANCH' '\$env:OTA_GIT_BRANCH' || true)"
  if [ -n "$branch" ]; then
    printf 'branch\t%s\n' "$branch"
    return 0
  fi
  rev="$(extract_marker_value "$command" 'OTA_GIT_REV' '\$env:OTA_GIT_REV' || true)"
  if [ -n "$rev" ]; then
    printf 'git_rev\t%s\n' "$rev"
    return 0
  fi
  version="$(extract_marker_value "$command" 'OTA_VERSION' '\$env:OTA_VERSION' || true)"
  if [ -n "$version" ]; then
    printf 'version\t%s\n' "$version"
    return 0
  fi
  return 1
}

emit_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf '%s=%s\n' "$key" "$value" >> "$GITHUB_OUTPUT"
  fi
}

contract_input="ota.yaml"
resolve_only=0

for arg in "$@"; do
  case "$arg" in
    --resolve-only) resolve_only=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [ "$contract_input" = "ota.yaml" ]; then
        contract_input="$arg"
      else
        usage
        exit 1
      fi
      ;;
  esac
done

contract_path="$(resolve_contract_path "$contract_input")"

kind=""
version=""
rev=""
branch=""
sh_command=""
powershell_command=""

while IFS=$'\t' read -r path value; do
  value="$(strip_wrapping_quotes "$value")"
  case "$path" in
    agent.bootstrap.ota.source.kind) kind="$value" ;;
    agent.bootstrap.ota.source.version) version="$value" ;;
    agent.bootstrap.ota.source.rev) rev="$value" ;;
    agent.bootstrap.ota.source.branch) branch="$value" ;;
    agent.bootstrap.ota.sh) sh_command="$value" ;;
    agent.bootstrap.ota.powershell) powershell_command="$value" ;;
  esac
done < <(parse_bootstrap_fields "$contract_path")

if [ -z "$kind" ]; then
  inferred="$(infer_source_from_command "$sh_command" || infer_source_from_command "$powershell_command" || true)"
  if [ -n "$inferred" ]; then
    kind="${inferred%%$'\t'*}"
    value="${inferred#*$'\t'}"
    case "$kind" in
      version) version="$value" ;;
      git_rev) rev="$value" ;;
      branch) branch="$value" ;;
    esac
  fi
fi

case "$kind" in
  version)
    if [ -z "$version" ]; then
      echo "error: agent.bootstrap.ota.source.kind is version but no version was declared" >&2
      exit 1
    fi
    export OTA_VERSION="$version"
    from_git=0
    ;;
  git_rev)
    if [ -z "$rev" ]; then
      echo "error: agent.bootstrap.ota.source.kind is git_rev but no rev was declared" >&2
      exit 1
    fi
    export OTA_GIT_REV="$rev"
    from_git=1
    ;;
  branch)
    if [ -z "$branch" ]; then
      echo "error: agent.bootstrap.ota.source.kind is branch but no branch was declared" >&2
      exit 1
    fi
    export OTA_GIT_BRANCH="$branch"
    from_git=1
    ;;
  "")
    echo "error: contract does not declare a usable agent.bootstrap.ota source" >&2
    echo "next: declare agent.bootstrap.ota.source or use a legacy bootstrap command that includes OTA_VERSION, OTA_GIT_REV, or OTA_GIT_BRANCH" >&2
    exit 1
    ;;
  *)
    echo "error: unsupported agent.bootstrap.ota.source.kind '$kind'" >&2
    exit 1
    ;;
esac

emit_output "source-kind" "$kind"
emit_output "contract-path" "$contract_path"
emit_output "version" "$version"
emit_output "git-rev" "$rev"
emit_output "git-branch" "$branch"

if [ "$resolve_only" -eq 1 ]; then
  printf 'CONTRACT_PATH=%s\n' "$contract_path"
  printf 'SOURCE_KIND=%s\n' "$kind"
  [ -n "$version" ] && printf 'OTA_VERSION=%s\n' "$version"
  [ -n "$rev" ] && printf 'OTA_GIT_REV=%s\n' "$rev"
  [ -n "$branch" ] && printf 'OTA_GIT_BRANCH=%s\n' "$branch"
  if [ "${from_git:-0}" -eq 1 ]; then
    printf 'INSTALL_ARGS=--from-git\n'
  else
    printf 'INSTALL_ARGS=\n'
  fi
  exit 0
fi

if [ "${from_git:-0}" -eq 1 ]; then
  curl -fsSL https://dist.ota.run/install.sh | sh -s -- --from-git
else
  curl -fsSL https://dist.ota.run/install.sh | sh
fi
