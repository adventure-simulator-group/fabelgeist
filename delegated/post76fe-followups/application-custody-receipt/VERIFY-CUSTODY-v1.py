#!/usr/bin/env python3
"""Check public custody metadata, ciphertext identities and recorded results."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--ciphertexts',type=Path,required=True);a.add_argument('--prior-inventory',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post76fe-application-receipt-v1'
assert set(m['files'])=={'RESTORATION.json','SCAN.json','SCAN-v2.json','VERIFY-CUSTODY.py'} and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,r in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(r['bytes'],r['sha256'])
root=args.ciphertexts;raw=(root/'MANIFEST.json').read_bytes();assert h(raw)==m['ciphertext_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-encrypted-private-evidence-v1' and len(cm['objects'])==12;seen=set();names=set()
for x in cm['objects']:
 n=x['ciphertext'];assert len(PurePosixPath(n).parts)==1 and n==x['label']+'.aesgcm' and n not in names;f=root/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();nonce=b[:12];assert (len(b),h(b))==(x['ciphertext_bytes'],x['ciphertext_sha256']) and len(nonce)==12 and nonce not in seen;assert x['associated_data']=='726-audit-private-evidence:'+x['label']+':v1';seen.add(nonce);names.add(n)
assert {x.name for x in root.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'};assert h((root/'README.md').read_bytes())==m['ciphertext_readme_sha256'] and h((root/'VERIFY.py').read_bytes())==m['ciphertext_verifier_sha256']
rest=json.loads((p/'RESTORATION.json').read_bytes());assert rest['ciphertext_manifest_sha256']==h(raw) and (rest['identity_exit'],rest['restore_exit'],rest['collections'],rest['matched_file_bytes_and_modes'],rest['matched_directory_modes_including_roots'],rest['prior_nonces_checked'],rest['prior_nonce_duplicates'],rest['new_distinct_nonces'])==(0,0,12,109,20,626,0,12);assert all(x['exit']==0 and x['stderr_bytes']==0 for x in rest['selected_replay_checks']) and len(rest['selected_replay_checks'])==2
for name,candidates in [('SCAN.json',25),('SCAN-v2.json',667)]:
 s=json.loads((p/name).read_bytes());assert s['positive_controls_passed'] and s['known_path_candidates']==candidates and s['known_path_value_hits']==s['known_individual_path_digest_hits']==0 and all(x==0 for x in s['pattern_hit_counts'].values())
checks=['12 ciphertext identities and distinct new nonces','recorded authenticated same-machine restoration and bounded scan']
if args.prior_inventory:
 old=json.loads(args.prior_inventory.read_bytes())['objects'];values={bytes.fromhex(x['nonce_hex']) for x in old};assert len(old)==len(values)==626 and not values&seen;checks.append('distinct from supplied 626-object nonce inventory')
else:checks.append('prior 626-object inventory not supplied; cross-scope uniqueness not rechecked')
print(json.dumps({'checks':checks,'qualification':'No-key identity check and recorded results; same-machine key, prior inventory provenance, later controls, external history and independent custody remain separate.'}))
