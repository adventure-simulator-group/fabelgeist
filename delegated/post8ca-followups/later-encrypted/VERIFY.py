#!/usr/bin/env python3
"""Verify ciphertext bytes or authenticate and restore encrypted evidence."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,io,json,sys,tarfile
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--key',type=Path);ap.add_argument('--restore-root',type=Path);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();assert not a.restore_root or a.key
h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-later-process-encrypted-v1' and len(m['objects'])==16 and m['prior_nonce_count']==687 and m['new_nonce_count']==16
names=set();nonces=set()
for row in m['objects']:
 label=row['label'];name=PurePosixPath(row['ciphertext']);assert label and all(c.isalnum() or c in '-_' for c in label) and len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names
 f=p/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert len(b)>=28 and (len(b),h(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and b[:12] not in nonces and row['associated_data']=='726-audit-private-evidence:'+label+':v1';names.add(name.name);nonces.add(b[:12])
assert {x.name for x in p.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'}
prior_status='not supplied; comparison to prior nonces not repeated'
if a.prior_inventory:
 x=json.loads(a.prior_inventory.read_bytes());old={bytes.fromhex(n) for n in x['nonce_values_hex']};assert len(x['nonce_values_hex'])==len(old)==687 and all(len(n)==12 for n in old) and not old&nonces;prior_status='distinct from supplied 687 values; inventory provenance remains separate'
if not a.key:
 print(json.dumps({'ciphertext_objects':len(names),'new_unique_nonces':len(nonces),'prior_check':prior_status,'qualification':'No-key ciphertext identities only; authentication, restoration and source comparison not repeated.'}));raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
assert a.key.is_file() and not a.key.is_symlink();key=a.key.read_bytes();assert len(key)==32
verified=[]
for row in m['objects']:
 blob=(p/row['ciphertext']).read_bytes();plain=AESGCM(key).decrypt(blob[:12],blob[12:],row['associated_data'].encode());assert (len(plain),h(plain))==(row['plaintext_archive_bytes'],row['plaintext_archive_sha256']);contents=[];seen=set()
 with tarfile.open(fileobj=io.BytesIO(plain),mode='r:gz') as tar:
  for item in tar.getmembers():
   q=PurePosixPath(item.name);assert not q.is_absolute() and q.parts and q.parts[0]=='evidence' and '..' not in q.parts and str(q)==item.name and item.name not in seen and (item.isfile() or item.isdir());seen.add(item.name)
   data=tar.extractfile(item).read() if item.isfile() else None
   if data is not None:assert len(data)==item.size
   contents.append((q,item.mode,data))
 assert ('evidence' in seen) and sum(x[2] is not None for x in contents)==row['regular_files'] and sum(x[2] is None for x in contents)==row['directories_including_root']
 for name,mode,data in contents:
  if str(name)!='evidence':assert str(name.parent) in seen
 verified.append((row['label'],contents))
if a.restore_root:
 root=a.restore_root;assert not root.exists() and not root.is_symlink();root.mkdir(mode=0o700)
 for label,contents in verified:
  dest=root/label;dest.mkdir(mode=0o700)
  dirs=sorted(((n,mode) for n,mode,data in contents if data is None),key=lambda x:len(x[0].parts))
  for n,mode in dirs:dest.joinpath(*n.parts).mkdir(mode=0o700)
  for n,mode,data in contents:
   if data is not None:
    f=dest.joinpath(*n.parts)
    with f.open('xb') as out:out.write(data)
    f.chmod(mode)
  for n,mode in reversed(dirs):dest.joinpath(*n.parts).chmod(mode)
 print(json.dumps({'authenticated_objects':len(verified),'restored_to_fresh_destination':True,'prior_check':prior_status,'qualification':'Archive authentication and fresh restore only; source comparison and custody independence remain separate.'}))
else:print(json.dumps({'authenticated_objects':len(verified),'restored_to_fresh_destination':False,'prior_check':prior_status,'qualification':'Authenticated archives in memory; source comparison and custody independence remain separate.'}))
