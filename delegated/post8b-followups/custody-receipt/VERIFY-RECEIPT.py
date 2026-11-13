#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('unoptimized')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=Path);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();assert not a.prior_inventory or a.ciphertexts;H=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post8b-public-receipt-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink() and bool(f.stat().st_mode&0o100)==row['owner_executable'];b=f.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
r=json.loads((p/'RESTORATION.json').read_bytes());bind=json.loads((p/'CIPHERTEXT-BINDING.json').read_bytes());s=json.loads((p/'SCAN.json').read_bytes());assert r['ciphertext_manifest_sha256']==bind['ciphertext_manifest_sha256'] and (r['source_root_count'],r['stage_snapshot_root_count'],r['control_root_count'],r['source_regular_files'],r['prior_unique_nonce_count'],r['new_unique_nonce_count'])==(44,1,1,2584,772,46);assert r['source_inventory_before_after_final_matches'] and r['matched_file_bytes_and_modes'] and r['matched_directory_modes'] and r['matched_root_modes'] and r['authentication_verifier_child_exit']==0 and len(r['verification_checks'])==5 and all(x['child_exit']==x['outer_wrapper_exit']==x['child_stderr_bytes']==x['outer_stderr_bytes']==0 for x in r['verification_checks']);assert s['positive_controls_pass'] and s['match_count']==0
count=0;prior='not supplied; prior comparison not repeated'
if a.ciphertexts:
 root=a.ciphertexts;assert root.is_dir() and not root.is_symlink();b=(root/'MANIFEST.json').read_bytes();assert H(b)==bind['ciphertext_manifest_sha256'];cm=json.loads(b);assert cm['schema']=='726-post8b-custody-encrypted-v1' and len(cm['objects'])==46 and cm['prior_nonce_count']==772 and cm['new_nonce_count']==46;names=set();nonces=set()
 for i,row in enumerate(cm['objects'],1):
  name=PurePosixPath(row['ciphertext']);label='post8b-'+str(i).zfill(2);assert row['label']==label and len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names;f=root/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert len(data)>=28 and (len(data),H(data))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and data[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name.name);nonces.add(data[:12])
 for n,row in bind['auxiliary_metadata'].items():
  f=root/n;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),H(data))==(row['bytes'],row['sha256'])
 assert {x.name for x in root.iterdir()}==names|set(bind['auxiliary_metadata'])|{'MANIFEST.json'};count=len(names)
 if a.prior_inventory:
  old=json.loads(a.prior_inventory.read_bytes())['nonce_values_hex'];oldset={bytes.fromhex(n) for n in old};assert len(old)==len(oldset)==772 and all(len(n)==12 for n in oldset) and not oldset&nonces;prior='distinct from 772 supplied values; prior completeness and provenance unproved'
print(json.dumps({'receipt_members':len(m['files']),'ciphertext_objects_checked':count,'prior_check':prior,'qualification':'No-key receipt and ciphertext identities only; authenticated restore is recorded, not repeated; independent custody remains separate.'}))
