#!/usr/bin/env python3
"""Check public custody identities; prior nonce inventory is optional."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=Path,required=True);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();h=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((p/'MANIFEST.json').read_bytes());assert manifest['schema']=='726-post76fe-public-custody-receipt-v1';rows=manifest['files'];assert set(rows)=={'DIAGNOSIS.md','REPLAY-RECORDS.json','RESTORATION.json','PUBLIC-SCAN.json','VERIFY-CUSTODY.py'};assert {x.name for x in p.iterdir()}==set(rows)|{'MANIFEST.json'}
for name,row in rows.items():
 q=PurePosixPath(name);assert len(q.parts)==1;f=p/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
raw=(a.ciphertexts/'MANIFEST.json').read_bytes();assert h(raw)==manifest['ciphertext_manifest_sha256'];m=json.loads(raw);assert m['schema']=='726-encrypted-private-evidence-v1' and len(m['objects'])==17
names={x['ciphertext'] for x in m['objects']};assert len(names)==17 and {x.name for x in a.ciphertexts.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'}
seen=set()
for x in m['objects']:
 q=PurePosixPath(x['ciphertext']);assert len(q.parts)==1 and q.name==x['label']+'.aesgcm';f=a.ciphertexts/q.name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();n=b[:12];assert (len(b),h(b))==(x['ciphertext_bytes'],x['ciphertext_sha256']) and len(n)==12 and n not in seen;assert x['associated_data']=='726-audit-private-evidence:'+x['label']+':v1';seen.add(n)
rest=json.loads((p/'RESTORATION.json').read_bytes());assert rest['ciphertext_manifest_sha256']==h(raw) and (rest['restore_exit'],rest['collections'],rest['matched_file_bytes_and_modes'],rest['matched_directory_modes_including_roots'])==(0,17,198,22)
checks=['17 ciphertext identities and mutually distinct nonces','recorded authenticated same-machine restoration']
if a.prior_inventory:
 old=json.loads(a.prior_inventory.read_bytes())['objects'];prior={bytes.fromhex(x['nonce_hex']) for x in old};assert len(old)==len(prior)==550 and not (prior & seen);checks.append('distinct from supplied 550-object nonce inventory')
else:checks.append('prior 550-object nonce inventory not supplied; cross-scope uniqueness not rechecked')
print(json.dumps({'checks':checks,'qualification':'No-key identity and recorded receipt only. Fresh decryption requires the separately held key; prior inventory provenance, Git objects, independent custody and product behavior remain separate.'}))
