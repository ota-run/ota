#!/usr/bin/env sh
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

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
RUNTIME_DIR="$ROOT_DIR/.runtime"
AUTH_KEYS="$RUNTIME_DIR/authorized_keys"
COMPOSE_FILE="$ROOT_DIR/compose.yaml"
CONTAINER_NAME="ota-example-remote-sshd"
SSH_DIR="$HOME/.ssh"
SSH_CONFIG="$SSH_DIR/config"
SSH_INCLUDE="$SSH_DIR/ota-shared-remote-topology.conf"

if lsof -nP -iTCP:2222 -sTCP:LISTEN >/dev/null 2>&1 \
  && ! docker ps --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"
then
  printf '%s\n' "Local TCP port 2222 is already in use. Stop that service before using this dogfood example." >&2
  exit 1
fi

KEY_CANDIDATE=""
for candidate in \
  "$HOME/.ssh/id_ed25519.pub" \
  "$HOME/.ssh/id_rsa.pub" \
  "$HOME/.ssh/codespaces.jetbrains.pub"
do
  if [ -f "$candidate" ]; then
    KEY_CANDIDATE="$candidate"
    break
  fi
done

if [ -z "$KEY_CANDIDATE" ]; then
  printf '%s\n' "No default local SSH public key found under ~/.ssh. Create one first, then rerun prepare-dogfood.sh." >&2
  exit 1
fi

mkdir -p "$RUNTIME_DIR" "$HOME/.ssh"
cp "$KEY_CANDIDATE" "$AUTH_KEYS"
chmod 600 "$AUTH_KEYS"

PRIVATE_KEY=${KEY_CANDIDATE%.pub}

cat >"$SSH_INCLUDE" <<EOF
Host ota-remote-devbox
  HostName 127.0.0.1
  Port 2222
  User ota
  IdentityFile $PRIVATE_KEY
  IdentitiesOnly yes
  StrictHostKeyChecking no
  UserKnownHostsFile /dev/null
  LogLevel QUIET
EOF

chmod 600 "$SSH_INCLUDE"

touch "$SSH_CONFIG"
TEMP_CONFIG="$RUNTIME_DIR/ssh_config.tmp"
grep -Fvx "Include $SSH_INCLUDE" "$SSH_CONFIG" >"$TEMP_CONFIG" || true
{
  printf '%s\n' "Include $SSH_INCLUDE"
  cat "$TEMP_CONFIG"
} >"$SSH_CONFIG"
rm -f "$TEMP_CONFIG"
chmod 600 "$SSH_CONFIG"

docker compose -f "$COMPOSE_FILE" up -d --build

printf '%s\n' "Prepared dockerized ssh target."
printf '%s\n' "Using public key: $KEY_CANDIDATE"
printf '%s\n' "SSH include: $SSH_INCLUDE"
printf '%s\n' "Use: ./target/debug/ota run sandbox ./examples/shared-remote-topology --stream"
