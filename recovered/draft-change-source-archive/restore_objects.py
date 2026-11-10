#!/usr/bin/env python3
"""Restore verified draft change blobs to a new content-addressed directory."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

if len(sys.argv) != 2:
    raise SystemExit("usage: restore_objects.py NEW_DESTINATION")
here = Path(__file__).resolve().parent
dest = Path(sys.argv[1]).absolute()
if dest.exists():
    raise FileExistsError("Destination already exists")
subprocess.run([sys.executable, str(here / "verify_archive.py")], check=True)
manifest = json.loads((here / "MANIFEST.json").read_bytes())
expected = {r[k]["oid"] for r in manifest["rows"] for k in ("before", "after") if r[k]}
dest.mkdir(parents=False, exist_ok=False)
marker = dest / "RECOVERY-INCOMPLETE"
marker.write_text("Investigate an interrupted restoration.\n", encoding="utf8")
seen = set()
with tarfile.open(here / "objects.tar.gz", "r:gz") as archive:
    for member in archive:
        parts = member.name.split("/")
        if len(parts) != 2 or parts[0] != "blob":
            continue
        oid = parts[1]
        if not member.isfile() or oid not in expected or oid in seen:
            raise ValueError("Unexpected draft source blob")
        inp = archive.extractfile(member)
        if inp is None:
            raise ValueError("Unreadable draft source blob")
        data = inp.read()
        if hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() != oid:
            raise ValueError("Draft source blob hash mismatch")
        (dest / oid).write_bytes(data)
        seen.add(oid)
if seen != expected:
    raise ValueError("Missing draft source blobs")
(dest / "MANIFEST.json").write_bytes((here / "MANIFEST.json").read_bytes())
marker.unlink()
print(f"Restored {len(seen)} draft source blobs")
