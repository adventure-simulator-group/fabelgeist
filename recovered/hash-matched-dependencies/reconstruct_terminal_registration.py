#!/usr/bin/env python3
"""Recreate one historical registration without executing audit commands."""
import hashlib
import json
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("usage: reconstruct_terminal_registration.py NEW_DESTINATION")
base = Path(__file__).parent.parent / "transcript-hash-matched" / "REGISTRATION-TERMINAL-BOUNDARY-L13-3802.json"
item = json.loads(base.read_bytes())
item["at"] = "2026-11-09T00:26:06.923123+00:00"
item["purpose"] = "Inspect interpolation, bounded summation and intervention profile."
item["rows"] = item["rows"][:1]
item["rows"][0]["prospective_ranges"] = [[340, 455], [470, 530], [670, 740]]
blob = (json.dumps(item, indent=2) + "\n").replace("\n", "\r\n").encode("utf-8")
expected = "b61acadd3902437f19fcefdd26969ab4e87602e58131268e0444bc38d5f2361c"
if hashlib.sha256(blob).hexdigest() != expected:
    raise ValueError("Reconstructed registration hash mismatch")
with Path(sys.argv[1]).open("xb") as stream:
    stream.write(blob)
print("Reconstructed registration and verified SHA-256")
