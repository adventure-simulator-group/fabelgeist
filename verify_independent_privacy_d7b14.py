#!/usr/bin/env python3
"""Verify the public changed-commit privacy review packet only."""
import hashlib,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent;p=r/'delegated/independent-privacy-d7b14'
assert hashlib.sha256((p/'MANIFEST.json').read_bytes()).hexdigest()=='6ba6ac12c52c840a5350dcf94c914a0fb9571fb5cc6bb40d6d76f58933b4976f'
subprocess.run([sys.executable,str(p/'verify.py')],cwd=r,check=True)
print('Public review packet verified; prior history, private contexts, unscanned encodings and remote availability remain separate.')
