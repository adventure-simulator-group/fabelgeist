#!/usr/bin/env python3
"""Check public XPBD archive and evidence stage; private custody is separate."""
import hashlib
import json
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
q = json.loads((root / "FOLLOWUP-43-PRIVATE-RETENTION.json").read_bytes())
stage = root / q["public_stage"]
archive = root / q["public_archive"]
digest = lambda data: hashlib.sha256(data).hexdigest()
assert digest((stage / "MANIFEST.json").read_bytes()) == q["public_manifest_sha256"]
assert digest(archive.read_bytes()) == q["public_archive_sha256"]
files = {p.relative_to(stage).as_posix() for p in stage.rglob("*") if p.is_file()}
with tarfile.open(archive, "r:gz") as tar:
    members = tar.getmembers()
    assert len(members) == len(files) == 142
    assert {m.name for m in members} == files
    for member in members:
        name = PurePosixPath(member.name)
        assert member.isfile() and not name.is_absolute() and ".." not in name.parts
        assert tar.extractfile(member).read() == (stage / name).read_bytes()
subprocess.run([sys.executable, "-B", str(stage / "verify-public-evidence.py")],
               cwd=root, check=True)
print("Public stage and complete archive match; private replay and earlier history remain separately qualified.")
