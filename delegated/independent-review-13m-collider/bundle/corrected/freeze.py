#!/usr/bin/env python3
import hashlib,json,pathlib,datetime
p=pathlib.Path(__file__).resolve().parent
assert not (p/'MANIFEST.json').exists()
items=[]
for f in sorted(p.rglob('*')):
    assert not f.is_symlink()
    if f.is_file():
        b=f.read_bytes(); items.append({'path':f.relative_to(p).as_posix(),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
m={'schema':'726-final-13m-collider-review-manifest-v1','author':'/root/hair_ops','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':items,'qualification':'Bounded evidence review; original draft critique was conveyed before corrected review but formally frozen after the corrected bytes were displayed. No product validation.'}
with (p/'MANIFEST.json').open('x') as o: json.dump(m,o,indent=2); o.write('\n')
