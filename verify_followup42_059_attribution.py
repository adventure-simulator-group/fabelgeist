#!/usr/bin/env python3
"""Bind an additive attribution qualification to frozen evidence bytes."""
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
q = json.loads((root / "QUALIFICATION-FOLLOWUP42-059-ATTRIBUTION.json").read_bytes())
digest = lambda data: hashlib.sha256(data).hexdigest()
archive = root / q["archive"]
assert digest(archive.read_bytes()) == q["archive_sha256"]
with tarfile.open(archive, "r:gz") as packet:
    report_bytes = packet.extractfile(q["report_member"]).read()
    receipt_bytes = packet.extractfile(q["receipt_member"]).read()
assert digest(report_bytes) == q["report_sha256"]
assert digest(receipt_bytes) == q["receipt_sha256"]
report = json.loads(report_bytes)
receipt = json.loads(receipt_bytes)
assert report["historical_attribution"][2]["original_author"] == q["report_label"]
assert not {"author", "reader", "reviewer"}.intersection(receipt)
for row in q["rows"]:
    index = int(row["pointer"].rsplit("/", 1)[1])
    historical = receipt["sources"][index]
    assert historical["original_index"] == row["role"]
    assert historical["fresh_personal_body_reads"] == row["fresh_personal_body_reads"]
    assert not {"author", "reader", "reviewer"}.intersection(historical)
print("Frozen attribution and receipt rows verified; a named reader is not established by these fields alone.")
