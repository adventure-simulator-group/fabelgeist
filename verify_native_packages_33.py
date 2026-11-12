#!/usr/bin/env python3
"""Verify selected native package, lock and source identities only."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-33.json').read_text());bs={x['bundle_id']:x for x in cp['bundles']}
def lock(t,path,name,version,digest):
 d=tomllib.loads(t.extractfile(path).read().decode());p=next(x for x in d['package'] if (x['name'],x['version'])==(name,version));assert p['checksum']==digest
with tarfile.open(R/bs['reflect-opaque-common-2565-2566']['archive']) as t:
 r=json.load(t.extractfile('registration.json'));raw=t.extractfile(r['archive']['path']).read();h=sha(raw);assert h==r['archive']['sha256']
 for p in ('sources/6533-Cargo.lock','sources/95e-Cargo.lock'):lock(t,p,'bevy_reflect_derive','0.19.1',h)
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as pkg:
  for x in r['entries']:
   if x.get('kind')=='native-package-role':assert pkg.extractfile(x['member']).read()==t.extractfile(x['archive']).read() and sha(t.extractfile(x['archive']).read())==x['sha256']
with tarfile.open(R/bs['reflect-types-2571-2572']['archive']) as t:
 r=json.load(t.extractfile('registration.json'));raw=t.extractfile('native/bevy_reflect-0.19.1.crate').read();h=sha(raw);assert h==r['package_sha256']
 for p in ('support/current-lock.source','support/historical-lock-39.source','support/historical-lock-497.source'):lock(t,p,'bevy_reflect','0.19.1',h)
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as pkg:
  for x in r['entries']:
   if 'package_member' in x:
    v=x['versions'][0];body=pkg.extractfile(x['package_member']).read();assert body==t.extractfile(v['path']).read() and sha(body)==v['sha256']
with tarfile.open(R/bs['catalog-numeric-material-2583-2584']['archive']) as t:
 r=json.load(t.extractfile('registration.json'));packages={}
 for p in r['packages']:
  raw=t.extractfile(p['archive']).read();assert (sha(raw),len(raw))==(p['sha256'],p['bytes']);lock(t,'source/root.lock',p['name'],p['version'],p['sha256']);packages[p['name']+'-'+p['version']]=raw
 for x in r['entries']:
  if x.get('kind')=='native-package':
   with tarfile.open(fileobj=io.BytesIO(packages[x['package']]),mode='r:gz') as pkg:body=pkg.extractfile(x['member']).read();assert body==t.extractfile(x['archive']).read() and sha(body)==x['sha256']
print('Verified four lock-pinned native package archives and their selected source members; no feature resolution or execution.')
