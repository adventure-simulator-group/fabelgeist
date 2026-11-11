#!/usr/bin/env python3
"""Verify historical/source evidence and qualifications, not product code."""
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
a=load('ASSESSMENT-QUICKSTEP-IDENTITY-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1350-1355.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1350,1356))
assert [x['lines'] for x in h['historical_source_roles']]==[867,170,90,203,174,109]
historical={};current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1350-1355.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];historical[x['original_index']]=raw;current[x['original_index']]=cur
assert [x['original_index'] for x in h['historical_source_roles'] if not x['current_comparison']['byte_identical']]==[1355]
assert h['historical_source_roles'][4]['prior_role_status']=='bounded-current-trace-only'
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
presentation=current[1350]
for x in [b'quickstep_phase: Option<UnitInterval>',b'start_tick: AnimationSampleTick',b'let continuing = previous.is_quickstep();',b'let mut visual_phase = phase.unwrap_or_else',b'let start_tick = source',b'*phase = Some(visual_phase)']:
 assert x in presentation
assert b'any_with_component::<' in current[1355] and b'any_with_component::<' not in historical[1355]
state=s['crates/adventuresim-tactical-core/src/animation/state.rs']
assert b'if self.action_kind() == SkeletonAction::Dodge' in state and b'ActionTimelineSample::Complete => action = ActionState::default()' in state
timeline=s['crates/adventuresim-tactical-core/src/animation/state/action_timeline.rs']
assert b'if current_tick > end_tick || end_tick == AnimationSampleTick::MAX' in timeline
assert b'ActionTimelineSample::Active(UnitInterval::ONE)' in timeline
server=s['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'begin_authoritative_quickstep(' in server and b'.begin_dodge(' in server
assert b'skeleton.weapon_guard() != WeaponGuardState::Raised' in server
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
for batch,ids in {375:[1350],376:[1351,1352,1353],377:[1354,1355]}.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert a['existing_finding_connections'][0]['id']=='ADLER-L11-QUICKSTEP-PHASE-ACTION-IDENTITY' and a['new_unique_candidate_count']==0
q=load('RECONCILIATION-059.json')['entries'];assert q[38]['contract_id']==a['existing_finding_connections'][0]['id']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [38,*range(1145,1151)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical reads and conditional existing quickstep connection; no product execution.')
