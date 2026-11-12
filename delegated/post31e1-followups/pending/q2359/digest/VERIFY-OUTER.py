#!/usr/bin/env python3
import argparse,hashlib,json,pathlib
p=pathlib.Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--scan',type=pathlib.Path);a=ap.parse_args()
m=json.loads((p/'MANIFEST.json').read_text());entries=list(p.iterdir());assert all(x.is_file() and not x.is_symlink() for x in entries)
assert {x.name for x in entries if x.name!='MANIFEST.json'}==set(m['files'])
for n,v in m['files'].items():
 assert pathlib.Path(n).name==n
 b=(p/n).read_bytes();assert len(b)==v['bytes'] and hashlib.sha256(b).hexdigest()==v['sha256']
for n in ('SEAL-VERIFY','SEAL-PRIVATE','SEAL-PATTERN'):
 assert (p/(n+'.exit')).read_text()=='0' and (p/(n+'.stderr')).read_bytes()==b''
x=json.loads((p/'SEAL-PRIVATE.stdout').read_text());y=json.loads((p/'SEAL-PATTERN.stdout').read_text())
assert x['forbidden_digest_total']==0 and not x['other_candidate_counts'] and not x['exceptions']
assert not y['candidate_counts'] and not y['exceptions']
if a.scan:
 b=(a.scan/'MANIFEST.json').read_bytes();s=json.loads((p/'SCAN-ANCHOR.json').read_text());assert len(b)==s['scan_manifest_bytes'] and hashlib.sha256(b).hexdigest()==s['scan_manifest_sha256'];print('PASS outer receipt and scan seal anchor.')
else:print('PASS outer receipt; scan anchor not repeated (supply --scan).')
print('LIMITS: privacy scans are bounded; private corpus, Git objects and transport are separately held same-machine dependencies.')
