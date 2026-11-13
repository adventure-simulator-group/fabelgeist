#!/usr/bin/env python3
"""Verify registered public review manifests and their declared members."""
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
assert manifest["schema"] == "726-post8b-review-binding-public-v1"
assert {x.name for x in p.iterdir()} == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    f = p / name
    assert f.is_file() and not f.is_symlink()
    raw = f.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
binding = json.loads((p / "BINDING.json").read_bytes())
assert binding["schema"] == "726-post8b-review-binding-v1"
transport = p.parent / "later-transports"
map_raw = (transport / "ARCHIVE-MAP.json").read_bytes()
assert sha(map_raw) == binding["stage_map_sha256"]
archive_map = json.loads(map_raw)
assert archive_map["schema"] == "726-post8b-later-transport-map-v1"
assert len(binding["reviews"]) == 3
assert {x["label"] for x in binding["reviews"]} == {"five-v3", "five-v4", "five-v5"}

def safe_name(name):
    q = PurePosixPath(name)
    return bool(name) and not q.is_absolute() and str(q) == name and all(x not in ("", ".", "..") for x in q.parts)

declared_count = 0
for review in binding["reviews"]:
    label, archive_name = review["label"], review["archive"]
    assert safe_name(label) and "/" not in label and archive_name == label + ".tar.gz"
    matches = [r for r in archive_map["collections"] if r["label"] == label]
    assert len(matches) == 1
    row = matches[0]
    assert row["archive"] == archive_name
    assert row["original_manifest_sha256"] == review["original_manifest_sha256"]
    archive = transport / archive_name
    assert archive.is_file() and not archive.is_symlink()
    archive_raw = archive.read_bytes()
    assert (len(archive_raw), sha(archive_raw)) == (row["archive_bytes"], row["archive_sha256"])
    with tarfile.open(archive, "r:gz") as tf:
        members = tf.getmembers()
        assert len({m.name for m in members}) == len(members)
        assert all(safe_name(m.name) and (m.isfile() or m.isdir()) for m in members)
        by_name = {m.name: m for m in members}
        manifest_name = label + "/MANIFEST.json"
        assert manifest_name in by_name and by_name[manifest_name].isfile()
        original_raw = tf.extractfile(by_name[manifest_name]).read()
        assert sha(original_raw) == review["original_manifest_sha256"]
        mapped = [m for m in row["members"] if m["path"] == manifest_name]
        assert len(mapped) == 1 and mapped[0]["kind"] == "file"
        assert (mapped[0]["bytes"], mapped[0]["sha256"]) == (len(original_raw), sha(original_raw))
        original = json.loads(original_raw)
        assert isinstance(original["files"], dict) and "REPORT.md" in original["files"]
        expected = {label, manifest_name}
        for name, detail in original["files"].items():
            assert safe_name(name) and name != "MANIFEST.json"
            member_name = label + "/" + name
            assert member_name in by_name and by_name[member_name].isfile()
            content = tf.extractfile(by_name[member_name]).read()
            assert (len(content), sha(content)) == (detail["bytes"], detail["sha256"])
            expected.add(member_name)
            declared_count += 1
        assert set(by_name) == expected
print(json.dumps({"archives": 3, "declared_files": declared_count,
                  "qualification": "Identity only; checkpoint anchoring, independent review, source semantics and product behavior remain separate."}))
