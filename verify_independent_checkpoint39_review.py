#!/usr/bin/env python3
"""Verify the public checkpoint 39 review packet only."""
import hashlib,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent;p=r/'delegated/independent-review-cp39'
assert hashlib.sha256((p/'MANIFEST.json').read_bytes()).hexdigest()=='d43811c881a95b528526f7cbc49745afd5978d516740b12cc58b1780ef01d8c9'
subprocess.run([sys.executable,str(p/'verify.py')],cwd=r,check=True)
print('Review packet verified; its input replay and prior temporary permissions remain separately qualified.')
