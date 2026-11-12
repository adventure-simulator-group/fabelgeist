#!/usr/bin/env python3
"""Verify a frozen public follow-up evidence transport."""
import hashlib,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent;p=r/'delegated/followup-40-41-qualified'
assert hashlib.sha256((p/'STAGE-MANIFEST.json').read_bytes()).hexdigest()=='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15'
subprocess.run([sys.executable,'-B',str(p/'verify-archives.py')],cwd=r,check=True)
print('Public archive transport verified; private originals and earlier Git history remain separately qualified.')
