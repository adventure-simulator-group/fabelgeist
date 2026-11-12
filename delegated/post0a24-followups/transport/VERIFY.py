#!/usr/bin/env python3
"""Verify pending evidence transport identities without running product code."""
import hashlib,io,json,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P=Path(__file__).resolve().parent; H=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((P/'MANIFEST.json').read_bytes());assert m['schema']=='726-pending-public-transports-v1'
expected={'MANIFEST.json','VERIFY.py'}
for row in m['copied_files']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;f=P/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(f.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111);expected.add(row['path'])
for packet in m['packets']:
 f=P/packet['archive'];assert f.is_file() and not f.is_symlink();blob=f.read_bytes();assert (len(blob),H(blob))==(packet['bytes'],packet['sha256']);expected.add(packet['archive']);seen={}
 with tarfile.open(fileobj=io.BytesIO(blob),mode='r:gz') as t:
  for member in t.getmembers():
   n=PurePosixPath(member.name);assert member.isfile() and not n.is_absolute() and n.parts[0]=='evidence' and '..' not in n.parts and member.name not in seen;b=t.extractfile(member).read();seen[member.name]=(len(b),H(b),oct(member.mode))
 assert seen=={r['path']:(r['bytes'],r['sha256'],r['mode']) for r in packet['members']}
actual={x.relative_to(P).as_posix() for x in P.rglob('*') if x.is_file() and not x.is_symlink()};assert actual==expected
print('Verified pending public transport bytes, Git executable bits and archived member bytes/modes; original private inputs and semantic review remain separate.')
