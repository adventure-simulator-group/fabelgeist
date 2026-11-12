#!/usr/bin/env python3
"""Read-only integrity check of the public proposal packet."""
import hashlib,json
from pathlib import Path
p=Path(__file__).resolve().parent;m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-city-streaming-redaction-proposal-manifest-v1'
actual={str(f.relative_to(p)) for f in p.rglob('*') if f.is_file() and not f.is_symlink()}-{'MANIFEST.json'};assert actual==set(m['files'])
assert not any(f.is_symlink() for f in p.rglob('*'))
for name,x in m['files'].items():
 b=(p/name).read_bytes();assert hashlib.sha256(b).hexdigest()==x['sha256'] and len(b)==x['bytes'],name
print('Verified',len(actual),'public proposal files; manifest integrity does not establish availability of external publication inputs or private originals.')
