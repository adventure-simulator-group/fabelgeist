#!/usr/bin/env python3
"""Restore hash-verified original registry source objects to a new directory."""
import gzip
import hashlib
import json
from pathlib import Path
import sys
import tarfile

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: restore_objects.py NEW_DESTINATION")
    here = Path(__file__).resolve().parent
    dest = Path(sys.argv[1]).absolute()
    manifest = json.loads((here / "MANIFEST.json").read_bytes())
    reg_gz = (here / "registry.json.gz").read_bytes()
    archive = (here / "objects.tar.gz").read_bytes()
    if sha(reg_gz) != manifest["registry_gzip_sha256"] or sha(archive) != manifest["objects_archive_sha256"]:
        raise ValueError("Archive checksum mismatch")
    registry = gzip.decompress(reg_gz)
    if (manifest["registry_sha256"] != "d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f"
            or sha(registry) != manifest["registry_sha256"]):
        raise ValueError("Registry checksum mismatch")
    files = json.loads(registry)["files"]
    expected = {row["sha256"] for row in manifest["sources"]}
    missing = [272, 501, 669, 670, 1537, 1958, 2416]
    if (len(files) != 2853 or len(expected) != 2462
            or [row["original"] for row in manifest["sources"]] != [i for i in range(2853) if i not in missing]
            or any(files[row["original"]].get("sha256") != row["sha256"] for row in manifest["sources"])):
        raise ValueError("Registry identity mismatch")
    dest.mkdir(parents=False, exist_ok=False)
    marker = dest / "RECOVERY-INCOMPLETE"
    marker.write_text("Remove only after investigating an interrupted restore.\n", encoding="utf-8")
    seen = set()
    with tarfile.open(here / "objects.tar.gz", "r:gz") as tar:
        for member in tar:
            if not member.isfile() or member.name not in expected or member.name in seen:
                raise ValueError("Unexpected or duplicate archive member")
            inp = tar.extractfile(member)
            if inp is None:
                raise ValueError("Archive member is unreadable")
            data = inp.read()
            if sha(data) != member.name:
                raise ValueError("Source object checksum mismatch")
            (dest / member.name).write_bytes(data)
            seen.add(member.name)
    if seen != expected:
        raise ValueError("Archive missing source objects")
    (dest / "original-scope-072.json").write_bytes(registry)
    marker.unlink()
    print(f"Verified {len(seen)} source objects and {len(manifest['sources'])} registry identities")

if __name__ == "__main__":
    main()
