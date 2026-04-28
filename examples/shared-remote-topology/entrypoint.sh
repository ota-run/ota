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

if ! id ota >/dev/null 2>&1; then
  adduser -D -s /bin/sh ota
fi

passwd -u ota >/dev/null 2>&1 || true

mkdir -p /home/ota/.ssh
cp /runtime/authorized_keys /home/ota/.ssh/authorized_keys
chown -R ota:ota /home/ota/.ssh
chmod 700 /home/ota/.ssh
chmod 600 /home/ota/.ssh/authorized_keys

ssh-keygen -A

cat >/etc/ssh/sshd_config <<'EOF'
Port 22
ListenAddress 0.0.0.0
Protocol 2
PermitRootLogin no
PasswordAuthentication no
KbdInteractiveAuthentication no
ChallengeResponseAuthentication no
AllowUsers ota
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
PidFile /var/run/sshd.pid
PrintMotd no
Subsystem sftp internal-sftp
EOF

exec /usr/sbin/sshd -D -e -f /etc/ssh/sshd_config
