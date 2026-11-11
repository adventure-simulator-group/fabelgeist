#!/usr/bin/env python3
"""Verify preserved readiness and repeated-view evidence without product execution."""
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
a=load('ASSESSMENT-CAPTURE-READINESS-COMPARISON-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1417-1421.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1417,1422))
assert [x['lines'] for x in h['historical_source_roles']]==[80,32,214,136,126]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1417-1421.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
ready=current[1418];assert b'if !animation_runtime.motion_is_processed(motion)' in ready and b'self.motion_ready_frames = 0' in ready and b'self.motion_ready_frames < 2' in ready
capture=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'let Some(playback) = playback else {\n            return;' in capture and b'if !sequence.ready_for_frame(&frame, &animation_runtime, observed_playback)' in capture
assert b'!sequence.applied || sequence.capture_in_flight' in capture and b'sequence.waiting += 1' in capture
assert capture.index(b'if !sequence.ready_for_frame')<capture.index(b'sequence.applied = true')
loader=s['crates/adventuresim-tactical-client/src/animation/loading.rs'];assert b'runtime.unavailable_motions.insert(key)' in loader and b'runtime.clips.insert(key, clip)' in loader
runtime=s['crates/adventuresim-tactical-client/src/animation/mod.rs'];assert b'self.clips.keys().any(|key| &key.motion == motion)' in runtime
harness=s['crates/adventuresim-tactical-client/src/animation_viewer/harness.rs'];assert b'let Some(harness) = capture.harness else' in harness and b'capture.waited >= CAPTURE_LOAD_FRAME_LIMIT' in harness
repeat=current[1419];assert b'if self.view_index == 0' in repeat and b'(position_delta > 0.0005 || rotation_delta > 0.05)' in repeat
assert b'(Some(expected), Some(actual)) => expected.distance(actual) <= 0.0005' in repeat
assert b'if sequence.view_index == 0' in capture and b'sequence.samples.push(FrameSample' in capture
model=s['crates/adventuresim-tactical-client/src/animation_viewer/report/model.rs'];assert b'bone.position.into_iter().all(f32::is_finite)' in model and b'bone.rotation_xyzw.into_iter().all(f32::is_finite)' in model
with tarfile.open(r/'native-evidence/glam-0.32.1/glam-0.32.1.crate','r:gz') as ar:
 v=ar.extractfile('glam-0.32.1/src/f32/vec3.rs').read();assert b'pub fn distance(self, rhs: Self) -> f32' in v and b'(self - rhs).length()' in v
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {407:[1416,1417],408:[1418,1419,1420,1421]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
for path in ('adler/READ-L11-178.json','adler/READ-L11-201.json','adler/READ-L11-203.json'):assert path in records
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries'];indices=[1212,1214,1215,1216,1217]
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==list(range(1417,1422))
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [35,*indices])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']=='REVIEW-CAPTURE-PREAPPLIED-READINESS-TIMEOUT-001' for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical reads and qualified capture readiness/comparison sources; no product execution.')
