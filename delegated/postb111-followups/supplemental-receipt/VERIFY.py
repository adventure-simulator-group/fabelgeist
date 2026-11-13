#!/usr/bin/env python3
"""Verify the public receipt and supplied encrypted-object identities."""
import argparse,hashlib,json,os,pathlib,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=argparse.ArgumentParser();p.add_argument('--ciphertexts',type=pathlib.Path,required=True);p.add_argument('--predecessor',type=pathlib.Path);p.add_argument('--git-checkout',type=pathlib.Path);p.add_argument('--primary',type=pathlib.Path);a=p.parse_args();r=pathlib.Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
def ck(v,m):
 if not v:raise ValueError(m)
receipt=json.loads((r/'RESTORATION.json').read_bytes());replay=json.loads((r/'REPLAYS.json').read_bytes());mb=(a.ciphertexts/'MANIFEST.json').read_bytes();ck(sha(mb)==receipt['ciphertext_manifest_sha256'],'ciphertext manifest');m=json.loads(mb);seen=set()
for x in m['objects']:
 p=a.ciphertexts/x['ciphertext'];ck(p.is_file() and not p.is_symlink(),'ciphertext type');b=p.read_bytes();ck(len(b)==x['ciphertext_bytes'] and sha(b)==x['ciphertext_sha256'],'ciphertext identity');ck(len(b)>=12 and b[:12] not in seen,'successor nonce reuse');seen.add(b[:12])
ck(len(m['objects'])==48 and all(x['exit']==0 and x['stderr_bytes']==0 for x in replay['checks']) and len(replay['checks'])==9,'receipt counts')
pred='not rechecked'
if a.predecessor:
 b=(a.predecessor/'MANIFEST.json').read_bytes();ck(sha(b)==receipt['predecessor_ciphertext_manifest_sha256'],'predecessor manifest');old=json.loads(b);ck(len(old['objects'])==46,'predecessor count')
 for x,y in zip(old['objects'],m['objects']):ck(x==y and (a.predecessor/x['ciphertext']).read_bytes()==(a.ciphertexts/y['ciphertext']).read_bytes(),'predecessor ciphertext changed')
 pred='verified'
external='not rechecked'
if a.git_checkout or a.primary:
 ck(bool(a.git_checkout) and bool(a.primary),'provide both external nonce scopes')
 names=[a.git_checkout/os.fsdecode(x) for x in subprocess.check_output(['git','ls-files','-z','--','*.aesgcm'],cwd=a.git_checkout,env=dict(os.environ,GIT_OPTIONAL_LOCKS='0')).split(b'\0') if x];primary=sorted(a.primary.glob('*.aesgcm'));ck(len(names)==420 and len(primary)==42,'external nonce scope count')
 for x in names+primary:
  with x.open('rb') as f:n=f.read(12)
  ck(len(n)==12 and n not in seen,'external nonce collision');seen.add(n)
 external='verified 420 tracked and 42 primary'
print(json.dumps({'result':'pass','ciphertexts':len(m['objects']),'predecessor':pred,'external_nonce_scope':external,'qualification':'Historical restoration and replay receipts only; fresh plaintext restoration still requires the separately held key.'},sort_keys=True))
