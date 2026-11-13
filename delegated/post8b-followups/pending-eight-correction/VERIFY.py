#!/usr/bin/env python3
"""Verify an additive correction against the preserved eight-row draft."""
import copy
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
here = Path(__file__).resolve().parent
prior = here.parent / "pending-eight"
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((here / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-eight-correction-public-v1"
assert {p.name for p in here.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    p = here / name
    assert p.is_file() and not p.is_symlink()
    data = p.read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
prior_manifest_raw = (prior / "MANIFEST.json").read_bytes()
assert sha(prior_manifest_raw) == "4f409bc6e5e34678d83667c2a8a325d89da045df1e495d29fe22490ab4bc8eed"
prior_manifest = json.loads(prior_manifest_raw)
assert {p.name for p in prior.iterdir()} == set(prior_manifest["files"]) | {"MANIFEST.json"}
for name, row in prior_manifest["files"].items():
    p = prior / name
    assert p.is_file() and not p.is_symlink()
    data = p.read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
prior_raw = (prior / "DRAFT.json").read_bytes()
assert sha(prior_raw) == "0c287d13b3f10b9b0457d0805957d6ce3b581fb325a401ff2aae15254de2e8dc"
expected = copy.deepcopy(json.loads(prior_raw))
assert expected["schema"] == "726-post8b-next-eight-central-draft-v1"
expected["schema"] = "726-post8b-next-eight-central-draft-v2"
expected["groups"][0]["summary"] = "Completed scene floor versus terrain support/clearance and raise-only versus follow-floor have distinct selected consumers. Four retained ankle/toe options form a local ordered pair; no supported swap is shown. Negative finite settle progress produces different symbolic clamp/raw treatment; the selected ordinary plan initializes zero, while timing-helper reach, final effect and policy remain open."
expected["groups"][1]["relationship"] = "q2700 retains its rejected ordinary native suspicion and open public admission policy; q2823 is a separate partial Lens 11 shape assessment. Ledger1126 is a planning relationship linking those records with normalized 774 F045 context. q2697's clue admission is distinct; no global alias or unique count follows."
expected["correction"] = {"predecessor_sha256": sha(prior_raw),
                          "scope": "Clarify the selected zero initialization and the separate puzzle records; all other draft content is unchanged."}
assert json.loads((here / "DRAFT-v2.json").read_bytes()) == expected
print(json.dumps({"corrected_fields": 2, "predecessor_preserved": True,
                  "qualification": "Additive draft correction only; independent assembly review, adoption and semantic closure remain pending."}))
