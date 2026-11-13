#!/usr/bin/env python3
"""Verify this receipt; optionally bind the ciphertext group and prior list."""
import argparse,hashlib,json,pathlib,stat,subprocess,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertexts',type=pathlib.Path);ap.add_argument('--prior-inventory',type=pathlib.Path);a=ap.parse_args();assert not a.prior_inventory or a.ciphertexts
m=json.loads((p/'MANIFEST.json').read_bytes());assert oct(stat.S_IMODE(p.stat().st_mode))==m['root_mode'];actual={};dirs={}
for q in sorted(p.rglob('*')):
 assert not q.is_symlink();name=q.relative_to(p).as_posix();mode=oct(stat.S_IMODE(q.stat().st_mode))
 if q.is_dir():dirs[name]=mode
 elif q.is_file() and name!='MANIFEST.json':
  b=q.read_bytes();actual[name]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':mode}
 else:assert q.is_file()
assert dirs=={x['path']:x['mode'] for x in m['directories']} and actual=={x['path']:{'bytes':x['bytes'],'sha256':x['sha256'],'mode':x['mode']} for x in m['files']}
r=json.loads((p/'RESTORATION.json').read_bytes());assert r['schema']=='726-post8b-second-restoration-v1' and r['restore_exit']==0 and r['collections']==53 and r['matched_file_bytes_and_modes'] and r['matched_directory_and_root_modes'] and r['source_before_after_equal'] and r['prior_nonce_count']==818 and r['new_nonce_count']==53 and r['cumulative_nonce_count']==871
result='receipt identity only'
if a.ciphertexts:
 q=a.ciphertexts;assert hashlib.sha256((q/'MANIFEST.json').read_bytes()).hexdigest()==r['ciphertext_manifest_sha256'];cmd=[sys.executable,'-B',str(q/'VERIFY.py')]
 if a.prior_inventory:cmd+=['--prior-inventory',str(a.prior_inventory)]
 x=subprocess.run(cmd,capture_output=True);assert x.returncode==0;result='ciphertext identities and optional prior nonces checked without a key'
print(json.dumps({'receipt':'matched','ciphertexts':result,'qualification':'Recorded restore/source comparison is not re-executed; same-machine and nonce-provenance limitations remain.'}))
