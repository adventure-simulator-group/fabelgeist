#!/usr/bin/env python3
"""Verify recovered evidence files without adopting their audit claims."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).parent
checks = [
    ("recovered/transcript-hash-matched", "sha256"),
    ("recovered/transcript-notes", "published_sha256"),
    ("recovered/transcript-queue-fragments", "normalized_json_sha256"),
]
count = 0
for relative, key in checks:
    directory = root / relative
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        name = item["file"]
        if Path(name).name != name:
            raise ValueError("Unsafe manifest filename")
        data = (directory / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != item[key]:
            raise ValueError("Hash mismatch: " + relative + "/" + name)
        count += 1
directory = root / "recovered/transcript-queue-fragments"
index = json.loads((directory / "EXTENDED-INDEX.json").read_text(encoding="utf-8"))
expected = {item["normalized_sha256"] for item in index["entries"]}
found = set()
with (directory / "PRINTED-VALUES.jsonl").open(encoding="utf-8") as stream:
    for line in stream:
        item = json.loads(line)
        data = (json.dumps(item["value"], indent=2, ensure_ascii=False) + "\n").encode("utf-8")
        sha = hashlib.sha256(data).hexdigest()
        if sha != item["normalized_sha256"] or sha in found:
            raise ValueError("Invalid or duplicate printed value: " + sha)
        found.add(sha)
if found != expected:
    raise ValueError("Printed value corpus and index differ")
print("Verified", count, "evidence files and", len(found), "printed values; claims remain qualified")
