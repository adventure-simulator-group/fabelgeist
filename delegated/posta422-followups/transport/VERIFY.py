#!/usr/bin/env python3
"""Verify additive public evidence archives, file bytes and directory modes."""
from pathlib import Path,PurePosixPath
import hashlib,io,json,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-posta422-transports-v1' and m['parent_checkpoint']=='a4223bf23487b5872bc5efe5b272d40f859456f2' and m['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009';names={'MANIFEST.json','VERIFY.py'};total=0
for r in m['bundles']:
 assert r['file']==r['label']+'.tar.gz' and r['file'] not in names;names.add(r['file']);raw=(p/r['file']).read_bytes();assert (len(raw),h(raw))==(r['archive_bytes'],r['archive_sha256']);want={x['path']:x for x in r['members']};assert len(want)==len(r['members']);seen=set()
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
  for item in a:
   n=item.name.rstrip('/');q=PurePosixPath(n);assert not q.is_absolute() and '..' not in q.parts and n in want and n not in seen;seen.add(n);x=want[n];assert oct(item.mode)==x['mode']
   if item.isdir():assert x['type']=='directory'
   else:
    assert item.isfile() and x['type']=='file';b=a.extractfile(item).read();assert (len(b),h(b))==(x['bytes'],x['sha256'])
    if n==r['manifest_member']:assert h(b)==r['source_manifest_sha256']
 assert seen==set(want) and r['manifest_member'] in seen;total+=len(seen)
assert {x.name for x in p.iterdir()}==names
print(f'Verified {len(m["bundles"])} public archives and {total} members; restricted originals, anchors and central reconciliation remain separate.')
