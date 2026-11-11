#!/usr/bin/env python3
"""Verify bounded attribution and ledger links; never run product code."""
import hashlib,io,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def full(ranges,n):
 cursor=1
 for lo,hi in sorted(ranges):
  assert lo==cursor and hi>=lo
  cursor=hi+1
 assert cursor==n+1
name='ASSESSMENT-DELEGATED-RECONCILIATION-7-95E.json';a=load(name);h=load(a['source_roles']);roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==[2374]+list(range(2376,2399))
w=load('COVERAGE-L01-ROLES.json')['roles'];cp=load(a['evidence_checkpoint']);by={};bundles={}
for bundle in cp['bundles']:
 bundles[bundle['bundle_id']]=bundle
 with tarfile.open(root/bundle['archive'],'r:gz') as t:by[bundle['bundle_id']]={x.name:t.extractfile(x).read() for x in t.getmembers()}
for x in roles:
 i=x['original_index'];r=w[i];cur=x['current_comparison'];assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']==a['source_roles'] and r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256'];assert sha((root/x['local_registration']['path']).read_bytes())==x['local_registration']['sha256'];assert x['new_personal_read_ranges']==[[1,x['lines']]] and cur['source_revision']==a['source_revision']
 bundle=next(b for b in cp['bundles'] if i in b['scope_original_roles']);data=by[bundle['bundle_id']];assert x['new_personal_read_attribution']==bundle['agent']
 report=json.loads(data[bundle['report_archive_member']])
 if bundle['agent']=='/root/hair_ops':
  item=next(z for z in report['role_records'] if z['original_index']==i);full(item['fresh_personally_displayed_body_ranges'],x['lines']);assert item['unread_assigned_ranges']==[] and x['new_personal_read_revision']=='not-recorded-byte-identical-6533-and-95e' and cur['current_personal_ranges']==[] and cur['byte_equal_personal_read_ranges']==[[1,cur['lines']]]
 elif bundle['agent']=='/root/rig_identity':
  item=next(z for z in report['original_roles'] if z['original_role']==i);old=item.get('actual_personally_displayed_ranges_historical',item.get('actual_personally_displayed_historical_ranges'));now=item.get('actual_personally_displayed_ranges_current',item.get('actual_personally_displayed_current_ranges'));full(old,x['lines']);full(now,cur['lines']);assert x['new_personal_read_revision']==x['snapshot_head'] and cur['current_personal_ranges']==[[1,cur['lines']]]
 else:
  receipt=json.loads(data['read-receipt.json']);reads=[z for z in receipt['reads'] if z['original_role']==i];assert len(reads)==2 and x['new_personal_read_revision']==x['snapshot_head']
  for read in reads:full(read['actual_display_ranges'],x['lines']);assert read['full_body_displayed'] and read['revision'] in (x['snapshot_head'],a['source_revision'])
 if i in (2381,2389,2395):assert x['prior_role_status']=='bounded-current-trace-only' and x['prior_current_observations_preserved']==r['current_observations']
assert '396–580' in next(x for x in roles if x['original_index']==2380)['historical_attribution_qualification']
assert 'Adler separately extended' in next(x for x in roles if x['original_index']==2397)['historical_attribution_qualification']
assert len(a['delegated_observations'])==51 and {x['agent'] for x in a['delegated_observations']}=={'/root/hair_ops','/root/mhr_character','/root/rig_identity'}
# Independently preserved current root supports and exact displayed ranges.
support=load(a['central_support']);raw=(root/support['archive']).read_bytes();assert len(raw)==support['bytes'] and sha(raw)==support['sha256']
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as tar:
 files=tar.getmembers();assert all(x.isfile() and not x.name.startswith('/') and '..' not in Path(x.name).parts for x in files);data={x.name:tar.extractfile(x).read() for x in files}
assert len(files)==len(data)==len(support['members']) and set(data)=={x['path'] for x in support['members']}
for x in support['members']:assert sha(data[x['path']])==x['sha256'] and len(data[x['path']])==x['bytes']
reg=json.loads(data['registration.json']);receipt=json.loads(data['read-receipt.json']);assert receipt['registration_sha256']==sha(data['registration.json']) and len(reg['sources'])==len(receipt['actual_personal_displays'])==11
inventory={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);inventory[path]=head.split()[2]
for i,(x,read) in enumerate(zip(reg['sources'],receipt['actual_personal_displays'])):
 body=data['source/'+str(i)];assert x['revision']==read['revision']==a['source_revision'] and read['path']==x['path'] and read['ranges']==x['prospectively_registered_ranges'] and read['sha256']==x['sha256'];assert sha(body)==x['sha256'] and len(body)==x['bytes'] and oid(body)==x['git_blob']==inventory[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in read['ranges']) and len(body.splitlines())==x['lines']
old=load('baseline-059/queue.json')['entries'];q=load('RECONCILIATION-059.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger)
indices=(179,187,188,191,192,193,194,195,196,197,201,202,510,517,518,519,520,521,522,523,524,525,526,527,529,530,531,532,533,534,535,536,537,538,539,546,629,1006,1008,1872,1876,1878,1881,1883,1885,2254,2454,3299)
for i in indices:
 x=q[i];assert x['contract_id']==old[i]['contract_id'] and x['current_reconciliation'].startswith('partial ') and (name in x['current_reconciliation'] or name in x.get('further_current_evidence',[]))
for i in (192,194,195,196,197,202,517,518,519,520,521,522,523,524,525,526,527,529,530,531,532,533,534,535,536,537,538,539,1881,1883,3299):
 x=ids[old[i]['contract_id']];assert x['record']==name and x['repair']=='not started' and x['validation']=='not performed'
for i in (179,187,191,193,510):assert name in ids[old[i]['contract_id']]['further_current_evidence']
for ident in ('REVIEW-MEDICAL-OBSERVATION-GAP-EVENT-95E','REVIEW-ITINERARY-CLOCK-NONPROGRESS-001','REVIEW-SAME-SNAPPED-NODE-ZERO-ROUTE-001'):assert name in ids[ident]['further_current_evidence']
candidates=[x for x in ledger if x.get('record')==name and x.get('historical_related_records')==['DELEGATED-EVIDENCE-CHECKPOINT-7.json',name]]
assert len(candidates)==26 and all(x['repair']=='not started' and x['validation']=='not performed' and x['additional_current_lenses']==[] for x in candidates)
assert 'Bandage separately carries' in ids['DELEGATE-WEB-MEDICAL-CONTEXT-CLAIM-95E']['relationship_qualification']
assert 'Adler READ-L11-391' in ids['REVIEW-CHARACTER-NONE-PARTY-ACTION-95E']['relationship_qualification']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['delegated_batch7_central_reconciliation']==name and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified 24 attributed delegated roles, bounded root support, 48 queue links and connected ledger; no product execution.')
