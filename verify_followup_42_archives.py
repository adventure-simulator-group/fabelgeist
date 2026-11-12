#!/usr/bin/env python3
"""Verify a frozen public follow-up evidence transport."""
import hashlib,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent;p=r/'delegated/followup-42'
assert hashlib.sha256((p/'STAGE-MANIFEST.json').read_bytes()).hexdigest()=='c6a849084f746adbcc392e46937928b624dbec52f8626aba3f6464527361e8b0'
subprocess.run([sys.executable,'-B',str(p/'verify-archives.py')],cwd=r,check=True)
print('Public archive transport verified; private originals and earlier Git history remain separately qualified.')
