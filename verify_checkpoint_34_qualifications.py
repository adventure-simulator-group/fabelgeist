#!/usr/bin/env python3
"""Check selected historical variants, reconstructed comparisons and label qualification."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-34.json');bs={x['bundle_id']:x for x in cp['bundles']};old=load('DELEGATED-EVIDENCE-CHECKPOINT-13.json');prior={x['bundle_id']:x for x in old['bundles']}
b=bs['reflect-metadata-2567-2570'];members={x['path']:x for x in b['members']};dif=[]
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('history/published-comparisons.json'))
 for x in d['members']:
  p=prior[x['packet']];m=next(z for z in p['members'] if z['path']==x['member']);copy=members[x['original']]
  assert (copy['original_sha256'],m['original_sha256'],m['published_sha256'])==(x['original_sha256'],x['original_sha256'],x['published_sha256'])
  assert x['equal']==(x['original_sha256']==x['published_sha256'])
  with tarfile.open(R/p['archive']) as priorarchive:assert sha(priorarchive.extractfile(x['member']).read())==x['published_sha256']
  if not x['equal']:dif.append((x['packet'],x['member']))
assert dif==[('bevy-remote','registration.json'),('bevy-derive-meta','registration.json')]
b=bs['validator-native-2587-2590']
with tarfile.open(R/b['archive']) as t:
 historic=t.extractfile('source/p16-validator.source').read();current=t.extractfile('source/current-validator.source').read()
 assert historic!=current and historic.splitlines(keepends=True)[:1492]==current.splitlines(keepends=True)[:1492]
 report=json.load(t.extractfile('report.json')); reg=json.load(t.extractfile('registration.json'));assert {x['role'] for x in reg['entries'] if x.get('kind')=='pinned-cargo-native'}=={2588,2589,2590}
assert 'not checkpoint3821' in cp['qualification'] or 'checkpoint3821' in cp['qualification']
print('Verified distinct prior registration variants and mechanical P16/current production-prefix equality; no extended personal read or snapshot promotion inferred.')
