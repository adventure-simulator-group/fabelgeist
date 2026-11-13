#!/usr/bin/env python3
"""Verify the bounded public review package against its sealed manifest."""
from pathlib import Path
import hashlib,json,sys
p=Path(__file__).resolve().parent
m=json.loads((p/'MANIFEST.json').read_bytes())
expected=set(m['files'])|{'MANIFEST.json'}
actual={x.name for x in p.iterdir()}
assert actual==expected and all((p/n).is_file() and not (p/n).is_symlink() for n in expected)
for name,row in m['files'].items():
 b=(p/name).read_bytes();assert len(b)==row['bytes'] and hashlib.sha256(b).hexdigest()==row['sha256']
binding=json.loads((p/'BINDING.json').read_bytes())
assert binding['draft_sha256']=='615e914ede1c9c389790e064ab9cb76ba262dd6dd72b5e04942d3baeb317b4c6'
print(json.dumps({'verified_members':len(m['files']),'draft_sha256':binding['draft_sha256'],'qualification':'Public bytes only; no external authenticity or source-semantic verification.'}))
