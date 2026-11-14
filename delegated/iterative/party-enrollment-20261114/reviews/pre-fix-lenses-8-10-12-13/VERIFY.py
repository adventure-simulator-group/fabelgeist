#!/usr/bin/env python3
"""Check the complete flat public review against its manifest."""
import hashlib,json,stat
from pathlib import Path
p=Path(__file__).resolve().parent;m=json.loads((p/'MANIFEST.json').read_bytes())
assert stat.S_IMODE(p.stat().st_mode)==m['root_mode']
assert {x.name for x in p.iterdir()}=={'MANIFEST.json'}|{x['name'] for x in m['files']}
for r in m['files']:
 q=p/r['name'];assert q.is_file() and not q.is_symlink() and q.stat().st_size==r['bytes']
 assert hashlib.sha256(q.read_bytes()).hexdigest()==r['sha256'] and stat.S_IMODE(q.stat().st_mode)==r['mode']
print(json.dumps({'verified_files':len(m['files']),'scope':'public review identities only'}))
