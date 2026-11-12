#!/usr/bin/env python3
"""Verify the captured same-machine recovery transcript and its qualification."""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent;d=json.loads((R/'RECOVERY-VERIFICATION-E0790A71.json').read_text());b=(R/d['evidence_only_log']).read_bytes()
assert hashlib.sha256(b).hexdigest()==d['log_sha256'] and len(b)==d['log_bytes'] and len(b.splitlines())==d['log_lines']==1117
assert b.rstrip().endswith(b'Published evidence checks passed; no product build, test or probe ran.') and d['published_commit']=='e0790a71589b583e7c459c2c21303accbb0ec1f0'
assert 'same-machine' in d['qualification'] and 'not independent-machine attestation' in d['qualification'] and d['product_validation']=='not performed'
print('Verified hash-linked same-machine recovery transcript; execution is not independently attested.')
