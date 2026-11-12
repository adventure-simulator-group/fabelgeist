#!/usr/bin/env python3
"""Verify this guide packet's file identities; not the protected evidence."""
from pathlib import Path
import hashlib,json,sys
p=Path(__file__).resolve().parent
if sys.flags.optimize:raise SystemExit('disable Python optimization')
m=json.loads((p/'MANIFEST.json').read_text());rows=m['files'];names={x['name'] for x in rows}
if len(rows)!=len(names) or {x.name for x in p.iterdir()}!=names|{'MANIFEST.json'}:raise SystemExit('file set mismatch')
for x in rows:
 f=p/x['name']
 if f.is_symlink() or not f.is_file():raise SystemExit('unsafe file')
 b=f.read_bytes()
 if len(b)!=x['bytes'] or hashlib.sha256(b).hexdigest()!=x['sha256']:raise SystemExit('identity mismatch: '+x['name'])
print('Guide packet file identities verified; consult both guide documents and the recorded bounded privacy scope.')
print('This does not authenticate the future encrypted collection, run a fresh private corpus scan, or establish product behavior or audit closure.')
