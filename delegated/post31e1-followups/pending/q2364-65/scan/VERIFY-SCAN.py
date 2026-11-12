#!/usr/bin/env python3
import argparse,hashlib,json,pathlib
p=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--packet',type=pathlib.Path);a=ap.parse_args()
m=json.loads((p/'MANIFEST.json').read_text());entries=list(p.iterdir())
assert all(x.is_file() and not x.is_symlink() for x in entries)
assert {x.name for x in entries if x.name!='MANIFEST.json'}==set(m['files'])
for n,r in m['files'].items():
 assert pathlib.Path(n).name==n
 b=(p/n).read_bytes();assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
for n in ('VERIFY-STANDALONE','VERIFY-FULL','PRIVATE','PATTERN'):
 assert (p/(n+'.exit')).read_text()=='0' and (p/(n+'.stderr')).read_bytes()==b''
x=json.loads((p/'PRIVATE.stdout').read_text());y=json.loads((p/'PATTERN.stdout').read_text())
assert x['forbidden_digest_total']==0 and not x['other_candidate_counts'] and not x['exceptions']
assert not y['candidate_counts'] and not y['exceptions']
if a.packet:
 b=(a.packet/'MANIFEST.json').read_bytes();anchor=json.loads((p/'PACKET-ANCHOR.json').read_text())
 assert len(b)==anchor['packet_manifest_bytes'] and hashlib.sha256(b).hexdigest()==anchor['packet_manifest_sha256'] and len(json.loads(b)['files'])==anchor['packet_file_count']
 print('PASS receipt and packet manifest anchor; run packet verifier for packet contents.')
else:print('PASS receipt; packet anchor not repeated (supply --packet).')
print('LIMITS: scanner classes and views are bounded; private-input corpus and Git/transport replay depend on separately held same-machine inputs, not independent public custody.')
