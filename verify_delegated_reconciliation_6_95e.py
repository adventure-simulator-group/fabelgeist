#!/usr/bin/env python3
"""Verify sixth delegated reconciliation and actual read attribution only."""
import hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def full(ranges,n):
 cursor=1
 for lo,hi in sorted(ranges):assert lo==cursor and hi>=lo;cursor=hi+1
 assert cursor==n+1
name='ASSESSMENT-DELEGATED-RECONCILIATION-6-95E.json';a=load(name);h=load(a['source_roles']);roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==[2361,2362,2363,2364,2365,2366,2367,2368,2369,2372,2373,2375]
w=load('COVERAGE-L01-ROLES.json')['roles'];cp=load(a['evidence_checkpoint']);by={}
for b in cp['bundles']:
 with tarfile.open(root/b['archive'],'r:gz') as t:by[b['bundle_id']]={x.name:t.extractfile(x).read() for x in t.getmembers()}
for x in roles:
 i=x['original_index'];r=w[i];cur=x['current_comparison'];assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'];assert sha((root/x['local_registration']['path']).read_bytes())==x['local_registration']['sha256'] and x['new_personal_read_ranges']==[[1,x['lines']]] and cur['source_revision']==a['source_revision']
 if i in (2361,2362):
  report=json.loads(by['web-dialogue']['report.json']);item=next(z for z in report['role_records'] if z['original_index']==i);full(item['actual_personally_displayed_current_body_ranges'],x['lines']);assert x['new_personal_read_revision']==a['source_revision'] and cur['byte_identical'] and cur['current_personal_ranges']==[[1,cur['lines']]]
 elif i in (2372,2373,2375):
  report=json.loads(by['web-equipment']['report.json']);item=next(z for z in report['role_records'] if z['original_index']==i);full(item['fresh_personally_displayed_body_ranges'],x['lines']);assert x['new_personal_read_revision']=='not-recorded-byte-identical-6533-and-95e' and cur['byte_identical'] and cur['current_personal_ranges']==[] and cur['byte_equal_personal_read_ranges']==[[1,cur['lines']]]
 elif i in (2364,2365,2366,2367,2368):
  report=json.loads(by['web-map']['REPORT.json']);item=next(z for z in report['original_roles'] if z['original_role']==i);assert x['new_personal_read_revision']==x['snapshot_head'] and item['actual_personally_displayed_ranges_historical']==x['new_personal_read_ranges'] and item['actual_personally_displayed_ranges_current']==cur['current_personal_ranges']
 else:
  receipt=json.loads(by['web-routes']['read-receipt.json']);reads=[z for z in receipt['reads'] if z['original_role']==i];assert len(reads)==2 and x['new_personal_read_revision']==x['snapshot_head']
  for z in reads:full(z['actual_display_ranges'],x['lines']);assert z['full_body_displayed'] and z['revision'] in (x['snapshot_head'],a['source_revision'])
assert next(x for x in roles if x['original_index']==2369)['prior_role_status']=='bounded-current-trace-only' and w[2369]['current_observations']
assert '296–484' in next(x for x in roles if x['original_index']==2375)['historical_attribution_qualification']
assert len(a['delegated_observations'])==21 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for i in (179,180,181,182,183,184,187,191,201,210,279,461,462,463,464,465,466,467,468,469,470,471,472,473,474,475,510,552,913,914,1644,1866):
 x=q[i];assert x['contract_id']==old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ') and (name in x['current_reconciliation'] or name in x.get('further_current_evidence',[]))
assert 'separately stored observer receipt' in q[462]['bounded_current_disposition'] and 'old successful read' in q[191]['bounded_current_disposition']
for i in (180,181,182,183,184,187,191,210,461,462,463,464,465,466,467,468,469,470,471,472,473,474,475,510):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (179,279):assert name in ids[old[i]['contract_id']]['further_current_evidence']
for ident in ('REVIEW-LIFE-ONLY-CASE-SITE-DEPENDENCY-95E','SEGOVB-L12-COND-SESSION-NPC-REUSE-BINDING','REVIEW-STRATEGIC-EQUIPMENT-OMISSION-READINESS-001'):assert name in ids[ident]['further_current_evidence']
for ident in ('REVIEW-DIALOGUE-REFERRAL-SOURCE-SHAPE-95E','REVIEW-STRATEGIC-ENCUMBRANCE-INPUT-AVAILABILITY-95E','REVIEW-WEB-MAP-BOUNDARY-SIMPLIFICATION-95E','REVIEW-WEB-MAP-ROAD-KIND-AUTHORITY-95E','REVIEW-WEB-MAP-SIMPLIFICATION-WORK-95E','REVIEW-FORAGE-LICENSE-UNKNOWN-95E','REVIEW-FORAGE-NOTICED-INFAMY-CUE-95E','REVIEW-FORAGE-PRIVILEGE-MAPPING-AUTHORITY-95E'):
 x=ids[ident];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
assert any('without recording which' in x for x in a['central_decisions'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch6_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified sixth delegated source attribution, prior bounded trace and connected ledger; no product execution.')
