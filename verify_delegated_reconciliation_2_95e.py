#!/usr/bin/env python3
"""Verify second central delegated reconciliation without product execution."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-2-95E.json';a=load(name);h=load(a['source_roles']);work=load('COVERAGE-L01-ROLES.json')['roles'];roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==[2306,2307,2308,2319,2320,2321,2322,2336,2337,2338]
for x in roles:
 r=work[x['original_index']];reg=(root/x['local_registration']['path']).read_bytes();assert sha(reg)==x['local_registration']['sha256'] and r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'] and x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==[[1,x['lines']]] and x['new_personal_read_attribution'] in ('/root/hair_ops','/root/mhr_character','/root/rig_identity')
 c=x['current_comparison'];assert c['source_revision']==a['source_revision'] and c['current_personal_ranges']==[[1,c['lines']]]
assert 'fresh_reads=[]' in next(x for x in roles if x['original_index']==2306)['historical_attribution_qualification'] and 'fresh_reads=[]' in next(x for x in roles if x['original_index']==2308)['historical_attribution_qualification']
assert not next(x for x in roles if x['original_index']==2336)['current_comparison']['byte_identical'] and not next(x for x in roles if x['original_index']==2337)['current_comparison']['byte_identical'] and next(x for x in roles if x['original_index']==2338)['current_comparison']['byte_identical']
assert len(a['delegated_observations'])==23 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for i in (161,168,169,170,171,172,173,175,377,378,379,380,381,401,402,421,422,429,430,431,432,436,572,1855,2411):
 assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and (name in q[i]['current_reconciliation'] or name in q[i].get('further_current_evidence',[]))
for i in (168,169,173,378,380,421,422,429,430):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (161,170,171,175,377,379,381,436):assert name in ids[old[i]['contract_id']]['further_current_evidence']
for ident in ('REVIEW-RIGID-SKIN-ROW-CORRESPONDENCE-001','REVIEW-MHR-SKELETON-COLUMN-ADMISSION-95E','REVIEW-RIG-EQUAL-BOUND-INSTANCE-QUESTION-95E','REVIEW-HAIR-RNG-EMPTY-INDEX-95E','REVIEW-HAIR-RNG-RANGE-HALF-OPEN-95E'):assert name in ids[ident]['further_current_evidence']
for ident in ('REVIEW-MHR-PARAMETER-NONFINITE-COEFFICIENT-95E','REVIEW-MHR-CORRECTIVE-INDEX-NARROWING-32BIT-95E','REVIEW-MHR-TINY-TRIANGLE-NORMAL-95E','REVIEW-HAIR-SMALL-SEGMENT-RESAMPLE-95E','REVIEW-HAIR-SMALL-SEGMENT-TANGENT-95E','REVIEW-HAIR-RNG-LARGE-INDEX-REACHABILITY-95E'):
 x=ids[ident];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
assert any('q172/q402' in x and 'not asserted for current source' in x for x in a['central_decisions'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch2_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified second delegated source attribution, guarded prior cases and connected ledger; no product execution.')
