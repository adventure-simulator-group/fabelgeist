#!/usr/bin/env python3
"""Verify fifth central reconciliation and partial lock evidence only."""
import hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-5-95E.json';a=load(name);h=load(a['source_roles']);roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==[2312,2314,2315,2326,2330,2331,2339,2347,2348,2358,2359,2360]
w=load('COVERAGE-L01-ROLES.json')['roles'];cp=load(a['evidence_checkpoint']);by={}
for b in cp['bundles']:
 with tarfile.open(root/b['archive'],'r:gz') as t:by[b['bundle_id']]={x.name:t.extractfile(x).read() for x in t.getmembers()}
for x in roles:
 i=x['original_index'];r=w[i];cur=x['current_comparison'];assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'];assert sha((root/x['local_registration']['path']).read_bytes())==x['local_registration']['sha256'] and x['new_personal_read_ranges']==[[1,x['lines']]] and cur['source_revision']==a['source_revision'] and cur['current_personal_ranges']==[[1,cur['lines']]]
 if i in (2358,2359,2360):
  report=json.loads(by['web-data']['report.json']);item=next(z for z in report['role_records'] if z['index']==i);assert x['new_personal_read_revision']==a['source_revision'] and cur['byte_identical'] and item['actual_personal_current_reads']==x['new_personal_read_ranges'] and 'not a newly claimed historical reviewer read' in item['historical_qualification']
 elif i in (2326,2330,2331,2339,2347,2348):
  report=json.loads(by['crate-metadata']['REPORT.json']);item=next(z for z in report['original_roles'] if z['original_index']==i);assert x['new_personal_read_revision']==x['snapshot_head'] and item['historical_displayed_ranges']==x['new_personal_read_ranges'] and item['current_displayed_ranges']==cur['current_personal_ranges']
 else:
  receipt=json.loads(by['mhr-metadata']['read-receipt.json']);reads=receipt['actual_source_displays'];old=next(z for z in reads if z['path']==cur['path'] and z['revision']==x['snapshot_head']);now=next(z for z in reads if z['path']==cur['path'] and z['revision']==a['source_revision']);assert x['new_personal_read_revision']==x['snapshot_head'] and old['ranges']==x['new_personal_read_ranges'] and old['sha256']==x['sha256'] and now['ranges']==cur['current_personal_ranges'] and now['sha256']==cur['sha256']
lock=load(a['partial_lock']);assert lock['original_index']==2313 and w[2313]['status']=='lock-metadata-partial' and w[2313]['assessment']==a['partial_lock'];assert lock['historical_unread_lines']==6898 and lock['current_unread_lines']==6904 and 'personally displayed only listed ranges' in lock['qualification'] and 'none grants the current reader full' in lock['qualification']
assert sum(hi-lo+1 for lo,hi in lock['historical_personal_ranges'])==195 and sum(hi-lo+1 for lo,hi in lock['current_personal_ranges'])==198
assert len(a['delegated_observations'])==15 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for i in (170,173,175,179,181,193,311,312,421,422,436,457,458,459,1284,1848,1851,2309,2410,2414):
 x=q[i];assert x['contract_id']==old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ') and (name in x['current_reconciliation'] or name in x.get('further_current_evidence',[]))
assert 'no new native body-read' in q[2309]['bounded_current_disposition'] and 'duplicate' in q[1284]['bounded_current_disposition']
for i in (179,457,458,1284):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (170,173,175,193,421,422,436,459):assert name in ids[old[i]['contract_id']]['further_current_evidence']
assert name in ids['REVIEW-MHR-TINY-TRIANGLE-NORMAL-95E']['further_current_evidence']
for ident in ('REVIEW-MEDICAL-OBSERVATION-GAP-EVENT-95E','REVIEW-LIFE-ONLY-CASE-SITE-DEPENDENCY-95E','REVIEW-MHR-FORWARD-WIDTH-DOCUMENTATION-95E','REVIEW-MHR-URI-ASSET-COHORT-95E'):
 x=ids[ident];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
assert any('not source role2309' in x for x in a['central_decisions']) and any('195/7093' in x and '198/7102' in x for x in a['central_decisions'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch5_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified fifth delegated source attribution, partial lock and connected ledger; no product execution.')
