#!/usr/bin/env python3
"""Verify checkpoint 39 public evidence; private originals are separate."""
import hashlib,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent;p=r/'delegated/checkpoint39-stage'
assert hashlib.sha256((p/'MANIFEST.json').read_bytes()).hexdigest()=='f8f2bb8d08a3f3fcc404bc941ace2fde02bc103ca31c22fd6135b91e9b5abf08'
subprocess.run([sys.executable,str(p/'verify-stage.py')],cwd=r,check=True)
print('Public checkpoint 39 verified; original substitutions and local custody are separately qualified.')
