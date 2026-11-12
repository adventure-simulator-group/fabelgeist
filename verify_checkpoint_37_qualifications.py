#!/usr/bin/env python3
"""Keep committed and observed working baselines distinct and bound selected read scopes."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-37.json').read_text());bs={x['bundle_id']:x for x in cp['bundles']};b=bs['helmet-crown-buffe-2611-2612']
with tarfile.open(R/b['archive']) as t:
 a=t.extractfile('history/published/BASELINE-95E-20261110.json').read();w=t.extractfile('history/published/working-baseline-observed.json').read();assert sha(a)=='decd0b95699edfc1a59039358bc3a171511c2c0199afab8967cfd7169aca3157' and sha(w)=='28138b4548101716246fee073984868472996f0e350d3f3526d237d61c6a67f9' and a!=w
 x=json.loads(a)['newly_assessed_ledger']['entries'];y=json.loads(w)['newly_assessed_ledger']['entries'];assert len(x)==1021 and len(y)==1022 and x[471:473]==y[471:473] and [z['id'] for z in x[471:473]]==['REVIEW-L3-HELMET-CARRIER-TOPOLOGY-ADMISSION','REVIEW-BUFFE-BREATH-COURSE-CONTEXT-95E']
 p=json.load(t.extractfile('git-proofs.json'));assert any(z['revision']=='c15289d7b23f6253e01fb02de79551a89ff10c47' and z['path']=='BASELINE-95E-20261110.json' and z['sha256']==sha(a) for z in p.get('paths',p.get('proofs',[])))
b=bs['pierced-sampling-2615-2616']
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('registration.json'));rows={x['tag']:x for x in d['entries'] if x.get('role')==2616};assert rows['pr-working-2616']['registered_ranges']==[[1,45]] and rows['current-2616']['registered_ranges']==[[1,49]] and rows['pr-working-2616']['sha256']!=rows['current-2616']['sha256']
 report=json.load(t.extractfile('report.json'));assert 'full saved bytes are not full personal reads' in report['scope']['personal_scope']
print('Verified distinct committed/working baseline copies, matching selected entries and limited sampling displays; no later full-body or checkpoint3821 credit inferred.')
