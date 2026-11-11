#!/usr/bin/env python3
"""Verify central delegated reconciliation without product execution."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-95E.json';a=load(name);h=load(a['source_roles']);w=load('COVERAGE-L01-ROLES.json')['roles'];rows=h['historical_source_roles'];assert [x['original_index'] for x in rows]==[2299,2300,2301,2316,2317,2318,2332,2333,2334,2335]
for x in rows:
 r=w[x['original_index']];reg=(root/x['local_registration']['path']).read_bytes();assert sha(reg)==x['local_registration']['sha256'] and r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'] and x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==[[1,x['lines']]] and x['new_personal_read_attribution'] in ('/root/hair_ops','/root/mhr_character','/root/rig_identity')
 current=x['current_comparison'];assert current['current_personal_ranges']==[[1,current['lines']]] and current['source_revision']==a['source_revision']
assert not next(x for x in rows if x['original_index']==2316)['current_comparison']['byte_identical'] and next(x for x in rows if x['original_index']==2317)['current_comparison']['byte_identical'] and not next(x for x in rows if x['original_index']==2318)['current_comparison']['byte_identical']
assert len(a['delegated_observations'])==21 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for i in (162,163,164,165,166,167,170,171,175,254,372,373,375,376,379,433,434,436,572,618,642):
 assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and (name in q[i]['current_reconciliation'] or name in q[i].get('further_current_evidence',[]))
for i in (162,165,166,167,170,171,254,372,375,376,433,434,436):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (163,164,175,373,379):assert name in ids[old[i]['contract_id']]['further_current_evidence']
assert sum(x['id']=='REVIEW-MHR-SKELETON-COLUMN-ADMISSION-95E' for x in ledger)==1 and ids['SEGOVB-L11-POINT-GRID-MAX-RADIUS']['record']==name
for ident in ('REVIEW-HAIR-NEAREST-UNDERFILLED-SEARCH-95E','REVIEW-HAIR-INTERPOLATE-ANTIPARALLEL-95E','REVIEW-HAIR-OPERATION-EXISTING-OWNERS-95E','REVIEW-HAIR-NEIGHBOR-CAPACITY-OVERFLOW-95E','REVIEW-MHR-SKELETON-COLUMN-ADMISSION-95E','REVIEW-MHR-COORDINATE-NARROWING-95E','REVIEW-MHR-BLEND-SHAPE-LOAD-MODE-95E','REVIEW-RIG-EQUAL-BOUND-INSTANCE-QUESTION-95E'):
 x=ids[ident];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch1_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified delegated source attribution, 21 historical links and central ledger relationships; no product execution.')
