#!/usr/bin/env python3
import pathlib,json,datetime,hashlib
P=pathlib.Path(__file__).resolve().parent;rows=[]
for f in sorted(P.rglob('*')):
 assert not f.is_symlink()
 if f.is_file() and f!=P/'MANIFEST.json':
  b=f.read_bytes();rows.append({'path':f.relative_to(P).as_posix(),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
x={'schema':'726-xpbd-support-attribution-addendum-manifest-v1','author':'/root/hair_ops','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':rows,'qualification':'Restricted evidence; possible local paths require privacy review prior to public transport; excludes itself.'};d=(json.dumps(x,indent=2)+'\n').encode()
with (P/'MANIFEST.json').open('xb') as f:f.write(d)
(P/'MANIFEST.json').chmod(0o600);print(json.dumps({'manifest_sha256':hashlib.sha256(d).hexdigest(),'files':len(rows)}))
