#!/usr/bin/env python3
"""Verify public packet transports without running product code."""
import gzip
import hashlib
import io
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
m = json.loads((p / "MANIFEST.json").read_bytes())
assert m["schema"] == "726-postd7a5-public-dispatch-transports-v1"
assert m["source_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
assert m["parent_checkpoint"] == "d7a5e77ebdbd44a05cde30881493d6fc22ef025d"
names = {"MANIFEST.json", "VERIFY.py"}
count = 0
for row in m["bundles"]:
    assert row["file"] == row["label"] + ".tar.gz" and row["file"] not in names
    names.add(row["file"])
    raw = (p / row["file"]).read_bytes()
    assert (len(raw), sha(raw)) == (row["archive_bytes"], row["archive_sha256"])
    wanted = {x["path"]: x for x in row["members"]}
    assert len(wanted) == len(row["members"])
    seen = set()
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
        for member in archive:
            name = PurePosixPath(member.name)
            assert not name.is_absolute() and name.parts[0] == "packet" and ".." not in name.parts
            assert member.isfile() and member.name in wanted and member.name not in seen
            seen.add(member.name)
            entry = wanted[member.name]
            data = archive.extractfile(member).read()
            assert (len(data), sha(data), oct(member.mode)) == (entry["bytes"], entry["sha256"], entry["mode"])
            if member.name == "packet/MANIFEST.json":
                assert sha(data) == row["source_manifest_sha256"]
    assert seen == set(wanted) and "packet/MANIFEST.json" in seen
    count += len(seen)
assert {x.name for x in p.iterdir()} == names
print(f"Verified {len(m['bundles'])} public transports and {count} exact members; "
      "restricted provenance and original replay dependencies remain separate.")
