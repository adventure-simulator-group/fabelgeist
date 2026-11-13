#!/usr/bin/env python3
"""Verify identities and unadopted status of two post-fourth drafts."""
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
p = Path(__file__).resolve().parent
sha = lambda raw: hashlib.sha256(raw).hexdigest()
manifest = json.loads((p / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-postfourth-synthesis-transport-v1"
assert {x.name for x in p.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    path = p / name
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
first_raw, second_raw = (p / "LANGUAGE-BESTIARY.json").read_bytes(), (p / "CALENDAR-AGGREGATE.json").read_bytes()
assert sha(first_raw) == "d854ee2be36411a2847b016abe7fbf62738504f995123c84e1e58ba09bb02c9f"
assert sha(second_raw) == "865f1c4cc7ddf385e47a2fc62c3127330eb4881a97b795d912346d9d161385b2"
first, second = json.loads(first_raw), json.loads(second_raw)
assert first["status"] == second["status"] == "unadopted central draft; independent assembly review pending"
assert [x["index"] for x in first["assessments"]] == [3033, 3023] and second["assessment"]["index"] == 3594
assert not second["assessment"]["original_status_member_present"] and second["assessment"]["checkpoint_historical_status_projection"] is None
print(json.dumps({"drafts": 2, "queue_rows": 3, "adopted": False,
                  "qualification": "Draft bytes, selected metadata and unadopted status only; source correctness and independent assembly remain separate."}))
