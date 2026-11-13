#!/usr/bin/env python3
"""Check or authenticate and restore encrypted audit evidence."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,io,json,os,stat,sys,tarfile
if sys.flags.optimize:raise RuntimeError('unoptimized Python required')
p=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--key',type=Path);ap.add_argument('--restore-root',type=Path);ap.add_argument('--prior-inventory',type=Path);a=ap.parse_args();assert not a.restore_root or a.key
sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-encrypted-private-evidence-v1' and m['algorithm'].startswith('AES-256-GCM')
rows=m['objects'];assert len(rows)==8;labels=set();names=set();nonces=set()
for row in rows:
 label=row['label'];assert label and label not in labels and all(c.isalnum() or c in '-_' for c in label);labels.add(label)
 name=PurePosixPath(row['ciphertext']);assert len(name.parts)==1 and name.name==label+'.aesgcm' and name.name not in names;names.add(name.name)
 f=p/name.name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),sha(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']) and len(b)>=28 and b[:12] not in nonces;nonces.add(b[:12])
 assert row['associated_data']=='726-audit-private-evidence:'+label+':v1'
assert {x.name for x in p.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'}
if a.prior_inventory:
 prior=json.loads(a.prior_inventory.read_bytes())['objects'];before={bytes.fromhex(x['nonce']) for x in prior};assert len(before)==len(prior)==638 and all(len(x)==12 for x in before) and not before&nonces
if not a.key:
 print('Verified ciphertext identities and new nonce uniqueness; authenticated restoration and prior-inventory provenance remain separate.')
 raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
assert a.key.is_file() and not a.key.is_symlink() and stat.S_IMODE(a.key.stat().st_mode)==0o600
key=a.key.read_bytes();assert len(key)==32
verified=[]
for row in rows:
 b=(p/row['ciphertext']).read_bytes();plain=AESGCM(key).decrypt(b[:12],b[12:],row['associated_data'].encode());assert (len(plain),sha(plain))==(row['plaintext_archive_bytes'],row['plaintext_archive_sha256'])
 seen=set();contents=[]
 with tarfile.open(fileobj=io.BytesIO(plain),mode='r:gz') as archive:
  for item in archive:
   name=PurePosixPath(item.name);assert not name.is_absolute() and name.parts and name.parts[0]=='evidence' and '..' not in name.parts and name.as_posix()==item.name.rstrip('/')
   assert name.as_posix() not in seen and (item.isfile() or item.isdir()) and 0<=item.mode<=0o7777;seen.add(name.as_posix())
   body=archive.extractfile(item).read() if item.isfile() else None
   if body is not None:assert len(body)==item.size
   contents.append((name,item.mode,body))
 dirs={n.as_posix() for n,_,body in contents if body is None};assert 'evidence' in dirs
 assert all(all(PurePosixPath(*n.parts[:i]).as_posix() in dirs for i in range(1,len(n.parts))) for n,_,_ in contents)
 assert sum(body is not None for _,_,body in contents)==row['regular_files'] and len(dirs)==row['directories'];verified.append((row['label'],contents))
if a.restore_root:
 root=a.restore_root;assert not root.exists() and not root.is_symlink();root.mkdir(mode=0o700)
 for label,contents in verified:
  dest=root/label;dest.mkdir(mode=0o700)
  dirs=sorted(((n,mode) for n,mode,body in contents if body is None),key=lambda x:(len(x[0].parts),x[0].as_posix()))
  for name,_ in dirs:dest.joinpath(*name.parts).mkdir(mode=0o700)
  for name,mode,body in contents:
   if body is not None:
    target=dest.joinpath(*name.parts)
    with target.open('xb') as f:f.write(body)
    target.chmod(mode)
  for name,mode in reversed(dirs):dest.joinpath(*name.parts).chmod(mode)
 print('Authenticated and restored all evidence into a newly created destination; no existing path was overwritten.')
else:print('Authenticated all plaintext archives in memory; no evidence was restored.')
print('Same-machine key custody, prior provenance, historical anchors and product behavior remain separate.')
