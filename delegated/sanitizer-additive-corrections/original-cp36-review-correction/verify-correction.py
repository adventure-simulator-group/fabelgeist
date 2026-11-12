#!/usr/bin/env python3
"""Verify the additive correction without modifying either packet."""
from pathlib import Path
import hashlib,json,sys
p=Path(__file__).resolve().parent
m=json.loads((p/'MANIFEST.json').read_bytes())
files={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}-{'MANIFEST.json'}
assert files==set(m['files'])
for name,x in m['files'].items():
    b=(p/name).read_bytes();assert (hashlib.sha256(b).hexdigest(),len(b))==(x['sha256'],x['bytes']),name
c=json.loads((p/'CORRECTION.json').read_bytes());v=json.loads((p/'verification-corrected.json').read_bytes());k='user_extraction_original_proposal_sha256';assert v[k]==c['correct_value']
if len(sys.argv)>1:
    old=Path(sys.argv[1]);o=c['original_review']
    for name,key in [('REPORT.json','report_sha256'),('MANIFEST.json','manifest_sha256'),('verification.json','verification_sha256'),('verify-observed.py','verifier_sha256')]:assert hashlib.sha256((old/name).read_bytes()).hexdigest()==o[key]
    before=json.loads((old/'verification.json').read_bytes());assert before[k]==c['incorrect_value'] and {x for x in before if before[x]!=v[x]}=={k}
print('Additive correction verified; original evidence is unchanged when supplied.')
