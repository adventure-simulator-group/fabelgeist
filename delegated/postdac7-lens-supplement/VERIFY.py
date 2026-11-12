#!/usr/bin/env python3
"""Verify additive lens correction and its time-bound restoration receipt."""
import gzip
import hashlib
import json
import subprocess
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
p = Path(__file__).resolve().parent
root = p.parent.parent
sha = lambda b: hashlib.sha256(b).hexdigest()
r = json.loads((p / "RECORD.json").read_bytes())
assert r["schema"] == "726-postdac7-lens-supplement-v1"
assert sha((p / "reviewer-correction/MANIFEST.json").read_bytes()) == r["reviewer_manifest_sha256"]
assert sha((p / "encrypted-originals/MANIFEST.json").read_bytes()) == r["encrypted_manifest_sha256"]
assert sha((p / "RESTORE-RESULT.json").read_bytes()) == r["restore_result_sha256"]
subprocess.run([sys.executable, "-B", str(p / "reviewer-correction/VERIFY.py"),
                "--publication", str(root)], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str(p / "encrypted-originals/VERIFY.py")],
               cwd=root, check=True)
restore = json.loads((p / "RESTORE-RESULT.json").read_bytes())
assert restore["ciphertext_manifest_sha256"] == r["encrypted_manifest_sha256"]
assert restore["correction_manifest_sha256"] == r["reviewer_manifest_sha256"]
assert len(restore["checks"]) == 3
assert all(x["exit"] == 0 and x["stderr_bytes"] == 0 for x in restore["checks"])
assert restore["preceding_v2_v3_driver_logs_match_results"]
assert restore["preceding_v3_scan_exit_zero"]
before = gzip.decompress((p / "CONTINUATION-BEFORE.md.gz").read_bytes())
after = gzip.decompress((p / "CONTINUATION-AFTER.md.gz").read_bytes())
assert sha(before) == r["continuation_before_sha256"]
assert sha(after) == r["continuation_after_sha256"]
previous = gzip.decompress((root / "delegated/postdac7-lens-map-correction"
                            / "CONTINUATION-AFTER.md.gz").read_bytes())
assert previous == before
anchor, insertion = r["anchor"].encode(), r["insertion"].encode()
assert before.count(anchor) == 1 and after == before.replace(anchor, insertion + anchor, 1)
print("Verified q2754/q2910 correction, authenticated-restoration receipt and "
      "continuation chain; decryption needs the separately held key.")
