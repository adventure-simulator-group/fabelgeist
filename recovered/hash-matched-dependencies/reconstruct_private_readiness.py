#!/usr/bin/env python3
"""Recreate a historical registration from pinned Git bytes and known time."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: reconstruct_private_readiness.py REPOSITORY NEW_DESTINATION")
repo = Path(sys.argv[1]).absolute()
revision = "ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04"
paths = [
    ("crates/adventuresim-stdb-module/src/disease.rs", [[730, 825], [1680, 1754]]),
    ("crates/adventuresim-core/src/disease.rs", [[1720, 1840]]),
    ("crates/adventuresim-core/src/physiology.rs", [[320, 410]]),
]
rows = []
for path, ranges in paths:
    data = subprocess.run(["git", "-C", str(repo), "show", revision + ":" + path],
                          check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    sha = hashlib.sha256(data).hexdigest()
    rows.append({"revision": revision, "git_path": path, "source_sha256": sha,
                 "actual_EOF": len(data.splitlines()),
                 "source_object": "adler-continuation-20261030/objects/" + sha,
                 "prospective_ranges": ranges})
item = {"schema": "726-prospective-current-source-registration-v1",
        "at": "2026-11-08T22:05:22.261001+00:00", "lenses": [13],
        "purpose": "Trace private terminal physiology and public illness projection relationship.",
        "rows": rows}
blob = (json.dumps(item, indent=2) + "\n").encode("utf-8")
expected = "b8c12ffd6952c0b25432be1c16f30139721c7037e39504939494a76c158fff29"
if hashlib.sha256(blob).hexdigest() != expected:
    raise ValueError("Reconstructed registration hash mismatch")
with Path(sys.argv[2]).open("xb") as stream:
    stream.write(blob)
print("Reconstructed registration and verified SHA-256")
