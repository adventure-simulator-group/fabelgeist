#!/usr/bin/env python3
"""Verify a source-only physics evidence packet and its archive."""
import argparse
import hashlib
import json
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
q = json.loads((root / "FOLLOWUP-44-PHYSICS-RETENTION.json").read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()
stage = root / q["public_stage"]
archive = root / q["archive"]
assert sha((stage / "PUBLIC-MANIFEST.json").read_bytes()) == q["public_manifest_sha256"]
assert sha((root / q["archive_index"]).read_bytes()) == q["archive_index_sha256"]
assert sha(archive.read_bytes()) == q["archive_sha256"]
files = {p.relative_to(stage).as_posix() for p in stage.rglob("*") if p.is_file()}
with tarfile.open(archive, "r:gz") as tar:
    members = tar.getmembers()
    assert len(members) == len(files) == 88 and {m.name for m in members} == files
    for member in members:
        path = PurePosixPath(member.name)
        assert member.isfile() and not path.is_absolute() and ".." not in path.parts
        assert tar.extractfile(member).read() == (stage / path).read_bytes()
subprocess.run([sys.executable, "-B", str(stage / "verify-public.py")], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str((root / q["archive_index"]).parent / "verify-archive.py")], cwd=root, check=True)
parser = argparse.ArgumentParser()
parser.add_argument("--private-root", type=Path)
args = parser.parse_args()
if args.private_root:
    original = args.private_root / "followup-44-physics-tests/original-evidence.tar.gz"
    assert sha(original.read_bytes()) == q["restricted_original_archive_sha256"]
    print("Restricted original archive identity checked; this alone does not prove independent custody.")
else:
    print("Restricted original archive was not supplied and was not checked.")
print("Source evidence only; no test, fixture, product, build or runtime execution.")
