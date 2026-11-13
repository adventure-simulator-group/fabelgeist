#!/usr/bin/env python3
"""Verify this review and optionally its exact five-group staged snapshot."""
import argparse, hashlib, json, stat
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--stage',type=Path);p.add_argument('--strict-mode',action='store_true');a=p.parse_args()
root=Path(__file__).resolve().parent
h=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'MANIFEST.json').read_bytes())
assert m['schema']=='726-package-review-v1'
assert set(x.name for x in root.iterdir())==set(m['files'])|{'MANIFEST.json'}
for name,row in m['files'].items():
 b=(root/name).read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
s=json.loads((root/'SNAPSHOT.json').read_bytes());assert s['schema']=='726-package-review-snapshot-v1'
if a.stage:
 actual={}
 for group in s['groups']:
  for path in (a.stage/group).rglob('*'):
   assert not path.is_symlink()
   if path.is_dir():continue
   assert path.is_file();actual[path.relative_to(a.stage).as_posix()]=path
 assert set(actual)=={x['path'] for x in s['files']}
 for row in s['files']:
  path=actual[row['path']];data=path.read_bytes();mode=stat.S_IMODE(path.stat().st_mode);expected=int(row['mode'],8)
  assert (len(data),h(data))==(row['bytes'],row['sha256'])
  assert mode==expected if a.strict_mode else mode&0o111==expected&0o111
 print('Verified public review and exact five-group snapshot:',len(actual),'files; other siblings excluded.')
else:print('Verified public review packet only; stage and restricted inputs were not checked.')
