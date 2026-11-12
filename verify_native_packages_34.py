#!/usr/bin/env python3
"""Verify selected archived package members against the pinned lock without execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads((R/p).read_text());cp=load('DELEGATED-EVIDENCE-CHECKPOINT-34.json');bs={x['bundle_id']:x for x in cp['bundles']};checked=[]
b=bs['reflect-metadata-2567-2570']
with tarfile.open(R/b['archive']) as t:
 reg=json.load(t.extractfile('registration.json'));lock=tomllib.loads(t.extractfile('sources/95e-Cargo.lock').read().decode());packs={}
 for name,x in reg['archives'].items():
  raw=t.extractfile(x['path']).read();z=next(y for y in lock['package'] if (y['name'],y['version'])==(name,x['version']));assert sha(raw)==x['sha256']==z['checksum'];packs[name]=raw;checked.append(name)
 for x in reg['entries']:
  if 'package' not in x:continue
  with tarfile.open(fileobj=io.BytesIO(packs[x['package']]),mode='r:gz') as pkg:raw=pkg.extractfile(x['member']).read()
  assert sha(raw)==x['sha256'] and raw==t.extractfile(x['archive']).read()
b=bs['validator-native-2587-2590']
with tarfile.open(R/b['archive']) as t:
 reg=json.load(t.extractfile('registration.json'));lock=tomllib.loads(t.extractfile('source/current-lock.lock').read().decode());packs={}
 for x in reg['packages']:
  raw=t.extractfile(x['archive']).read();z=next(y for y in lock['package'] if (y['name'],y['version'])==(x['name'],x['version']));assert sha(raw)==x['sha256']==z['checksum'];packs[x['name']+'-'+x['version']]=raw;checked.append(x['name'])
 for x in reg['entries']:
  if x.get('kind')!='pinned-cargo-native':continue
  with tarfile.open(fileobj=io.BytesIO(packs[x['package']]),mode='r:gz') as pkg:raw=pkg.extractfile(x['member']).read()
  assert sha(raw)==x['sha256'] and raw==t.extractfile(x['archive']).read()
 assert {x['role'] for x in reg['entries'] if x.get('kind')=='pinned-cargo-native'}=={2588,2589,2590}
 original=json.loads(gzip.decompress(t.extractfile('reference/registry.json.gz').read()));assert 'pinned-cargo' not in {x['id'] for x in original['snapshots']};roles=load('COVERAGE-L01-ROLES.json')['roles'];assert all(roles[i]['snapshot']=='pinned-cargo' for i in (2588,2589,2590))
assert len(checked)==5
print('Verified five lock-pinned native archives and selected members; missing original pinned-cargo snapshot-list entry remains qualified. No execution inferred.')
