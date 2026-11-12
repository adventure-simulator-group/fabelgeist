#!/usr/bin/env python3
"""Verify bounded particle evidence and its historical ledger connection."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
stage = root / "delegated/central-13o-particles"
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = (stage / "MANIFEST.json").read_bytes()
assert sha(manifest) == "8d9839d9ba3f9c08f02312d0ee09b3dc6ab8b7daa87982ad3bb0c46ee1210b2f"
scan_dir = root / "delegated/central-13o-scan"
scan_receipt_bytes = (scan_dir / "RECEIPT.json").read_bytes()
assert sha(scan_receipt_bytes) == "122e16d054d850ad6323a08ed1a046181f4cfb44527bdd22ba7600baf3a391fe"
scan_receipt = json.loads(scan_receipt_bytes)
scan_bytes = (scan_dir / "SCAN-RESULT.json").read_bytes()
assert scan_receipt["stage_manifest_sha256"] == sha(manifest)
assert scan_receipt["scan_result_sha256"] == sha(scan_bytes)
scan = json.loads(scan_bytes)
assert scan["forbidden_digest_total"] == 0 and not scan["other_candidate_counts"] and not scan["exceptions"]
rows = json.loads(manifest)["files"]
assert len(rows) == 28
assert {p.relative_to(stage).as_posix() for p in stage.rglob("*") if p.is_file()} == {"MANIFEST.json"} | {r["path"] for r in rows}
for row in rows:
    path = PurePosixPath(row["path"])
    assert not path.is_absolute() and ".." not in path.parts
    data = (stage / path).read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
subprocess.run([sys.executable, "-B", str(stage / "verify-evidence.py"), "--publication", str(root)], cwd=root, check=True)
data = (root / "ASSESSMENT-CENTRAL-PARTICLES-13O-95E.json").read_bytes()
assert sha(data) == "40d5fe149df72e0b65a34a5f8370505c275f442db6161cf514d7d738dd9293d8"
draft = json.loads((stage / "ASSESSMENT-DRAFT.json").read_bytes())
final = json.loads(data)
assert {k: v for k, v in final.items() if k not in {"central_display_evidence", "review_qualification"}} == {
    k: v for k, v in draft.items() if k not in {"central_display_evidence", "review_qualification"}
}
assert final["central_display_evidence"] == {k: "delegated/central-13o-particles/" + v for k, v in draft["central_display_evidence"].items()}
ledger = json.loads((root / "BASELINE-95E-20261110.json").read_bytes())["newly_assessed_ledger"]["entries"]
found = [x for x in ledger if x["id"] == "RIG-ADMISSION03-F082"]
assert len(found) == 1 and found[0] == json.loads((stage / "LEDGER-PROPOSAL.json").read_bytes())
assert found[0]["repair"] == "not started" and found[0]["validation"] == "not performed"
queue = json.loads((root / "RECONCILIATION-059.json").read_bytes())["entries"]
assert queue[1856]["current_reconciliation"].startswith("partial ")
assert queue[1859]["current_reconciliation"].startswith("partial ")
progress = json.loads((root / "CONTINUATION-PROGRESS-95E.json").read_bytes())
assert progress["new_current_ledger_candidates"] == len(ledger)
assert progress["lenses_complete"] == progress["queue_records_semantically_reconciled"] == 0
print("Verified the recovered F082 ledger connection, separate related queue nonclosures and draft/final qualification.")
print("No independent semantic review, native behavior or product execution is established.")
