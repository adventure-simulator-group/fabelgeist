#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import hashlib,json
p=Path(__file__).resolve().parent;m=json.loads((p/'MANIFEST.json').read_bytes());files=set(m['files'])|{'MANIFEST.json'};actual={x.relative_to(p).as_posix() for x in p.rglob('*') if x.is_file()}
assert actual==files and all(not x.is_symlink() for x in p.rglob('*'))
for n,r in m['files'].items():
 q=PurePosixPath(n);assert not q.is_absolute() and '..' not in q.parts;f=p/n;b=f.read_bytes();assert (len(b),hashlib.sha256(b).hexdigest())==(r['bytes'],r['sha256'])
print(json.dumps({'members':len(m['files']),'qualification':'Public bytes only; no independent authenticity or source-semantic verification.'}))
