#!/usr/bin/env python3
"""Check a point-in-time evidence scan receipt, not privacy of all history."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
folder = root / "delegated/checkpoint-privacy-prescan"
report = json.loads((folder / "REPORT.json").read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()
digest = (folder / "DIGEST-RESULT.json").read_bytes()
pattern = (folder / "PATTERN-RESULT.txt").read_bytes()
manifest = (folder / "INPUT-MANIFEST.json").read_bytes()
assert sha(digest) == report["digest_result_sha256"]
assert sha(pattern) == report["pattern_result_sha256"]
assert sha(manifest) == report["input_manifest_sha256"]
rows = json.loads(manifest)["files"]
assert len(rows) == report["input_files"] == 578
assert len({row["path"] for row in rows}) == len(rows)
for row in rows:
    path = PurePosixPath(row["path"])
    assert not path.is_absolute() and ".." not in path.parts
    assert len(row["sha256"]) == 64 and row["bytes"] >= 0
scan = json.loads(digest)
assert scan["forbidden_digest_total"] == 0 and not scan["exceptions"]
assert scan["other_candidate_counts"] == {"email": 48}
assert sum(item["count"] for item in scan["other_candidate_locations"]) == 48
assert all(item["kind"] == "email" and ".commit" in item["location"]
           for item in scan["other_candidate_locations"])
assert b"issues 0" in pattern
parser = argparse.ArgumentParser()
parser.add_argument("--private-root", type=Path)
args = parser.parse_args()
if args.private_root:
    path = args.private_root / "checkpoint-prescan-20261112/pattern-scanner.py"
    assert sha(path.read_bytes()) == report["pattern_scanner_sha256"]
    print("Retained same-machine pattern scanner identity checked.")
else:
    print("The separately retained pattern scanner was not supplied.")
print("Prior staged-input scan receipt checked; later files, old Git history and unknown representations remain outside its scope.")
