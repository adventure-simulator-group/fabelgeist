#!/usr/bin/env python3
"""Verify a qualified same-machine evidence recovery transcript."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parent;d=json.loads((r/'RECOVERY-VERIFICATION-A0D09CDB.json').read_text());b=(r/d['evidence_only_log']).read_bytes()
assert hashlib.sha256(b).hexdigest()==d['log_sha256'] and len(b)==d['log_bytes']==134867 and len(b.splitlines())==d['log_lines']==1574
assert b.rstrip().endswith(b'Published evidence checks passed for the qualified public reconstructions; retained private originals and previous Git history are separate. No product build, test or probe ran.')
assert d['published_commit']=='a0d09cdb771c526f42df398da2ad57d7c6e10520' and 'same-machine' in d['qualification'] and 'not independent-machine attestation' in d['qualification'] and d['product_validation']=='not performed'
print('Verified same-machine recovery transcript and its checkout-ID and history limitations.')
