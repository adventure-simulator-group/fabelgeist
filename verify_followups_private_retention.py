#!/usr/bin/env python3
"""Verify public custody receipt; optionally inspect retained private copies."""
import argparse,hashlib,json,stat
from pathlib import Path
r=Path(__file__).resolve().parent;d=json.loads((r/'FOLLOWUPS-PRIVATE-RETENTION.json').read_bytes());a=argparse.ArgumentParser();a.add_argument('--private-root',type=Path);args=a.parse_args();H=lambda b:hashlib.sha256(b).hexdigest()
assert len(d['collections'])==2 and d['temporary_custody']['current_files']==26 and d['product_validation']=='not performed'
c1=json.loads((r/'delegated/followup-40-41-qualified/CATALOG.json').read_bytes());c2=json.loads((r/'delegated/followup-42/CATALOG.json').read_bytes())
assert c1['private_retention_manifest_sha256']==d['collections'][0]['manifest_sha256'] and c2['private_retention_manifest_sha256']==d['collections'][1]['manifest_sha256']
incident=json.loads((r/'delegated/followup-42/TEMPORARY-CUSTODY-QUALIFICATION.json').read_bytes());assert incident['private_same_machine_capture_manifest_sha256']==d['temporary_custody']['manifest_sha256']
if args.private_root:
 for c in d['collections']:
  base=args.private_root/c['name'];raw=(base/'MANIFEST.json').read_bytes();assert H(raw)==c['manifest_sha256'];m=json.loads(raw);rows=m['files'];assert len(rows)==c['files'] and sum(x['bytes'] for x in rows)==c['bytes']
  assert not base.is_symlink() and stat.S_IMODE(base.stat().st_mode)&0o077==0
  expected={x['group']+'/'+x['path'] for x in rows};actual={f.relative_to(base).as_posix() for f in base.rglob('*') if f.is_file() and f!=base/'MANIFEST.json'};assert actual==expected
  for f in base.rglob('*'):assert not f.is_symlink() and stat.S_IMODE(f.stat().st_mode)&0o077==0
  for x in rows:
   b=(base/x['group']/x['path']).read_bytes();assert (H(b),len(b))==(x['sha256'],x['bytes'])
 c=d['temporary_custody'];base=args.private_root/c['name'];raw=(base/'MANIFEST.json').read_bytes();assert H(raw)==c['manifest_sha256'];m=json.loads(raw);assert len(m['items'])==c['current_files']
 assert not base.is_symlink() and stat.S_IMODE(base.stat().st_mode)&0o077==0
 for x in m['items']:
  f=base/x['retained_name'];assert not f.is_symlink() and stat.S_IMODE(f.stat().st_mode)&0o077==0;b=f.read_bytes();assert (H(b),len(b))==(x['observed_sha256'],x['observed_bytes'])
 print('Restricted retained copies and current temporary captures verified; earlier temporary contents remain unknown.')
else:print('Public receipt checked; private inputs were not supplied and their contents were not verified in this invocation.')
