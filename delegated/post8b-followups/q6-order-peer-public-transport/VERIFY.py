#!/usr/bin/env python3
"""Check public transport archive identities and registered member bytes."""
import hashlib
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = json.loads((p / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-q6-order-peer-public-transport-v1"
assert {x.name for x in p.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    f = p / name
    assert f.is_file() and not f.is_symlink()
    raw = f.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
mapping = json.loads((p / "ARCHIVE-MAP.json").read_bytes())
assert mapping["schema"] == "726-post8b-q6-order-peer-public-transport-map-v1"
assert mapping["parent"] == "8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b"
assert mapping["source_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
assert len(mapping["collections"]) == 1 and len({x["label"] for x in mapping["collections"]}) == 1
files = 0
for row in mapping["collections"]:
    label = row["label"]
    assert label and all(c.isalnum() or c == "-" for c in label) and row["archive"] == label + ".tar.gz"
    raw = (p / row["archive"]).read_bytes()
    assert (len(raw), sha(raw)) == (row["archive_bytes"], row["archive_sha256"])
    with tarfile.open(p / row["archive"], "r:gz") as tf:
        members = tf.getmembers()
        assert len(members) == len(row["members"]) and len({m.name for m in members}) == len(members)
        expected = {x["path"]: x for x in row["members"]}
        assert set(expected) == {m.name for m in members}
        for member in members:
            name = PurePosixPath(member.name)
            assert not name.is_absolute() and str(name) == member.name and ".." not in name.parts and name.parts[0] == label
            item = expected[member.name]
            assert member.mode == item["mode"] and (member.isfile() or member.isdir())
            assert item["kind"] == ("file" if member.isfile() else "directory")
            if member.isfile():
                data = tf.extractfile(member).read()
                assert (len(data), sha(data)) == (item["bytes"], item["sha256"])
                if member.name == label + "/MANIFEST.json":
                    assert sha(data) == row["original_manifest_sha256"]
                files += 1
        assert label + "/MANIFEST.json" in expected and expected[label + "/MANIFEST.json"]["kind"] == "file"
print(json.dumps({"archives": len(mapping["collections"]), "files": files,
                  "qualification": "Archive and registered original-manifest identities only; individual packet claims, private originals, source semantics and independent custody remain separate."}))
