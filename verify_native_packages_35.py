#!/usr/bin/env python3
"""Verify native SATS package sources and lock identity without executing code."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-35.json').read_text());b=next(x for x in cp['bundles'] if x['bundle_id']=='sats-macro-caller-2561-2562')
with tarfile.open(R/b['archive']) as t:
 reg=json.load(t.extractfile('registration.json'));locks=[tomllib.loads(t.extractfile(p).read().decode()) for p in ('sources/6533-Cargo.lock','sources/95e-Cargo.lock')];packs={}
 for name,x in reg['archives'].items():
  raw=t.extractfile(x['path']).read();assert sha(raw)==x['sha256']
  for lock in locks:
   z=next(y for y in lock['package'] if (y['name'],y['version'])==(name,x['version']));assert z['checksum']==sha(raw)
  packs[name]=raw
 for x in reg['entries']:
  if 'package' not in x:continue
  with tarfile.open(fileobj=io.BytesIO(packs[x['package']]),mode='r:gz') as pkg:raw=pkg.extractfile(x['member']).read()
  assert sha(raw)==x['sha256'] and raw==t.extractfile(x['archive']).read()
 assert len(packs)==3
print('Verified three SATS lock-pinned native archives and selected members; no feature resolution, macro or product execution inferred.')
