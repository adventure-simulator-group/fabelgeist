#!/usr/bin/env python3
"""Read-only receipt and optional in-process no-key ciphertext checks."""
import argparse,hashlib,json,pathlib,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=pathlib.Path);ap.add_argument('--prior-inventory',type=pathlib.Path);a=ap.parse_args();assert not a.prior_inventory or a.ciphertexts
h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert oct(stat.S_IMODE(p.stat().st_mode))==m['root_mode'];actual={};dirs={}
for q in sorted(p.rglob('*')):
 assert not q.is_symlink();name=q.relative_to(p).as_posix();mode=oct(stat.S_IMODE(q.stat().st_mode))
 if q.is_dir():dirs[name]=mode
 elif q.is_file() and name!='MANIFEST.json':
  b=q.read_bytes();actual[name]={'bytes':len(b),'sha256':h(b),'mode':mode}
 else:assert q.is_file()
assert dirs=={x['path']:x['mode'] for x in m['directories']} and actual=={x['path']:{'bytes':x['bytes'],'sha256':x['sha256'],'mode':x['mode']} for x in m['files']}
r=json.loads((p/'RESTORATION.json').read_bytes());assert r['schema']=='726-post8b-second-restoration-v1' and r['restore_exit']==0 and r['collections']==53 and r['matched_file_bytes_and_modes'] and r['matched_directory_and_root_modes'] and r['source_before_after_equal'] and r['prior_nonce_count']==818 and r['new_nonce_count']==53 and r['cumulative_nonce_count']==871
status='receipt identity only'
if a.ciphertexts:
 root=a.ciphertexts;raw=(root/'MANIFEST.json').read_bytes();assert h(raw)==r['ciphertext_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-post8b-second-custody-encrypted-v1' and len(cm['objects'])==53 and cm['prior_nonce_count']==818 and cm['new_nonce_count']==53
 names=set();nonces=set()
 for i,row in enumerate(cm['objects'],1):
  label='post8b2-'+str(i).zfill(2);name=pathlib.PurePosixPath(row['ciphertext']);assert row['label']==label and len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names
  q=root/name;assert q.is_file() and not q.is_symlink();b=q.read_bytes();assert len(b)>=28 and (len(b),h(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and b[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name.name);nonces.add(b[:12])
 assert {x.name for x in root.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'};status='ciphertext identities checked without a key'
 if a.prior_inventory:
  raw=a.prior_inventory.read_bytes();assert h(raw)==r['prior_nonce_inventory_sha256'];old=json.loads(raw)['nonce_values_hex'];seen={bytes.fromhex(x) for x in old};assert len(old)==len(seen)==818 and all(len(x)==12 for x in seen) and not seen&nonces;status='ciphertext identities and 818 supplied prior nonces checked without a key'
print(json.dumps({'receipt':'matched','ciphertexts':status,'qualification':'Recorded restore/source comparison is not re-executed; same-machine and nonce-provenance limitations remain.'}))
