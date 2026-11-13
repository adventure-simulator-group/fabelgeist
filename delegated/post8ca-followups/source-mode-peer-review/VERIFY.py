#!/usr/bin/env python3
from pathlib import Path
import hashlib,json
p=Path(__file__).resolve().parent;m=json.loads((p/'MANIFEST.json').read_bytes());names=set(m['files'])|{'MANIFEST.json'}
assert {x.name for x in p.iterdir()}==names
for n,r in m['files'].items():
 f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),hashlib.sha256(b).hexdigest())==(r['bytes'],r['sha256'])
print(json.dumps({'members':len(m['files']),'qualification':'Public bytes only; no independent authenticity or restoration.'}))
