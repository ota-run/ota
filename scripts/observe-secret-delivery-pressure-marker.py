#!/usr/bin/env python3
"""Point-in-time administrator observation, never execution authority or attestation."""

# Copyright (C) 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import os
import pwd
import stat
import sys
MARKER = "selected-work-executed"


def observe_marker_absence():
    if os.geteuid() != 0:
        raise SystemExit("marker observation requires the root administrator")
    execution = pwd.getpwnam("ota-authority-exec")
    if execution.pw_uid == 0 or execution.pw_gid == 0:
        raise SystemExit("marker observation execution owner is invalid")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    descriptor = os.open("/", flags)
    try:
        for component in ("srv", "ota-v3-pressure", None):
            metadata = os.fstat(descriptor)
            owner = (execution.pw_uid, execution.pw_gid) if component is None else (0, 0)
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or (metadata.st_uid, metadata.st_gid) != owner
                or (component is None and stat.S_IMODE(metadata.st_mode) != 0o750)
                or (component is not None and metadata.st_mode & 0o022)
            ):
                raise SystemExit("marker observation directory is invalid")
            if component is not None:
                child = os.open(component, flags, dir_fd=descriptor)
                os.close(descriptor)
                descriptor = child
        try:
            os.stat(MARKER, dir_fd=descriptor, follow_symlinks=False)
        except FileNotFoundError:
            return
        raise SystemExit("selected-work marker is present")
    finally:
        os.close(descriptor)


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit("marker observation accepts no arguments")
    try:
        observe_marker_absence()
    except (OSError, KeyError):
        sys.exit("marker observation is unavailable or uncertain")
    print("administrator_selected_work_marker_absent=true")
