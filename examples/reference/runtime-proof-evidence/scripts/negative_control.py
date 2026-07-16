# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

try:
    urllib.request.urlopen("http://127.0.0.1:1/marker", timeout=1)
except urllib.error.URLError:
    Path(os.environ["OTA_PROOF_NEGATIVE_CONTROL_ATTESTATION_FILE"]).write_text(
        json.dumps(
            {
                "transaction_id": os.environ["OTA_PROOF_TRANSACTION_ID"],
                "control_id": os.environ["OTA_PROOF_NEGATIVE_CONTROL_ID"],
                "obligation_id": os.environ["OTA_PROOF_NEGATIVE_CONTROL_OBLIGATION"],
                "failure_kind": "dependency_unavailable",
            }
        )
    )
    sys.exit(1)

raise RuntimeError("negative control unexpectedly reached the dependency")
