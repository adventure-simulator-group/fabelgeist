#!/usr/bin/env python3
"""Packaging integrity only; runs no repository code or project checks."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
manifest=json.loads((ROOT/"MANIFEST.json").read_text());bad=[]
for item in manifest["files"]:
    path=ROOT/item["path"]
    if not path.is_file():bad.append(item["path"]+": missing");continue
    h=hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda:f.read(1024*1024),b""):h.update(block)
    if path.stat().st_size!=item["bytes"] or h.hexdigest()!=item["sha256"]:bad.append(item["path"]+": hash/size mismatch")
print(json.dumps({"files":len(manifest["files"]),"errors":bad},indent=2))
raise SystemExit(bool(bad))
