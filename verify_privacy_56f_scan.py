#!/usr/bin/env python3
"""Verify the bounded public privacy-scan transport, not private custody."""
import hashlib, subprocess, sys
from pathlib import Path
root = Path(__file__).resolve().parent
stage = root / "delegated/privacy-56f-scan"
expected = "f9d81b3c76cb1c87a709a7438567c0b367ad2ce2dae4a88e1183c8112395fffe"
assert hashlib.sha256((stage / "PUBLIC-MANIFEST.json").read_bytes()).hexdigest() == expected
subprocess.run([sys.executable, str(stage / "verify-public-transport.py")], cwd=root, check=True)
print("Public scan transport verified; private originals and previously published history remain separate.")
