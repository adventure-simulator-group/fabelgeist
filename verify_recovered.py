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
print("Verified", count, "recovered evidence files; audit claims remain qualified")
