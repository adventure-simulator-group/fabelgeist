#!/usr/bin/env python3
"""Verify source-read attribution, bounded support and ledger links only."""
import hashlib,io,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent;load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def full(ranges,n):
 cursor=1
 for lo,hi in sorted(ranges):assert lo==cursor and hi>=lo;cursor=hi+1
 assert cursor==n+1
name='ASSESSMENT-DELEGATED-RECONCILIATION-8-95E.json';a=load(name);roles=load(a['source_roles'])['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2399,2415))+list(range(2417,2431))
w=load('COVERAGE-L01-ROLES.json')['roles'];cp=load(a['evidence_checkpoint']);by={}
for b in cp['bundles']:
 with tarfile.open(root/b['archive'],'r:gz') as t:by[b['bundle_id']]={x.name:t.extractfile(x).read() for x in t.getmembers()}
for x in roles:
 i=x['original_index'];r=w[i];cur=x['current_comparison'];assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'];assert sha((root/x['local_registration']['path']).read_bytes())==x['local_registration']['sha256'] and x['new_personal_read_ranges']==[[1,x['lines']]] and cur['source_revision']==a['source_revision'] and cur['current_personal_ranges']==[[1,cur['lines']]]
 b=next(b for b in cp['bundles'] if i in b['scope_original_roles']);data=by[b['bundle_id']];assert x['new_personal_read_attribution']==b['agent'];report=json.loads(data[b['report_archive_member']])
 if b['agent']=='/root/hair_ops':
  item=next(z for z in report['role_records'] if z['original_index']==i);full(item['fresh_personally_displayed_body_ranges'],cur['lines']);assert item['unread_assigned_ranges']==[] and cur['byte_identical'] and x['new_personal_read_revision']==a['source_revision'];assert any(str(i) in z['scope'] or b['bundle_id'] in z['scope'] for z in cp['current_session_clarifications'])
 elif b['agent']=='/root/rig_identity':
  item=next(z for z in report['original_roles'] if z['original_role']==i);full(item['actual_personally_displayed_historical_ranges'],x['lines']);full(item['actual_personally_displayed_current_ranges'],cur['lines']);assert x['new_personal_read_revision']==x['snapshot_head']
 else:
  receipt=json.loads(data['read-receipt.json']);reads=[z for z in receipt['reads'] if z['original_role']==i];assert len(reads)==2 and x['new_personal_read_revision']==x['snapshot_head']
  for read in reads:
   expected=x['lines'] if read['revision']==x['snapshot_head'] else cur['lines'];full(read['actual_display_ranges'],expected);assert read['full_body_displayed'] and read['revision'] in (x['snapshot_head'],a['source_revision'])
 if i==2413:assert x['prior_role_status']=='bounded-current-trace-only' and x['prior_current_observations_preserved']==r['current_observations']
 if i==2428:assert not cur['byte_identical'] and 'RIG-SUPP-PP-006' in x['historical_attribution_qualification']
assert 'fresh 391–2165' in next(x for x in roles if x['original_index']==2399)['historical_attribution_qualification'];assert len(a['delegated_observations'])==48 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
support=load(a['central_support']);raw=(root/support['archive']).read_bytes();assert len(raw)==support['bytes'] and sha(raw)==support['sha256']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 files=t.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:t.extractfile(x).read() for x in files}
assert len(files)==len(data)==len(support['members']) and set(data)=={x['path'] for x in support['members']}
for x in support['members']:assert sha(data[x['path']])==x['sha256'] and len(data[x['path']])==x['bytes']
reg=json.loads(data['registration.json']);receipt=json.loads(data['read-receipt.json']);assert receipt['registration_sha256']==sha(data['registration.json']) and len(reg['sources'])==len(receipt['actual_personal_displays'])==11
inventory={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);inventory[path]=head.split()[2]
for i,(x,read) in enumerate(zip(reg['sources'],receipt['actual_personal_displays'])):
 body=data['source/'+str(i)];assert x['revision']==read['revision']==a['source_revision'] and read['path']==x['path'] and read['ranges']==x['prospectively_registered_ranges'] and read['sha256']==x['sha256'];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(body)==x['git_blob']==inventory[x['path']] and len(body.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in read['ranges'])
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
indices=(193,197,198,199,200,201,202,203,510,513,514,515,516,517,540,541,542,543,546,547,548,549,552,556,1385,1749,1899,2454,2456,2457,2458,2459)
for i in indices:
 x=q[i];assert x['contract_id']==old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ') and (name in x['current_reconciliation'] or name in x.get('further_current_evidence',[]))
for i in (198,200,201,203,513,514,515,516,540,541,542,543,546,547,548,549,552,556):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (193,197,202,510,517):assert name in ids[old[i]['contract_id']]['further_current_evidence']
for ident in ('REVIEW-MEDICAL-OBSERVATION-GAP-EVENT-95E','REVIEW-TERRAIN-FAMILY-HOURS-CARDINALITY-001','DELEGATE-QUEST-LOOT-TARGET-PARTY-ROW-AGGREGATION-95E'):assert name in ids[ident]['further_current_evidence']
x=ids['NEW-TACTICAL-ENTRY-CHARACTER-ID-001'];assert 'RIG-SUPP-PP-006' in x['relationship_qualification'] and 'not a new unique issue' in x['relationship_qualification'] and any(z['id']==x['id'] for z in load('BASELINE-20261110.json')['repair_ledger']['entries'])
candidates=[x for x in ledger if x.get('record')==name and x.get('historical_related_records')==['DELEGATED-EVIDENCE-CHECKPOINT-8.json',name]];assert len(candidates)==16 and all(x['repair']=='not started' and x['validation']=='not performed' and x['additional_current_lenses']==[] for x in candidates)
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch8_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified 30 attributed delegated roles, bounded root support, 32 queue links and connected ledger; no product execution.')
