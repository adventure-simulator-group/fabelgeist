#!/usr/bin/env python3
"""No-key identity checks, or optional authentication and safe restore."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,io,json,stat,sys,tarfile
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--key',type=Path);ap.add_argument('--restore-root',type=Path);ap.add_argument('--prior-inventory',type=Path);ap.add_argument('--plaintext-inventory',type=Path);a=ap.parse_args();assert not a.restore_root or a.key;assert not a.plaintext_inventory or a.key
h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post8b-custody-encrypted-v1' and len(m['objects'])==46 and m['prior_nonce_count']==772 and m['new_nonce_count']==46
names=set();nonces=set()
for i,row in enumerate(m['objects'],1):
 label='post8b-'+str(i).zfill(2);name=PurePosixPath(row['ciphertext']);assert row['label']==label and len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names
 q=p/name;assert q.is_file() and not q.is_symlink();b=q.read_bytes();assert len(b)>=28 and (len(b),h(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and b[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name.name);nonces.add(b[:12])
assert {x.name for x in p.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'}
prior='not supplied; prior-nonce comparison not repeated'
if a.prior_inventory:
 old=json.loads(a.prior_inventory.read_bytes())['nonce_values_hex'];seen={bytes.fromhex(x) for x in old};assert len(old)==len(seen)==772 and all(len(x)==12 for x in seen) and not seen&nonces;prior='distinct from 772 supplied values; provenance and completeness not established'
if not a.key:
 print(json.dumps({'ciphertext_objects':len(names),'new_unique_nonces':len(nonces),'prior_check':prior,'qualification':'No-key ciphertext identities only; authentication, restoration and source comparison not repeated.'}));raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
st=a.key.lstat();assert stat.S_ISREG(st.st_mode) and not a.key.is_symlink();key=a.key.read_bytes();assert len(key)==32
pi=json.loads(a.plaintext_inventory.read_bytes())['objects'] if a.plaintext_inventory else None
if pi is not None:assert set(pi)=={x['label'] for x in m['objects']}
verified=[]
for row in m['objects']:
 blob=(p/row['ciphertext']).read_bytes();clear=AESGCM(key).decrypt(blob[:12],blob[12:],row['associated_data'].encode())
 if pi is not None:assert (len(clear),h(clear))==(pi[row['label']]['bytes'],pi[row['label']]['sha256'])
 contents=[];seen={}
 with tarfile.open(fileobj=io.BytesIO(clear),mode='r:gz') as t:
  for x in t.getmembers():
   q=PurePosixPath(x.name);assert not q.is_absolute() and q.parts and q.parts[0]=='evidence' and '..' not in q.parts and str(q)==x.name and x.name not in seen and (x.isfile() or x.isdir()) and 0<=x.mode<=0o7777
   data=t.extractfile(x).read() if x.isfile() else None
   if data is not None:assert len(data)==x.size
   seen[x.name]='file' if data is not None else 'dir';contents.append((q,x.mode,data))
 assert seen.get('evidence')=='dir' and sum(x[2] is not None for x in contents)==row['regular_files'] and sum(x[2] is None for x in contents)==row['directories_including_root']
 for name,mode,data in contents:
  if str(name)!='evidence':assert seen.get(str(name.parent))=='dir'
 verified.append((row['label'],contents))
if a.restore_root:
 root=a.restore_root;assert not root.exists() and not root.is_symlink();root.mkdir(mode=0o700)
 for label,contents in verified:
  dest=root/label;dest.mkdir(mode=0o700);dirs=sorted(((n,mode) for n,mode,data in contents if data is None),key=lambda x:len(x[0].parts))
  for n,mode in dirs:dest.joinpath(*n.parts).mkdir(mode=0o700)
  for n,mode,data in contents:
   if data is not None:
    q=dest.joinpath(*n.parts)
    with q.open('xb') as f:f.write(data)
    q.chmod(mode)
  for n,mode in reversed(dirs):dest.joinpath(*n.parts).chmod(mode)
 print(json.dumps({'authenticated_objects':len(verified),'restored_to_fresh_destination':True,'prior_check':prior,'qualification':'Authenticated archives and fresh restore only; source comparison and custody independence remain separate.'}))
else:print(json.dumps({'authenticated_objects':len(verified),'restored_to_fresh_destination':False,'prior_check':prior,'qualification':'Authenticated archives in memory; source comparison and custody independence remain separate.'}))
