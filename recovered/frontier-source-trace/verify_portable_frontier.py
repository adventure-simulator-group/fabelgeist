#!/usr/bin/env python3
"""Verify frontier source output and Git membership without a repository."""
import hashlib
import json
from pathlib import Path
import re
import tarfile

base = Path(__file__).resolve().parent
root = base.parent.parent
trace = json.loads((base / "TRACE.json").read_bytes())
tree_dir = root / "recovered/draft-change-source-archive"
manifest = json.loads((tree_dir / "MANIFEST.json").read_bytes())
archive_path = tree_dir / "objects.tar.gz"
if hashlib.sha256(archive_path.read_bytes()).hexdigest() != manifest["archive_sha256"]:
    raise ValueError("Git tree archive checksum mismatch")
revision = "ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04"
root_tree = manifest["roots"][revision]
objects = {}
with tarfile.open(archive_path, "r:gz") as archive:
    for member in archive:
        if member.name.startswith("tree/") or member.name == "commit/" + revision:
            if not member.isfile() or member.name in objects:
                raise ValueError("Unexpected Git tree object")
            inp = archive.extractfile(member)
            if inp is None:
                raise ValueError("Unreadable Git tree object")
            data = inp.read()
            kind, oid = member.name.split("/", 1)
            digest = hashlib.sha1(kind.encode() + b" " + str(len(data)).encode() + b"\0" + data).hexdigest()
            if digest != oid:
                raise ValueError("Git tree object checksum mismatch")
            objects[member.name] = data
if objects["commit/" + revision].split(b"\n", 1)[0] != b"tree " + root_tree.encode():
    raise ValueError("Selected revision root tree mismatch")

def tree_entry(oid, target):
    data = objects["tree/" + oid]
    pos = 0
    while pos < len(data):
        space = data.index(b" ", pos)
        end = data.index(b"\0", space + 1)
        if end + 21 > len(data):
            raise ValueError("Malformed Git tree")
        mode = int(data[pos:space], 8)
        name = data[space + 1:end]
        child = data[end + 1:end + 21].hex()
        if name == target:
            return mode, child
        pos = end + 21
    raise ValueError("Source path missing from Git tree")

ordinals = {item["read_ordinal"] for item in trace["sources"]}
records = {}
with (root / "recovered/late-command-history/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] in ordinals:
            records[item["transcript_ordinal"]] = item
count = 0
for item in trace["sources"]:
    if item["revision"] != revision:
        raise ValueError("Unexpected source revision")
    data = (base / "sources" / item["source_sha256"]).read_bytes()
    if (hashlib.sha256(data).hexdigest() != item["source_sha256"]
            or hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() != item["git_blob_sha1"]):
        raise ValueError("Frontier source checksum mismatch")
    parts = item["path"].encode("utf8").split(b"/")
    node = root_tree
    for index, part in enumerate(parts):
        mode, node = tree_entry(node, part)
        if index < len(parts) - 1 and mode != 0o40000:
            raise ValueError("Frontier source parent is not a tree")
        if index == len(parts) - 1 and (mode not in (0o100644, 0o100755, 0o120000) or node != item["git_blob_sha1"]):
            raise ValueError("Frontier source Git path mismatch")
    lines = data.decode("utf8").splitlines()
    record = records[item["read_ordinal"]]
    if record["recorded_exit_code"] != 0 or any(record["redactions"].values()):
        raise ValueError("Source output failed or was redacted")
    output = record["output_text"].replace("\r\r\n", "\n").replace("\r\n", "\n")
    if sum(x["read_ordinal"] == item["read_ordinal"] for x in trace["sources"]) > 1:
        marker = "\n " + item["path"] + "\n"
        start = output.find(marker)
        if start < 0:
            raise ValueError("Source output heading missing")
        output = output[start + len(marker):]
        end = output.find("\n crates/")
        if end >= 0:
            output = output[:end]
    printed = {int(m.group(1)): m.group(2) for m in re.finditer(r"(?m)^(\d+): (.*)$", output)}
    needed = {i for lo, hi in item["ranges"] for i in range(lo, hi + 1)}
    if not needed.issubset(printed) or any(printed[i] != lines[i - 1] for i in needed):
        raise ValueError("Frontier displayed source mismatch: " + item["path"] + " at " + str(item["read_ordinal"]))
    count += len(needed)
if len(trace["sources"]) != 12 or count != 755:
    raise ValueError("Frontier observation count mismatch")
print(f"Verified {len(trace['sources'])} frontier observations and {count} displayed lines without Git")
