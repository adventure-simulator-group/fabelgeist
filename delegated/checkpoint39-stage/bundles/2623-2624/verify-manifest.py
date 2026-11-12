#!/usr/bin/env python3
"""Read-only manifest verification."""
from pathlib import Path
import hashlib,json
b=Path(__file__).resolve().parent;m=json.loads((b/'MANIFEST.json').read_text());actual={str(p.relative_to(b)) for p in b.rglob('*') if p.is_file() and p.name!='MANIFEST.json'};listed={x['path'] for x in m['files']};assert actual==listed and len(listed)==len(m['files'])
for x in m['files']:
 d=(b/x['path']).read_bytes();assert len(d)==x['bytes'] and hashlib.sha256(d).hexdigest()==x['sha256']
print(json.dumps({'status':'passed','files':len(listed),'writes':'none'},indent=2))
