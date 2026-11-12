#!/usr/bin/env python3
"""Verify the unapplied evidence reconstruction proposal, not legacy adapters."""
import argparse
import hashlib
import json
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
q = json.loads((root / "PATH-DIGEST-CORRECTION-PROPOSAL-STATUS.json").read_bytes())
stage = root / q["proposal"]
archive = root / q["proposal_archive"]
sha = lambda data: hashlib.sha256(data).hexdigest()
assert sha((stage / "MANIFEST.json").read_bytes()) == q["proposal_manifest_sha256"]
assert sha(archive.read_bytes()) == q["proposal_archive_sha256"]
files = {p.relative_to(stage).as_posix() for p in stage.rglob("*") if p.is_file()}
with tarfile.open(archive, "r:gz") as tar:
    members = tar.getmembers()
    assert len(members) == len(files) == 21 and {m.name for m in members} == files
    for member in members:
        path = PurePosixPath(member.name)
        assert member.isfile() and not path.is_absolute() and ".." not in path.parts
        assert tar.extractfile(member).read() == (stage / path).read_bytes()
safety = root / "delegated/path-digest-correction-proposal-safety"
assert sha((safety / "MANIFEST.json").read_bytes()) == q["safety_receipt_manifest_sha256"]
subprocess.run([sys.executable, "-B", str(stage / "verify-reconstruction-only.py")], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str(safety / "verify-safety-receipts.py")], cwd=root, check=True)
parser = argparse.ArgumentParser()
parser.add_argument("--private-root", type=Path)
args = parser.parse_args()
if args.private_root:
    private = args.private_root / "path-digest-correction-proposal"
    assert sha((private / "PRIVATE-ORIGINAL-MANIFEST.json").read_bytes()) == q["restricted_original_manifest_sha256"]
    assert sha((private / "PRIVATE-SUPPLEMENTAL-MANIFEST.json").read_bytes()) == q["restricted_supplemental_manifest_sha256"]
    print("Selected private manifest identities checked, without repeating private substitution replay.")
else:
    print("Private inputs were not supplied and were not checked.")
print("The proposal is unapplied; legacy verifier adaptations and old public history remain separate.")
