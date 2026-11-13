#!/usr/bin/env python3
"""Check the additive review-chronology correction without altering proposals."""
from pathlib import Path
import hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();old=(p/'REVIEW-STATUS-v1.json').read_bytes();new=(p/'REVIEW-STATUS.json').read_bytes();x=json.loads(old);y=json.loads(new);c=json.loads((p/'REVIEW-STATUS-CORRECTION.json').read_bytes());assert h(old)==c['predecessor_sha256']==y['predecessor_status_sha256'] and h(new)==c['successor_sha256'] and set(y)-set(x)=={'predecessor_status_sha256','chronology_qualification'} and all(x[k]==y[k] for k in x if k!='status')
print('Verified preserved review-status predecessor and chronology correction; proposals and audit counts are unchanged.')
