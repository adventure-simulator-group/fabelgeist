#!/usr/bin/env python3
"""Verify bounded public proposals and historical-status projections."""
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = json.loads((p / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-reconciliation-public-v1"
assert {x.name for x in p.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    f = p / name
    assert f.is_file() and not f.is_symlink()
    raw = f.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
five = json.loads((p / "FIVE-PROPOSAL-v5.json").read_bytes())
three = json.loads((p / "THREE-PENDING-v4.json").read_bytes())
history = json.loads((p / "HISTORY-STATUS.json").read_bytes())
assert five["schema"] == "726-post8b-five-synthesis-proposal-v5"
assert five["status"].startswith("unadopted") and [x["index"] for x in five["assessments"]] == [2561,3044,2545,2743,2744]
assert three["schema"] == "726-post8b-three-central-draft-v4" and three["status"].startswith("unadopted")
assert [x["index"] for x in three["assessments"]] == [2913,2962,2952]
assert five["published_checkpoint"] == three["published_checkpoint"] == history["checkpoint"] == "8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b"
assert five["source_revision"] == three["source_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
assert history["counts"] == {"partial": 1404, "unassessed": 2309} and len(history["selected"]) == 8
assert all(x["current_reconciliation"] == "unassessed" and x["original_and_reconstructed_equal_as_parsed"] for x in history["selected"])
for name in ("PATH-CHECK-MATERIAL-v7.json", "PATH-CHECK-THREE-v4.json"):
    data = json.loads((p / name).read_bytes())
    assert all(x["exit"] == 0 for x in data["paths"])
print(json.dumps({"five_proposed": 5, "three_pending": 3, "historical_status_projections": 8,
                  "qualification": "Public identity and structural checks only; path existence does not establish source semantics or product behavior."}))
