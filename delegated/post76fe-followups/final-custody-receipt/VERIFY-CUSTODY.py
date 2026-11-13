#!/usr/bin/env python3
"""Check public custody metadata and recorded same-machine restore results."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('unoptimized Python required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=Path,required=True);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();h=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post76fe-final-custody-receipt-v1'
assert set(m['files'])=={'RESTORATION.json','SCAN.json','VERIFY-CUSTODY.py'} and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for name,row in m['files'].items():
 assert len(PurePosixPath(name).parts)==1;f=p/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
root=a.ciphertexts;raw=(root/'MANIFEST.json').read_bytes();assert h(raw)==m['ciphertext_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-encrypted-private-evidence-v1' and len(cm['objects'])==8
seen=set();names=set()
for x in cm['objects']:
 n=x['ciphertext'];assert len(PurePosixPath(n).parts)==1 and n==x['label']+'.aesgcm' and n not in names;f=root/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(x['ciphertext_bytes'],x['ciphertext_sha256']) and len(b)>=28 and b[:12] not in seen and x['associated_data']=='726-audit-private-evidence:'+x['label']+':v1';names.add(n);seen.add(b[:12])
assert {x.name for x in root.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'} and h((root/'README.md').read_bytes())==m['ciphertext_readme_sha256'] and h((root/'VERIFY.py').read_bytes())==m['ciphertext_verifier_sha256']
rest=json.loads((p/'RESTORATION.json').read_bytes());assert rest['ciphertext_manifest_sha256']==h(raw) and (rest['identity_exit'],rest['nonce_replay_exit'],rest['restore_exit'],rest['compare_exit'],rest['collections'],rest['matched_file_bytes_and_modes'],rest['matched_directory_modes_including_roots'],rest['prior_nonces_checked'],rest['prior_nonce_duplicates'],rest['new_distinct_nonces'])==(0,0,0,0,8,514,45,638,0,8)
assert sum(x['regular_files'] for x in cm['objects'])==514 and sum(x['directories'] for x in cm['objects'])==45
scan=json.loads((p/'SCAN.json').read_bytes());assert scan['ciphertext_manifest_sha256']==h(raw) and scan['positive_controls_passed'] and scan['known_path_value_hits']==scan['known_individual_path_digest_hits']==0 and all(x==0 for x in scan['pattern_hit_counts'].values())
if a.prior_inventory:
 prior=json.loads(a.prior_inventory.read_bytes())['objects'];nonces={bytes.fromhex(x['nonce']) for x in prior};assert len(prior)==len(nonces)==638 and all(len(x)==12 for x in nonces) and not nonces&seen;prior_status='distinct from supplied 638-value inventory'
else:prior_status='prior inventory not supplied; cross-scope uniqueness not repeated'
print(json.dumps({'checks':['eight ciphertext identities and distinct new nonces','recorded authenticated same-machine restore and finite scan',prior_status],'qualification':'No-key checks do not repeat authenticated restoration or establish prior provenance, independent custody or product behavior.'}))
