#!/usr/bin/env python3
"""Recreate four transcript-derived upstream source-copy candidates."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: restore_upstream.py REPOSITORY DESTINATION")
repo = Path(sys.argv[1]).absolute()
if not repo.is_dir():
    raise ValueError("Repository directory does not exist")
dest = Path(sys.argv[2]).absolute()
manifest = json.loads((Path(__file__).parent / "MANIFEST.json").read_text(encoding="utf-8"))
dest.mkdir(parents=False, exist_ok=False)
marker = dest / "RECOVERY-INCOMPLETE"
marker.write_text("Source recovery did not complete.\n", encoding="utf-8")
for item in manifest:
    name = item["historical_name"]
    if Path(name).name != name:
        raise ValueError("Invalid destination name")
    source = subprocess.run(
        ["git", "-C", str(repo), "show", item["revision"] + ":" + item["git_path"]],
        check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    ).stdout
    blob = hashlib.sha1(b"blob " + str(len(source)).encode() + b"\0" + source).hexdigest()
    if hashlib.sha256(source).hexdigest() != item["source_sha256"] or blob != item["source_git_blob_sha1"]:
        raise ValueError("Upstream source identity mismatch: " + name)
    candidate = source + b"\n"
    if hashlib.sha256(candidate).hexdigest() != item["candidate_sha256"]:
        raise ValueError("Candidate hash mismatch: " + name)
    (dest / name).write_bytes(candidate)
marker.unlink()
print("Verified and restored", len(manifest), "upstream source candidates")
