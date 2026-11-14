#!/usr/bin/env python3
"""Verify ciphertext identity, or authenticate and restore to a fresh directory."""
import argparse, hashlib, io, json, os, pathlib, stat, sys, tarfile
if sys.flags.optimize: raise RuntimeError('run without optimization')
if os.name != 'posix': raise RuntimeError('restoration requires POSIX path semantics')
a=argparse.ArgumentParser(); a.add_argument('--key',type=pathlib.Path); a.add_argument('--restore',type=pathlib.Path); z=a.parse_args()
if z.restore and not z.key: raise RuntimeError('restoration requires key')
p=pathlib.Path(__file__).resolve().parent
m=json.loads((p/'MANIFEST.json').read_bytes())
if m['schema']!='726-weather-snapshot-private-envelope-v1' or m['associated_data']!='726-weather-snapshot-slice-private-v1': raise RuntimeError('unexpected envelope')
c=p/'evidence.aesgcm'; s=c.lstat()
if not stat.S_ISREG(s.st_mode) or c.is_symlink(): raise RuntimeError('unexpected ciphertext')
if s.st_size < 28 or s.st_size >= 100000000: raise RuntimeError('ciphertext size limit')
b=c.read_bytes()
if len(b)!=m['ciphertext_bytes'] or hashlib.sha256(b).hexdigest()!=m['ciphertext_sha256'] or len(b)<28: raise RuntimeError('ciphertext identity mismatch')
if not z.key:
    print(json.dumps({'ciphertext_identity':True,'authenticated':False,'restored':False})); raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
ks=z.key.lstat()
if not stat.S_ISREG(ks.st_mode) or z.key.is_symlink(): raise RuntimeError('unexpected key')
k=z.key.read_bytes()
if len(k)!=32: raise RuntimeError('invalid key')
clear=AESGCM(k).decrypt(b[:12],b[12:],m['associated_data'].encode())
contents={}; total=0
with tarfile.open(fileobj=io.BytesIO(clear),mode='r:gz') as t:
    for x in t:
        q=pathlib.PurePosixPath(x.name)
        if '\\' in x.name or q.is_absolute() or not q.parts or '..' in q.parts or str(q)!=x.name or x.name in contents or not (x.isfile() or x.isdir()) or not 0<=x.mode<=0o7777: raise RuntimeError('unsafe archive entry')
        if len(contents)>=100001 or x.size>512*1024**2: raise RuntimeError('archive limit')
        data=t.extractfile(x).read() if x.isfile() else None
        if data is not None and len(data)!=x.size: raise RuntimeError('short archive member')
        total+=len(data) if data is not None else 0
        if total>512*1024**2: raise RuntimeError('archive byte limit')
        contents[x.name]=(x.mode,data)
if 'private-index.json' not in contents or contents['private-index.json'][1] is None: raise RuntimeError('missing index')
index=json.loads(contents['private-index.json'][1])
if index['schema']!='726-weather-private-index-v1' or len(index['entries'])!=m['entries']: raise RuntimeError('index mismatch')
expected={}
for x in index['entries']:
    name=x['path']
    if name in expected or name=='private-index.json': raise RuntimeError('duplicate index')
    expected[name]=x
if set(contents)!=set(expected)|{'private-index.json'}: raise RuntimeError('member set mismatch')
for name,row in expected.items():
    mode,data=contents[name]
    if mode!=row['mode'] or ('directory' if data is None else 'file')!=row['type']: raise RuntimeError('member mode/type mismatch')
    if data is not None and (len(data)!=row['bytes'] or hashlib.sha256(data).hexdigest()!=row['sha256']): raise RuntimeError('member identity mismatch')
    parent=str(pathlib.PurePosixPath(name).parent)
    if parent!='.' and (parent not in contents or contents[parent][1] is not None): raise RuntimeError('missing parent')
if z.restore:
    root=z.restore.absolute()
    if root.resolve(strict=False) != root: raise RuntimeError('symlinked or noncanonical restore ancestor')
    public=p.resolve()
    if root==public or root in public.parents or public in root.parents:
        raise RuntimeError('restore destination must be disjoint from public package')
    keydir=z.key.resolve().parent
    if root==keydir or root in keydir.parents or keydir in root.parents:
        raise RuntimeError('restore destination must be disjoint from key')
    registration=json.loads(contents['registration.json'][1])
    for item in registration['sources']:
        source=pathlib.Path(item['path']).resolve(strict=False)
        if root==source or root in source.parents or source in root.parents:
            raise RuntimeError('restore destination must be disjoint from sources')
    if root.exists() or root.is_symlink(): raise RuntimeError('fresh destination required')
    root.mkdir(mode=0o700)
    dirs=sorted(((n,v) for n,v in contents.items() if v[1] is None),key=lambda a:len(pathlib.PurePosixPath(a[0]).parts))
    for name,(mode,_) in dirs: (root/name).mkdir(mode=0o700)
    for name,(mode,data) in contents.items():
        if data is not None:
            q=root/name
            with q.open('xb') as f: f.write(data)
            q.chmod(mode)
    for name,(mode,_) in reversed(dirs): (root/name).chmod(mode)
print(json.dumps({'ciphertext_identity':True,'authenticated':True,'members_verified':len(expected),'private_index_sha256':hashlib.sha256(contents['private-index.json'][1]).hexdigest(),'restored':bool(z.restore),'qualification':'Authenticated contents and fresh restore; comparison to sources and independent custody are separate.'}))
