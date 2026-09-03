# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

marker = None
for _ in range(30):
    try:
        with urllib.request.urlopen("http://127.0.0.1:8081/marker", timeout=3) as response:
            if response.status == 200:
                candidate = response.read().decode()
                if candidate:
                    marker = candidate
                    break
    except (OSError, urllib.error.URLError):
        pass
    time.sleep(0.2)

if not marker:
    raise RuntimeError("dependency did not return the proof marker within the bounded retry window")

Path(os.environ["OTA_PROOF_ATTESTATION_FILE"]).write_text(
    json.dumps(
        {
            "transaction_id": os.environ["OTA_PROOF_TRANSACTION_ID"],
            "observation_id": os.environ["OTA_PROOF_OBSERVATION_ID"],
            "marker": marker,
        }
    )
)
