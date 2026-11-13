#!/usr/bin/env python3
"""Check a bounded receipt and optional ciphertext bytes without a key."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=Path);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();assert not a.prior_inventory or a.ciphertexts
h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post8ca-process-public-receipt-v2' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for name,row in m['files'].items():
 f=p/name;assert f.is_file() and not f.is_symlink() and bool(f.stat().st_mode&0o100)==row['owner_executable'];b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
r=json.loads((p/'RESTORATION.json').read_bytes());bind=json.loads((p/'CIPHERTEXT-BINDING.json').read_bytes());scan=json.loads((p/'SCAN.json').read_bytes());assert r['ciphertext_manifest_sha256']==bind['ciphertext_manifest_sha256'] and (r['source_root_count'],r['control_root_count'],r['source_regular_files'],r['source_directories_including_roots'],r['control_regular_files'],r['control_directories_including_root'],r['prior_unique_nonce_count'],r['new_unique_nonce_count'])==(36,1,2303,208,6,1,650,37);assert r['source_inventory_before_after_matches'] and r['matched_file_bytes_and_modes'] and r['matched_directory_modes'] and r['matched_root_modes'] and all(x['exit']==0 and x['stderr_bytes']==0 for x in r['verification_checks']);assert scan['positive_controls_passed'] and scan['match_count']==0
count=0;prior_status='not supplied; prior comparison not repeated'
if a.ciphertexts:
 root=a.ciphertexts;assert root.is_dir() and not root.is_symlink();raw=(root/'MANIFEST.json').read_bytes();assert h(raw)==bind['ciphertext_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-post8ca-process-encrypted-v1' and len(cm['objects'])==37;names=set();nonces=set()
 for row in cm['objects']:
  name=PurePosixPath(row['ciphertext']);label=row['label'];assert len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names;f=root/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert len(b)>=28 and (len(b),h(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and b[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name.name);nonces.add(b[:12])
 for name,row in bind['auxiliary_metadata'].items():
  f=root/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
 assert {x.name for x in root.iterdir()}==names|set(bind['auxiliary_metadata'])|{'MANIFEST.json'};count=len(names)
 if a.prior_inventory:
  x=json.loads(a.prior_inventory.read_bytes());old={bytes.fromhex(n) for n in x['nonce_values_hex']};assert len(x['nonce_values_hex'])==len(old)==650 and all(len(n)==12 for n in old) and not old&nonces;prior_status='distinct from supplied 650 values; provenance remains separate'
print(json.dumps({'receipt_members':len(m['files']),'ciphertext_objects_checked':count,'prior_check':prior_status,'qualification':'Receipt and optional no-key ciphertext identities only; recorded restore is not repeated, and independent custody, prior-inventory provenance and product behavior remain separate.'}))
