#!/usr/bin/env python3
"""Read-only verification of this receipt and its bound encrypted payloads."""
import argparse,hashlib,json,stat,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
H=lambda b:hashlib.sha256(b).hexdigest()
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--ciphertext-dir',type=Path,required=True);q=a.parse_args();m=json.loads((p/'MANIFEST.json').read_bytes());files=m['files'];assert {x['path'] for x in files}=={x.name for x in p.iterdir() if x.name!='MANIFEST.json'}
for x in files:
 f=p/x['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(f.stat().st_mode)))==(x['bytes'],x['sha256'],x['mode'])
r=json.loads((p/'RESTORATION.json').read_bytes());assert r['restore_exit']==0 and r['selected_replay_exits']==[0] and (r['collections'],r['matched_file_bytes_and_modes'],r['matched_directory_modes'])==(24,1594,83)
c=q.ciphertext_dir;raw=(c/'MANIFEST.json').read_bytes();assert H(raw)==r['ciphertext_manifest_sha256'];doc=json.loads(raw);assert doc['schema']=='726-encrypted-private-evidence-v1' and len(doc['objects'])==24
for x in doc['objects']:
 f=c/x['ciphertext'];assert f.parent==c and f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b))==(x['ciphertext_bytes'],x['ciphertext_sha256'])
assert {x.name for x in c.iterdir()}=={x['ciphertext'] for x in doc['objects']}|{'MANIFEST.json','README.md','VERIFY.py'}
print('Receipt and 24 ciphertext byte identities match; authentication and restoration require the separately held key.')
