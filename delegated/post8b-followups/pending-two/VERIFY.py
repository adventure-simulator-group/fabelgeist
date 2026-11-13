#!/usr/bin/env python3
"""Check a pending two-row synthesis and its source-packet identities."""
import hashlib
import json
import sys
import tarfile
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
here = Path(__file__).resolve().parent
stage = here.parent
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((here / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-two-pending-public-v1"
assert {p.name for p in here.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, entry in manifest["files"].items():
    path = here / name
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    assert (len(raw), sha(raw)) == (entry["bytes"], entry["sha256"])
draft = json.loads((here / "DRAFT.json").read_bytes())
assert draft["schema"] == "726-post8b-next-two-central-draft-v1" and draft["status"].startswith("unadopted")
assert draft["parent"] == "8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b"
assert draft["source_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
assert draft["source_tree"] == "1eec97de340d9b7debbb30ca0c0fb2240a73b93f"
assert [g["index"] for g in draft["groups"]] == [1781, 3051]
assert [g["lens"] for g in draft["groups"]] == [12, 13]
assert draft["selected_rows"] == 2 and draft["accepted_unique_defect_increment"] == draft["closures"] == 0
prefix = "delegated/post8b-followups/"
packets = 0
for group in draft["groups"]:
    assert group["row"]["published_wrapper"] == "unassessed"
    for packet in group["packets"]:
        name = packet["path"]
        assert name.startswith(prefix)
        relative = Path(name[len(prefix):])
        assert not relative.is_absolute() and ".." not in relative.parts
        archive = stage / relative
        assert archive.is_file() and not archive.is_symlink()
        assert sha(archive.read_bytes()) == packet["archive_sha256"]
        with tarfile.open(archive, "r:gz") as tar:
            candidates = [m for m in tar.getmembers() if m.name.endswith("/MANIFEST.json")]
            assert len(candidates) == 1 and candidates[0].isfile()
            assert sha(tar.extractfile(candidates[0]).read()) == packet["original_manifest_sha256"]
        packets += 1
print(json.dumps({"rows": 2, "packet_identities": packets,
                  "qualification": "Identity and stated structure only; independent assembly review, adoption, semantic closure and product behavior are not established."}))
