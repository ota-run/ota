# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import json
import os
import urllib.request
from pathlib import Path

with urllib.request.urlopen("http://127.0.0.1:8081/marker", timeout=3) as response:
    marker = response.read().decode()

if not marker:
    raise RuntimeError("dependency did not return the proof marker")

Path(os.environ["OTA_PROOF_ATTESTATION_FILE"]).write_text(
    json.dumps(
        {
            "transaction_id": os.environ["OTA_PROOF_TRANSACTION_ID"],
            "observation_id": os.environ["OTA_PROOF_OBSERVATION_ID"],
            "marker": marker,
        }
    )
)
