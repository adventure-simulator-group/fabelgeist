#!/usr/bin/env python3
"""Verify metric/support evidence without running the product."""
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
a=load('ASSESSMENT-REPORT-METRIC-SUPPORT-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1433-1436.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed' and a['conditional_findings']==[]
assert [x['original_index'] for x in h['historical_source_roles']]==[1433,1434,1435,1436]
assert [x['lines'] for x in h['historical_source_roles']]==[2050,65,50,55]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1433-1436.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
plugin=s['crates/adventuresim-tactical-client/src/animation/presentation/plugin.rs'];assert b'procedural::apply_terrain_leg_ik' in plugin and b'.before(TransformSystems::Propagate)' in plugin and b'procedural::refresh_raised_support_after_propagation' in plugin and b'.after(TransformSystems::Propagate)' in plugin
ik=s['crates/adventuresim-tactical-client/src/animation/procedural/ik/mod.rs'];assert b'footwork.left_support_weight = observed' in ik and b'memory.left_support_weight = support_observation' in ik
obs=s['crates/adventuresim-tactical-client/src/animation/procedural/ik/observation.rs'];section=obs[obs.index(b'pub(in crate::animation) fn refresh_raised_support_after_propagation'):obs.index(b'pub(super) fn observed_support_weight')]
assert b'state.0.left_support_weight = support' in section and b'raised.left_solve_target = Some(ankle)' in section and b'raised.left_support_weight =' not in section
state=s['crates/adventuresim-tactical-client/src/animation/procedural/ik/state.rs'];assert b'left_support_weight: self.0.left_support_weight' in state
cap=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'.map(|state| (state.left_support_weight, state.right_support_weight))' in cap and b'ik_left_support_weight: leg_ik.left_support_weight' in cap
assert b'Last,' in s['crates/adventuresim-tactical-client/src/animation_viewer/capture/capture_schedule.rs']
with tarfile.open(r/'native-evidence/bevy_app-0.19.1/bevy_app-0.19.1.crate','r:gz') as ar:
 native=ar.extractfile('bevy_app-0.19.1/src/main_schedule.rs').read();assert b'PostUpdate.intern(),\n                Last.intern(),' in native
metrics=current[1433];assert b'if is_quickstep_scenario(&frame.scenario)' in metrics and b'let quickstep_toe_contact = if is_quickstep_scenario(&frame.scenario)' in metrics
assert metrics.index(b'if is_quickstep_scenario(&frame.scenario)')<metrics.index(b'let quickstep_toe_contact')
assert b'ik_left_knee_foot_yaw_offset_degrees' in metrics and b'.fold(0.0, f32::max)' in metrics
slip=current[1436];assert b'support >= 0.9' in slip and b'body_turn <= 0.5' in slip and b'if !distance.is_finite()' in slip
assert b'if samples.len() < 3' in current[1435] and b'if phase_span >= 0.8' in metrics
val=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs'];assert b'metrics.maximum_knee_foot_yaw_offset_degrees <= 22.6' in val and b'let reported_support_contacts_valid = reported_support_contacts_are_valid(&frames)' in val
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {414:[1433],415:[1434,1435,1436]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
for n in (552,557):
 x=json.loads(records[f'segovb/READ-L11-{n}.json']);assert x['actual_current_target']=='e2755a732c84e67296b5e3441a4259a84f461b2f' and x['new_target_reconciliation_complete'] is False and x['accepted_unique_count_change']==0
for n in (200,202,203):
 x=json.loads(records[f'adler/READ-L11-{n}.json']);assert x['current_reconciled'] is False and x['accepted_unique_count_change']==0
assert 'adler/VERIFY-L11-203.json' not in m and 'adler/READ-L11-203.json' in records and 'adler/STATE-L11-203.json' in records
search=a['targeted_local_evidence_search'];assert search['query']=='Exact regular filename VERIFY-L11-203.json'
assert len(search['collections'])==3 and all(not x['matching_relative_paths'] and x['error_count']==0 for x in search['collections'])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1231,1232,1233,1234]
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1433,1434,1435,1436]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [42,45,*indices])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical metrics bodies, attributed earlier support/yaw evidence and bounded 95e connections; no product execution.')
