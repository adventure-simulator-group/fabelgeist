#!/usr/bin/env python3
"""Freeze the manifest of this fresh packet only."""
import pathlib,json,hashlib,datetime
P=pathlib.Path(__file__).resolve().parent;rows=[]
for f in sorted(P.rglob('*')):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':
  d=f.read_bytes();rows.append({'path':f.relative_to(P).as_posix(),'bytes':len(d),'sha256':hashlib.sha256(d).hexdigest()})
x={'schema':'726-xpbd-particles-reconcile-manifest-v1','author':'/root/hair_ops','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':rows,'qualification':'Restricted local source/evidence packet; original Git commit author metadata and local paths require privacy review before publication. Manifest excludes itself.'};b=(json.dumps(x,indent=2)+'\n').encode()
with (P/'MANIFEST.json').open('xb') as f:f.write(b)
(P/'MANIFEST.json').chmod(0o600)
print(json.dumps({'manifest_sha256':hashlib.sha256(b).hexdigest(),'files':len(rows)}))
