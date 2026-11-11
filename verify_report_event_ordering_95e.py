#!/usr/bin/env python3
"""Verify event/report source and recovery evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-REPORT-EVENT-ORDERING-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1437-1440.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed' and a['conditional_findings']==[]
assert [x['original_index'] for x in h['historical_source_roles']]==[1437,1438,1439,1440]
assert [x['lines'] for x in h['historical_source_roles']]==[234,352,48,792]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1437-1440.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
v=current[1439];assert b'previous.scenario != current.scenario' in v and b'!previous.payload.same_stream(current.payload)' in v and b'current.advances_from(previous)' in v
assert b'previous.owner == event.owner' in v and b'previous.payload == event.payload' in v
adj=v[v.index(b'.all(|(previous, current)|'):v.index(b'&& presentation_events')];assert b'owner' not in adj
model=current[1438];assert b'pub(in crate::animation_viewer) owner: String' in model and b'.sample_tick\n                .checked_delta(previous.sample_tick)' in model
cap=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'owner: capture_entity_id(event.owner)' in cap and b'*skeleton = SkeletonState::default()' in cap
producer=s['crates/adventuresim-tactical-client/src/animation/presentation/events.rs'];assert b'pub owner: Entity' in producer and b'for CoalescedContact { sequence, foot } in contacts' in producer and b'latest_coalesced_landing' in producer
assert b'checked_sub(previous.0)' in s['crates/adventuresim-tactical-core/src/animation/contact.rs']
val=current[1440];assert b'presentation_events::presentation_events_valid(&frames, &presentation_events)' in val and b'&& validation.event_stream_valid' in val
assert b'quality_percent = if capture_complete' in val and b'acceptance_passed: validation_passed(validation)' in val
assert b'Quality score: {:.2}%' in s['crates/adventuresim-tactical-client/src/animation_viewer/report/html.rs']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {416:[1437,1438],417:[1439],418:[1440]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
ledger=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'];owner=ledger['SEGOVB-L12-PRESENTATION-EVENT-OWNER-PRIVATE-TEXT']
for x in owner['qualified_same_lens_reuse']:
 assert x['receipt'] in records and x['registration'] in records and x['receipt_sha256']==m[x['receipt']]['sha256'] and x['registration_sha256']==m[x['registration']]['sha256']
assert 'adler/READ-L11-203.json' in records and 'adler/STATE-L11-203.json' in records and 'adler/VERIFY-L11-203.json' not in m
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1235,1236,1237,1238]
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1437,1438,1439,1440]
assert q[47]['contract_id']=='ADLER-L11-PRESENTATION-EVENT-STREAM-ORDERING' and q[255]['contract_id']=='SEGOVB-L12-PRESENTATION-EVENT-OWNER-PRIVATE-TEXT'
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [47,255,*indices])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical report bodies, attributed event records and bounded current producer connections; no product execution.')
