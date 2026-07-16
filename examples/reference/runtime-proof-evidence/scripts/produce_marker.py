# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import os
import urllib.request

marker = os.environ["OTA_PROOF_DEPENDENCY_MARKER"].encode()
request = urllib.request.Request("http://127.0.0.1:8081/marker", data=marker, method="POST")
with urllib.request.urlopen(request, timeout=3) as response:
    if response.status != 204:
        raise RuntimeError(f"unexpected dependency response: {response.status}")
