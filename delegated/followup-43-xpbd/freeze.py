#!/usr/bin/env python3
"""Freeze the manifest of this newly created public stage only."""
import pathlib,json,hashlib,datetime
P=pathlib.Path(__file__).resolve().parent;rows=[]
for f in sorted(P.rglob('*')):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':
  b=f.read_bytes();rows.append({'path':f.relative_to(P).as_posix(),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
x={'schema':'726-xpbd-public-transport-manifest-v1','author':'/root/hair_ops','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':rows,'qualification':'Public stage bytes; original and reconstructed identities remain separate. Excludes this manifest.'};b=(json.dumps(x,indent=2)+'\n').encode()
with (P/'MANIFEST.json').open('xb') as f:f.write(b)
print(json.dumps({'manifest_sha256':hashlib.sha256(b).hexdigest(),'files':len(rows)}))
