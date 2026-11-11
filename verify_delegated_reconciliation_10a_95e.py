#!/usr/bin/env python3
"""Verify eight delegated P17 reads and their bounded ledger connections."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda n:json.loads((R/n).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def normalize(ranges):
 out=[]
 for a,b in sorted(ranges):
  assert 1<=a<=b
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bundles={x['bundle_id']:x for x in cp['bundles']};selected=['p17-supported-slip','p17-report-model','p17-observation-validation-hair','p17-core-measurement'];rec=load('ASSESSMENT-DELEGATED-P17-2460-2467.json');rows=rec['historical_source_roles'];work=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in rows]==list(range(2460,2468))
for name in selected:
 b=bundles[name];raw=(R/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes']
 receipt_name='personal-read-receipt.json' if name in ('p17-supported-slip','p17-core-measurement') else 'read-receipt.json'
 with tarfile.open(R/b['archive'],'r:gz') as t:receipt=json.load(t.extractfile(receipt_name));report=json.load(t.extractfile(b['report_archive_member']))
 assert report==load(b['report_path'])
 for x in rows:
  i=x['original_index']
  if i not in b['scope_original_roles']:continue
  cc=x['current_comparison'];assert x['receipt_member']==receipt_name and x['new_personal_read_attribution']==b['agent'] and x['evidence_archive']==b['archive'] and x['delegate_report']==b['report_path']
  assert work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']=='ASSESSMENT-DELEGATED-P17-2460-2467.json' and work[i]['historical_sha256']==x['sha256'] and x['snapshot_head']=='90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696'
  if name in ('p17-supported-slip','p17-core-measurement'):
   old=[r['ranges'] for r in receipt['assigned_personal_reads'] if r.get('role')==i and r['revision']==x['snapshot_head']]
   now=[r['ranges'] for r in receipt['assigned_personal_reads'] if r.get('role')==i and r['revision']==cc['source_revision']]
  elif name=='p17-report-model':
   old=[r['actual_display_ranges'] for r in receipt['reads'] if r.get('role')==i and r['revision']==x['snapshot_head']]
   now=[r['actual_display_ranges'] for r in receipt['reads'] if r.get('role')==i and r['revision']==cc['source_revision']]
  else:
   a=next(r for r in receipt['roles'] if r['original_index']==i);old=[a['p17_actual_display_ranges']];now=[a['current_actual_display_ranges']]
  assert len(old)==1 and old[0]==x['actual_personal_display_chunks'] and normalize(old[0])==x['new_personal_read_ranges']==[[1,x['lines']]]
  assert (now[0] if now else [])==cc['actual_personal_display_chunks'] and normalize(cc['actual_personal_display_chunks'])==cc['personal_ranges']
  if cc['status']=='present':assert cc['personal_ranges']==[[1,cc['lines']]] and cc['sha256'] and cc['git_blob']
  else:assert i==2463 and cc['status']=='absent' and not cc['personal_ranges'] and cc['sha256'] is None
report=load('ASSESSMENT-DELEGATED-RECONCILIATION-10A-95E.json');assert len(report['delegated_observations'])==15 and len(report['new_qualified_ledger_rows'])==5 and report['repair']=='not started' and report['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
for ident in report['new_qualified_ledger_rows']:
 x=ids[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10A-95E.json' and x['repair']=='not started' and x['validation']=='not performed'
assert 'no new unique issue' in ids['SEGOVB-L11-CAPTURE-RUN-ARTIFACT-CERTIFICATION']['relationship_qualification']
for ident in ('P17-SLIP-INVALID-BODY-TURN','REVIEW-LANDING-RECOVERY-RATE-OVERFLOW-001','ADLER-L11-LOCOMOTION-PROFILE-ADMISSION','CAPTURE-LANDING-MISSING-HEIGHT-AS-ZERO-95E','REVIEW-REPORT-KNEE-AXIS-OBSERVATION-95E','P17-HEIGHT-DOUBLE-WARMUP'):
 assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10A-95E.json' in ids[ident]['further_current_evidence']
progress=load('CONTINUATION-PROGRESS-95E.json');assert progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0 and progress['new_current_ledger_candidates']==len(ledger)
assert progress['delegated_batch10_pending_central_reconciliation']=='DELEGATED-EVIDENCE-CHECKPOINT-10.json' and progress['delegated_batch10_partial_reconciliation']==['ASSESSMENT-DELEGATED-RECONCILIATION-10A-95E.json']
print('Verified eight P17 historical roles, exact receipts, 15 observations and five qualified rows; the remaining 23 packets are pending.')
