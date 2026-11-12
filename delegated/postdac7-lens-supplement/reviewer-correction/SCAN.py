#!/usr/bin/env python3
from pathlib import Path
import json,re
r=Path(__file__).resolve().parent
names=['README.md','CORRECTION.json','VERIFY.py','SCAN.py']
email=rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+'+re.escape(b'.')+rb'[A-Za-z]{2,}'
if not re.search(email,b'name'+b'@'+b'example'+b'.invalid'):raise SystemExit('email self-check')
patterns={'local_home':b'/'+b'home'+b'/|/'+b'Users'+b'/|/'+b'root'+b'/','temporary':b'/'+b'tmp'+b'/|/'+b'private/'+b'tmp'+b'/','file_uri':b'file:'+b'//','email_like':email,'private_key':b'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----','common_token':b'(?i)sk-[A-Za-z0-9_-]{20,}'}
counts={k:sum(len(re.findall(v,(r/n).read_bytes())) for n in names) for k,v in patterns.items()}
print(json.dumps({'scope':'Four named plaintext public correction files only; bounded patterns, no restricted corpus or nested archives, no arbitrary-secret guarantee.','files':names,'synthetic_email_self_check':'PASS','matches':counts,'individual_removed_path_digests_included':False,'result':'PASS' if not any(counts.values()) else 'REVIEW'},indent=2))
if any(counts.values()):raise SystemExit(1)
