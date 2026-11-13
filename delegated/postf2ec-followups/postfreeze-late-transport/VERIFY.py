#!/usr/bin/env python3
"""Verify exact public transport bytes and source manifests; no product work."""
import hashlib,io,json,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-postf2ec-postfreeze-late-transports-v1' and m['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and m['parent_checkpoint']=='f2ec8cbad7becbcc9a8e9da22ba512f52975768f';names={'MANIFEST.json','VERIFY.py'};total=0
for r in m['bundles']:
 assert r['file']==r['label']+'.tar.gz' and r['file'] not in names;names.add(r['file']);raw=(p/r['file']).read_bytes();assert (len(raw),h(raw))==(r['archive_bytes'],r['archive_sha256']);wanted={x['path']:x for x in r['members']};assert len(wanted)==len(r['members']);seen=set()
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
  for item in a:
   name=PurePosixPath(item.name);assert not name.is_absolute() and '..' not in name.parts and item.isfile() and item.name in wanted and item.name not in seen;seen.add(item.name);body=a.extractfile(item).read();row=wanted[item.name];assert (len(body),h(body),oct(item.mode))==(row['bytes'],row['sha256'],row['mode'])
   if item.name==r['manifest_member']:assert h(body)==r['source_manifest_sha256']
 assert seen==set(wanted) and r['manifest_member'] in seen;total+=len(seen)
assert {x.name for x in p.iterdir()}==names
print(f'Verified {len(m["bundles"])} exact transports and {total} members; restricted originals, historical anchors and central reconciliation remain separate.')
