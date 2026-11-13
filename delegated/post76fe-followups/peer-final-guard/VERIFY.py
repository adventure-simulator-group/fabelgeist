#!/usr/bin/env python3
"""Verify this bounded public peer report's file identities."""
from pathlib import Path
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('unoptimized Python required')
p=Path(__file__).resolve().parent
m=json.loads((p/'MANIFEST.json').read_bytes())
assert m['schema']=='726-post76fe-final-guard-independent-peer-v1'
assert set(m['files'])=={'REPORT.md','SCAN.json','VERIFY.py'}
assert {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,r in m['files'].items():
 f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes()
 assert (len(b),hashlib.sha256(b).hexdigest())==(r['bytes'],r['sha256'])
s=json.loads((p/'SCAN.json').read_bytes())
assert s['positive_controls_passed'] and all(v==0 for v in s['pattern_hits'].values())
assert s['known_path_value_hits']==s['known_path_digest_hits']==0
assert s['report_sha256']==hashlib.sha256((p/'REPORT.md').read_bytes()).hexdigest()
print('Verified public review and finite scan record; private raw evidence and custody are separate.')
