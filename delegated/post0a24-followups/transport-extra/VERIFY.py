#!/usr/bin/env python3
"""Verify the extra pending public transport without executing product code."""
import hashlib,json,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P=Path(__file__).resolve().parent;m=json.loads((P/'MANIFEST.json').read_bytes());assert m['schema']=='726-pending-extra-transports-v1';expected={'MANIFEST.json','VERIFY.py'}
for row in m['files']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;p=P/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),hashlib.sha256(b).hexdigest())==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111);expected.add(row['path'])
assert {p.relative_to(P).as_posix() for p in P.rglob('*') if p.is_file() and not p.is_symlink()}==expected
print('Verified extra public transport bytes and Git executable bits; original private inputs and semantic review remain separate.')
