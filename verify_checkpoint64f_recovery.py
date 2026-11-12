#!/usr/bin/env python3
"""Verify same-machine checkpoint receipts without asserting remote custody."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
packet = root / "delegated/checkpoint64f-verification"
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = (packet / "MANIFEST.json").read_bytes()
assert sha(manifest) == "21192611370b6a9d98981ec80ec3a7760b09347721de76f1e7fdabf166c6fd1b"
rows = json.loads(manifest)["files"]
assert {p.name for p in packet.iterdir() if p.is_file()} == {"MANIFEST.json"} | {x["path"] for x in rows}
for row in rows:
    path = PurePosixPath(row["path"])
    assert not path.is_absolute() and ".." not in path.parts
    data = (packet / path).read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
pre = json.loads((packet / "PREPUBLISH-RECORD.json").read_bytes())
recovery = json.loads((packet / "RECOVERY-RECORD.json").read_bytes())
a = (packet / "PREPUBLISH-EVIDENCE.log").read_bytes()
b = (packet / "RECOVERY-EVIDENCE.log").read_bytes()
assert a == b and (len(a), a.count(b"\n"), sha(a)) == (140542, 1648, pre["evidence_log_sha256"])
assert sha(b) == recovery["transcript_sha256"]
assert b"Published evidence checks passed for the qualified public reconstructions" in b
assert pre["commit"] == recovery["commit"] == "64f0b01e1da02a6132fe18d77edebaa9ff0db96a"
assert pre["parent"] == recovery["parent"] == "a0d09cdb771c526f42df398da2ad57d7c6e10520"
scan = json.loads((packet / "FINAL-DIGEST-SCAN.json").read_bytes())
assert scan["stats"]["top_files"] == 584 and scan["forbidden_digest_total"] == 0 and not scan["exceptions"]
assert scan["other_candidate_counts"] == {"email": 48}
assert b"issues 0" in (packet / "FINAL-PATTERN-SCAN.txt").read_bytes()
main = json.loads((packet / "MAIN-POSTCHECK.json").read_bytes())
assert main["observed_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
parser = argparse.ArgumentParser()
parser.add_argument("--git-repo", type=Path)
args = parser.parse_args()
if args.git_repo:
    line = subprocess.check_output(["git", "-C", str(args.git_repo), "rev-list", "--parents", "-n", "1", pre["commit"]], text=True).strip()
    assert line.split() == [pre["commit"], pre["parent"]]
    print("Specified local Git repository contains the recorded commit-parent relationship.")
else:
    print("No Git repository supplied; the commit-parent relationship was not rechecked here.")
print("Byte-identical evidence transcripts and bounded scan receipts verified; checkout identity and remote custody remain separately qualified.")
