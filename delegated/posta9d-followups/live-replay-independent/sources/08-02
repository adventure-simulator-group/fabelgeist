#!/usr/bin/env python3
"""Bind a qualified earlier evidence-only replay transcript."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
p=root/'delegated/post26e-optional-followup-check'
h=lambda b:hashlib.sha256(b).hexdigest()
r=json.loads((p/'RECORD.json').read_bytes())
b=(p/r['transcript']).read_bytes()
assert r['exit']==0 and (len(b),h(b))==(r['transcript_bytes'],r['transcript_sha256'])
assert h((root/r['verifier']).read_bytes())==r['verifier_sha256']
s=json.loads((p/'SCAN.json').read_bytes())
assert s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
print('Verified qualified optional evidence replay transcript; input identities and independent custody remain separate.')
