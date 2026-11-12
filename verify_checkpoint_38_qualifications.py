#!/usr/bin/env python3
"""Preserve selected redacted variants, working snapshot and prior display caveat."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads((R/p).read_text());cp=load('DELEGATED-EVIDENCE-CHECKPOINT-38.json');bs={x['bundle_id']:x for x in cp['bundles']};prior=next(x for x in load('DELEGATED-EVIDENCE-CHECKPOINT-17.json')['bundles'] if x['bundle_id']=='armor-extremities-joint-hair-ops');old={x['path']:x for x in prior['members']};b=bs['extremity-joint-2617-2618'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('history/prior-role/published-comparison.json'));assert d['archive_sha256']==prior['archive_sha256']==sha((R/prior['archive']).read_bytes());different=[]
 with tarfile.open(R/prior['archive']) as pub:
  for x in d['comparisons']:
   m=old[x['member']];assert (m['original_sha256'],m['published_sha256'])==(x['original_sha256'],x['published_sha256']);assert sha(pub.extractfile(x['member']).read())==x['published_sha256'];assert x['byte_identical']==(x['original_sha256']==x['published_sha256'])
   if not x['byte_identical']:different.append(x['member'])
 assert different==['historical-manifest.json']
 a=t.extractfile('history/published/BASELINE-95E-20261110.json').read();w=t.extractfile('history/published/working-baseline-observed.json').read();assert sha(a)=='28138b4548101716246fee073984868472996f0e350d3f3526d237d61c6a67f9' and sha(w)=='fc828ecf7e09c2b563c46ec6b8ef5c67267baeb5455f1ea62d3739b2e2ded6ad' and a!=w
 ar=json.loads(a)['newly_assessed_ledger']['entries'];wr=json.loads(w)['newly_assessed_ledger']['entries'];key='REVIEW-METAL-SEED-DRAW-OVERLAP-95E';assert next(x for x in ar if x['id']==key)==next(x for x in wr if x['id']==key)
 p=json.load(t.extractfile('git-proofs.json'));assert any(z['revision']=='56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e' and z['path']=='BASELINE-95E-20261110.json' and z['sha256']==sha(a) for z in p.get('paths',p.get('proofs',[])))
b=bs['joint-pauldron-2621-2622']
with tarfile.open(R/b['archive']) as t:
 report=json.load(t.extractfile('report.json'));priorread=next(x for x in report['historical_attribution'] if x['reviewer']=='/root/mhr_character earlier checkpoint17 packet');assert 'pre-registration locator display of joint_cup.rs line142 with no credit' in priorread['scope'] and 'later reread' in priorread['scope']
print('Verified prior redaction variant, separate committed/working snapshots and earlier unregistered locator qualification; no extra reader credit inferred.')
