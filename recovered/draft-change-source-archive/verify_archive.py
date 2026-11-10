#!/usr/bin/env python3
"""Verify draft Git trees and the complete set of changed source blobs."""
import hashlib
import json
from pathlib import Path
import tarfile

here = Path(__file__).resolve().parent
manifest = json.loads((here / "MANIFEST.json").read_bytes())
pinned = json.loads((here.parent / "draft-git-bundle/MANIFEST.json").read_bytes())
archive_path = here / "objects.tar.gz"
if (manifest["base"] != pinned["base_commit"] or manifest["drafts"] != pinned["drafts"]
        or hashlib.sha256(archive_path.read_bytes()).hexdigest() != manifest["archive_sha256"]):
    raise ValueError("Draft archive identity mismatch")
objects = {"tree": {}, "blob": {}, "commit": {}}
with tarfile.open(archive_path, "r:gz") as archive:
    for member in archive:
        parts = member.name.split("/")
        if (not member.isfile() or len(parts) != 2 or parts[0] not in objects
                or parts[1] in objects[parts[0]]):
            raise ValueError("Unexpected draft archive member")
        inp = archive.extractfile(member)
        if inp is None:
            raise ValueError("Unreadable draft archive member")
        data = inp.read()
        digest = hashlib.sha1(parts[0].encode() + b" " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if digest != parts[1]:
            raise ValueError("Draft Git object hash mismatch")
        objects[parts[0]][parts[1]] = data
if (len(objects["tree"]) != manifest["distinct_trees"]
        or len(objects["blob"]) != manifest["distinct_blobs"]
        or len(objects["commit"]) != manifest["commit_objects"]):
    raise ValueError("Draft Git object count mismatch")
revisions = [manifest["base"]] + [d["commit"] for d in manifest["drafts"]]
if set(objects["commit"]) != set(revisions) or set(manifest["roots"]) != set(revisions):
    raise ValueError("Draft revision inventory mismatch")
for rev in revisions:
    if objects["commit"][rev].split(b"\n", 1)[0] != b"tree " + manifest["roots"][rev].encode():
        raise ValueError("Draft root tree mismatch")

def flatten(tree_id, prefix=b""):
    data = objects["tree"][tree_id]
    result = {}
    pos = 0
    while pos < len(data):
        space = data.index(b" ", pos)
        end = data.index(b"\0", space + 1)
        mode = int(data[pos:space], 8)
        name = data[space + 1:end]
        if not name or b"/" in name or end + 21 > len(data):
            raise ValueError("Malformed Git tree")
        oid = data[end + 1:end + 21].hex()
        path = prefix + name
        if mode == 0o40000:
            nested = flatten(oid, path + b"/")
            if result.keys() & nested.keys():
                raise ValueError("Duplicate tree path")
            result.update(nested)
        else:
            if path in result:
                raise ValueError("Duplicate tree path")
            result[path] = (mode, oid)
        pos = end + 21
    return result

base = flatten(manifest["roots"][manifest["base"]])
rows = {}
for row in manifest["rows"]:
    key = (row["branch"], row["path"].encode("utf8", "surrogateescape"))
    if key in rows:
        raise ValueError("Duplicate draft change row")
    rows[key] = row
if len(rows) != manifest["path_rows"]:
    raise ValueError("Draft changed-path count mismatch")
used_blobs = set()
for draft in manifest["drafts"]:
    after = flatten(manifest["roots"][draft["commit"]])
    changed = {p for p in base.keys() | after.keys() if base.get(p) != after.get(p)}
    recorded = {p for branch, p in rows if branch == draft["branch"]}
    if changed != recorded:
        raise ValueError("Draft changes are not complete")
    for path in changed:
        row = rows[(draft["branch"], path)]
        for label, actual in (("before", base.get(path)), ("after", after.get(path))):
            node = row[label]
            if actual is None:
                if node is not None:
                    raise ValueError("Draft absence mismatch")
                continue
            if (node is None or node["type"] != "blob" or int(node["mode"], 8) != actual[0]
                    or node["oid"] != actual[1]):
                raise ValueError("Draft path and Git tree disagree")
            data = objects["blob"][node["oid"]]
            if hashlib.sha256(data).hexdigest() != node["sha256"]:
                raise ValueError("Draft source checksum mismatch")
            used_blobs.add(node["oid"])
if used_blobs != set(objects["blob"]):
    raise ValueError("Unexpected draft source objects")
print(f"Verified {len(manifest['drafts'])} draft trees, {len(rows)} changes and {len(used_blobs)} source blobs")
