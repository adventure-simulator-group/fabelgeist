#!/usr/bin/env python3
"""Bounded privacy scan of this successor guide's plaintext files."""
from pathlib import Path
import json,re
r=Path(__file__).resolve().parent
names=['RECOVERY-GUIDE.md','RECOVERY-GUIDE-ADDENDUM.md','PRIVACY-CORRECTION.md','REPLAY-FOLLOWUPS.py','RESTORE-Q2812-ANCHORS.py','VERIFY-GUIDE.py','PRIVACY-SCAN.py']
email=rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+'+re.escape(b'.')+rb'[A-Za-z]{2,}'
sample=b'name'+b'@'+b'example'+b'.invalid'
if not re.search(email,sample):raise SystemExit('synthetic email pattern self-check failed')
patterns={'unix_local_home':b'/' + b'home' + b'/' + b'|' + b'/' + b'Users' + b'/','temporary_path':b'/' + b'tmp' + b'/' + b'|' + b'/' + b'private' + b'/' + b'tmp' + b'/','file_uri':b'file:' + b'/' + b'/','email_like':email,'private_key_pem':b'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----','common_api_key':b'(?i)sk-[A-Za-z0-9_-]{20,}'}
counts={}
for label,pat in patterns.items():
 counts[label]=sum(len(re.findall(pat,(r/n).read_bytes())) for n in names)
receipt={'scope':'Seven named plaintext successor files only; bounded regex patterns, no nested encrypted archives or private corpus. No claim about arbitrary secrets or local path values.','files':names,'synthetic_email_self_check':'PASS','matches':counts,'individual_removed_path_digests_included':False,'result':'PASS' if not any(counts.values()) else 'REVIEW'}
print(json.dumps(receipt,indent=2))
if any(counts.values()):raise SystemExit(1)
