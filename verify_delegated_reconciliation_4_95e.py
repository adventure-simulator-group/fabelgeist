#!/usr/bin/env python3
"""Verify fourth central delegated reconciliation, with no product execution."""
import hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-4-95E.json';a=load(name);h=load(a['source_roles']);roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==[2343,2344,2345,2346,2349,2350,2351,2352,2353,2355,2356,2357]
w=load('COVERAGE-L01-ROLES.json')['roles'];cp=load(a['evidence_checkpoint']);by={}
for b in cp['bundles']:
 with tarfile.open(root/b['archive'],'r:gz') as t:by[b['bundle_id']]={x.name:t.extractfile(x).read() for x in t.getmembers()}
for x in roles:
 i=x['original_index'];r=w[i];cur=x['current_comparison'];assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'];assert sha((root/x['local_registration']['path']).read_bytes())==x['local_registration']['sha256'] and x['new_personal_read_ranges']==[[1,x['lines']]] and cur['source_revision']==a['source_revision'] and cur['current_personal_ranges']==[[1,cur['lines']]]
 if i in (2355,2356,2357):
  report=json.loads(by['strategic-web']['report.json']);item=next(z for z in report['role_records'] if z['index']==i);assert x['new_personal_read_revision']==a['source_revision'] and cur['byte_identical'] and item['actual_current_personal_reads']==x['new_personal_read_ranges'] and 'not a separately claimed historical reviewer read' in item['historical_6533_display']
 elif i in (2349,2350,2351):
  report=json.loads(by['xpbd-dynamics']['REPORT.json']);item=next(z for z in report['original_roles'] if z['original_index']==i);assert x['new_personal_read_revision']==x['snapshot_head'] and item['historical_displayed_ranges']==x['new_personal_read_ranges'] and item['current_displayed_ranges']==cur['current_personal_ranges']
 else:
  d=by['shell-solver' if i in (2343,2344,2345,2346) else 'xpbd-solver'];receipt=json.loads(d['read-receipt.json']);reads=receipt.get('personal_reads',receipt.get('actual_source_displays'));old=next(z for z in reads if z['path']==cur['path'] and z['revision']==x['snapshot_head']);now=next(z for z in reads if z['path']==cur['path'] and z['revision']==a['source_revision']);assert x['new_personal_read_revision']==x['snapshot_head'] and old['ranges']==x['new_personal_read_ranges'] and old['sha256']==x['sha256'] and now['ranges']==cur['current_personal_ranges'] and now['sha256']==cur['sha256']
assert '1–160 of 162' in next(x for x in roles if x['original_index']==2350)['historical_attribution_qualification']
assert 'null read_status' in next(x for x in roles if x['original_index']==2357)['historical_attribution_qualification']
assert len(a['delegated_observations'])==21 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for i in (176,177,178,193,444,445,447,448,449,450,451,452,453,454,456,459,622,625,1856,1858,1859,1860,2449,3296):
 x=q[i];assert x['contract_id']==old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ') and (name in x['current_reconciliation'] or name in x.get('further_current_evidence',[]))
assert 'no ordinary storefront panic' in q[454]['bounded_current_disposition'] and 'no ordinary-path panic' in q[459]['bounded_current_disposition'] and 'POST rerender' in q[456]['bounded_current_disposition']
for i in (176,177,178,193,447,448,449,450,451,452,453,454,456,459):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (444,445):assert name in ids[old[i]['contract_id']]['further_current_evidence']
for ident in ('REVIEW-SHELL-GRID-RADIUS-SPACING-95E','REVIEW-SHELL-NONDEGENERATE-BEND-CUTOFF-95E','REVIEW-SHELL-COLLISION-QUANTIZATION-95E','REVIEW-XPBD-POSITIVE-TINY-MASS-PIN-95E','REVIEW-XPBD-INVERSE-MASS-SUM-OVERFLOW-95E','REVIEW-XPBD-ATTACHMENT-RESERVED-NAMES-95E','REVIEW-XPBD-RAW-ATTACHMENT-STRIDE-95E','REVIEW-XPBD-CONSTRAINT-PARTICLE-MEMBERSHIP-95E','REVIEW-XPBD-REORDER-VALUE-EXTENT-95E','REVIEW-XPBD-NEGATIVE-SPEED-CAP-95E','REVIEW-XPBD-SPEED-CAP-PHASE-95E'):
 x=ids[ident];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
assert any('actual original 059 queue entry193' in x for x in a['central_decisions']) and any('not proof of personal reads' in x for x in a['central_decisions'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch4_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified fourth delegated source attribution, safeguards and connected ledger; no product execution.')
