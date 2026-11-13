#!/usr/bin/env python3
"""Verify a bounded public evidence transport; this does not validate product code."""
import hashlib
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
root = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = json.loads((root / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-public-transports-v1"
expected_files = set(manifest["files"]) | {"MANIFEST.json"}
assert {x.name for x in root.iterdir() if x.is_file()} == expected_files
assert all(x.is_file() and not x.is_symlink() for x in root.iterdir())
for name, identity in manifest["files"].items():
    raw = (root / name).read_bytes()
    assert (len(raw), sha(raw)) == (identity["bytes"], identity["sha256"])
mapping = json.loads((root / "ARCHIVE-MAP.json").read_bytes())
assert len(mapping["collections"]) == len({x["label"] for x in mapping["collections"]})
file_count = 0
for collection in mapping["collections"]:
    archive = root / collection["archive"]
    raw = archive.read_bytes()
    assert (len(raw), sha(raw)) == (collection["archive_bytes"], collection["archive_sha256"])
    expected = {x["path"]: x for x in collection["members"]}
    assert len(expected) == len(collection["members"])
    with tarfile.open(archive, "r:gz") as tar:
        members = tar.getmembers()
        assert len(members) == len(expected) and len({m.name for m in members}) == len(members)
        for member in members:
            name = member.name
            parts = PurePosixPath(name).parts
            assert parts and parts[0] == collection["label"] and ".." not in parts and not name.startswith("/")
            row = expected[name]
            assert member.mode == row["mode"] and member.uid == member.gid == 0 and member.mtime == 0
            if row["kind"] == "directory":
                assert member.isdir()
            else:
                assert row["kind"] == "file" and member.isfile()
                data = tar.extractfile(member).read()
                assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
                file_count += 1
print(json.dumps({"collections": len(mapping["collections"]), "file_members": file_count,
                  "qualification": "Public archive bytes and recorded modes only; no independent custody or product behavior."}))
