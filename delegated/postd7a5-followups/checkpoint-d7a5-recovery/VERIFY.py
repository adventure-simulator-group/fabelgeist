#!/usr/bin/env python3
"""Verify the time-bound published-checkpoint recovery summaries."""
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
m = json.loads((p / "MANIFEST-v2.json").read_bytes())
assert m["schema"] == "726-postd7a5-recovery-public-v2"
assert sha((p / "MANIFEST.json").read_bytes()) == m["predecessor_manifest_sha256"]
for name, digest in m["files"].items():
    assert sha((p / name).read_bytes()) == digest
r = json.loads((p / "RECOVERY-RECORD.json").read_bytes())
pre = json.loads((p / "PRECOMMIT-RECORD.json").read_bytes())
assert r["checkpoint"] == "d7a5e77ebdbd44a05cde30881493d6fc22ef025d"
assert r["previous_checkpoint"] == pre["parent"] == "dac7d0055716053f245139971eb67ddfd87fa462"
assert r["captured_paths_checked"] == pre["staged_files"] == 417
assert r["recovery_driver_exit"] == pre["driver_exit"] == 0
assert r["prepublication_and_recovery_transcripts_byte_identical"]
assert r["transcript_sha256"] == pre["driver_transcript_sha256"]
assert r["before"] == r["after"] == {"head": r["checkpoint"], "clean": True, "symbolic_ref_exit": 1}
assert pre["strict_diff_check_exit"] == 2 and pre["recorded_exact_exception_files"] == 25
assert pre["excluded_remainder_exit"] == 0 and pre["staged_scans_exit_zero"]
print("Verified same-machine recovery summaries and recorded strict-diff exception; "
      "full logs and independent custody remain separate.")
