#!/usr/bin/env python3
"""Check original receipt/ciphertext identities using Git mode semantics."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('unoptimized Python required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--receipt',type=Path,required=True);ap.add_argument('--ciphertexts',type=Path,required=True);ap.add_argument('--final-scan',type=Path,required=True);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();own=json.loads((p/'MANIFEST.json').read_bytes());assert own['schema']=='726-post8ca-source-custody-git-v1' and set(own['files'])=={'README.md','VERIFY.py','BINDING.json'} and {x.name for x in p.iterdir()}==set(own['files'])|{'MANIFEST.json'}
for name,row in own['files'].items():
 f=p/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),h(data))==(row['bytes'],row['sha256'])
binding=json.loads((p/'BINDING.json').read_bytes());assert binding['schema']=='726-post8ca-source-custody-git-binding-v1'
receipt=a.receipt;assert receipt.is_dir() and not receipt.is_symlink();raw=(receipt/'MANIFEST.json').read_bytes();assert h(raw)==binding['receipt_manifest_sha256'];m=json.loads(raw);rows={x['path']:x for x in m['files']};assert len(rows)==len(m['files'])==3 and set(rows)=={'RESTORATION.json','SCAN.json','VERIFY-CUSTODY.py'} and {x.name for x in receipt.iterdir()}==set(rows)|{'MANIFEST.json'}
for name,row in rows.items():
 assert len(PurePosixPath(name).parts)==1;f=receipt/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),h(data))==(row['bytes'],row['sha256']) and bool(f.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111)
root=a.ciphertexts;assert root.is_dir() and not root.is_symlink() and a.final_scan.is_file() and not a.final_scan.is_symlink();manifest=(root/'MANIFEST.json').read_bytes();assert h(manifest)==binding['ciphertext_manifest_sha256'];cm=json.loads(manifest);assert cm['schema']=='726-encrypted-private-evidence-v1' and len(cm['objects'])==4;names=set();nonces=set()
for row in cm['objects']:
 name=row['ciphertext'];label=row['label'];assert len(PurePosixPath(name).parts)==1 and name==label+'.aesgcm' and name not in names;f=root/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),h(data))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and len(data)>=28 and data[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name);nonces.add(data[:12])
assert {x.name for x in root.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'}
r=json.loads((receipt/'RESTORATION.json').read_bytes());assert r['ciphertext_manifest_sha256']==h(manifest) and r['restore_exit']==0 and r['source_root_count']==4 and r['matched_regular_files']==1767 and r['matched_directories_including_roots']==94 and r['matched_file_bytes_and_modes'] and r['matched_directory_modes'] and r['matched_root_modes'] and r['source_inventory_before_and_after_matches'] and (r['prior_unique_nonce_count'],r['new_unique_nonce_count'])==(646,4) and r['all_nonces_distinct'];assert {x['label'] for x in r['selected_replay_checks']}=={'q185-peer-strong','q3100-author-strong','q3100-peer-strong'} and all(x['exit']==0 and x['stderr_bytes']==0 for x in r['selected_replay_checks'])
for rawscan in ((receipt/'SCAN.json').read_bytes(),a.final_scan.read_bytes()):
 s=json.loads(rawscan);assert s['ciphertext_manifest_sha256']==h(manifest) and s['positive_controls_pass'] and s['known_path_candidates']==5199 and s['private_source_literal_hits']==s['individual_path_digest_hits']==0 and not any(s['pattern_counts'].values())
assert h(a.final_scan.read_bytes())==binding['final_scan_sha256'];prior_status='prior inventory not supplied; cross-scope nonce uniqueness not repeated'
if a.prior_inventory:
 prior=json.loads(a.prior_inventory.read_bytes())['objects'];values={bytes.fromhex(x['nonce']) for x in prior};assert len(values)==len(prior)==646 and all(len(x)==12 for x in values) and not values&nonces;prior_status='distinct from supplied 646-value inventory; provenance remains separate'
print(json.dumps({'receipt_files':len(rows),'ciphertext_objects':len(names),'new_nonces_distinct':len(nonces),'prior_check':prior_status,'qualification':'Checks identities and Git executable bits plus recorded restore/scan results; does not enforce original full receipt modes, repeat authenticated restore, establish independent custody or validate product behavior.'}))
