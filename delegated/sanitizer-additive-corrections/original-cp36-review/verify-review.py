#!/usr/bin/env python3
"""Verify the review packet itself without modifying it."""
from pathlib import Path
import hashlib, json
p=Path(__file__).resolve().parent
m=json.loads((p/'MANIFEST.json').read_bytes())
names={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}-{'MANIFEST.json'}
assert names==set(m['files'])
for name,x in m['files'].items():
    b=(p/name).read_bytes()
    assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256'],name
print('Review packet manifest verified. Historical snapshot bytes, original extraction and publication dependencies require the separately parameterized check.')
