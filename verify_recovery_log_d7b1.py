#!/usr/bin/env python3
"""Verify the qualified same-machine evidence recovery transcript."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parent;d=json.loads((r/'RECOVERY-VERIFICATION-D7B14A15.json').read_text());b=(r/d['evidence_only_log']).read_bytes()
assert hashlib.sha256(b).hexdigest()==d['log_sha256'] and len(b)==d['log_bytes']==133692 and len(b.splitlines())==d['log_lines']==1562
assert b.rstrip().endswith(b'Published evidence checks passed for the qualified public reconstructions; retained private originals and previous Git history are separate. No product build, test or probe ran.')
assert d['published_commit']=='d7b14a15ef132422eca86e7d898b47423af92779' and 'same-machine' in d['qualification'] and 'not independent-machine attestation' in d['qualification'] and d['product_validation']=='not performed'
print('Verified the same-machine recovery transcript and its checkout-ID and history limitations.')
