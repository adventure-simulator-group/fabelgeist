#!/usr/bin/env python3
"""Verify the recovery package and restore selected-revision objects."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: restore_objects.py NEW_DESTINATION")
    here = Path(__file__).resolve().parent
    root = here.parent.parent
    dest = Path(sys.argv[1]).absolute()
    if dest.exists():
        raise FileExistsError("Restore destination already exists")
    subprocess.run([sys.executable, str(root / "verify_recovered.py")], check=True)
    manifest = json.loads((here / "MANIFEST.json").read_bytes())
    expected = {row["sha256"] for row in manifest["rows"]}
    dest.mkdir(parents=False, exist_ok=False)
    marker = dest / "RECOVERY-INCOMPLETE"
    marker.write_text("Remove only after investigating an interrupted restore.\n", encoding="utf-8")
    seen = set()
    with tarfile.open(here / "objects.tar.gz", "r:gz") as archive:
        for member in archive:
            if not member.isfile() or member.name not in expected or member.name in seen:
                raise ValueError("Unexpected archive member")
            inp = archive.extractfile(member)
            if inp is None:
                raise ValueError("Unreadable archive member")
            data = inp.read()
            if hashlib.sha256(data).hexdigest() != member.name:
                raise ValueError("Source object checksum mismatch")
            (dest / member.name).write_bytes(data)
            seen.add(member.name)
    if seen != expected:
        raise ValueError("Missing source objects")
    (dest / "MANIFEST.json").write_bytes((here / "MANIFEST.json").read_bytes())
    marker.unlink()
    print(f"Restored {len(seen)} selected revision objects")


if __name__ == "__main__":
    main()
