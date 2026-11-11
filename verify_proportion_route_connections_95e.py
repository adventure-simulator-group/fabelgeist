#!/usr/bin/env python3
"""Verify retained proportion/route source evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-PROPORTION-ROUTE-CONNECTIONS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1406-1409.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1406,1410))
assert [x['lines'] for x in h['historical_source_roles']]==[84,91,145,279]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1406-1409.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
core=s['crates/adventuresim-core/src/character_proportions.rs'];assert b'Self::UpperArmLength | Self::LowerArmLength | Self::LowerLegLength => 1.0' in core
assert b'self.translation_metres[p.index()][axis]' in core and b'(proportions.get(*p) - self.reference.get(*p))' in core and b'.all(|value| value.is_finite())' in core
body=current[1409];assert b'basis.is_finite()' in body and b'basis.0.translation(proportions)' in body and b'SkeletalJointOffset(bone.offset)' in body
fit=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/rig.rs'];assert b'reference.is_none_or(|r| r == basis.reference)' in fit and b'basis.translation(proportions)' in fit and b'*position = (Vec3::from_array(*position) + offset).to_array()' in fit
pose=s['crates/adventuresim-tactical-client/src/animation/pose_buffer/proportions.rs'];assert b'pose.translation += offset.0' in pose
producer=s['crates/adventuresim-character-creator/src/proportions.rs'];assert b'JointProportionBasis {' in producer and b'reference,' in producer
route=current[1408];assert b'let evaluation = AnimationEvaluation::from_skeleton(skeleton)?' in route and b'requested_path: path,\n        path,' in route
capture=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'let Some(semantic_route) = semantic_route else' in capture and b'semantic_route_runtime_evaluated: true' in capture
validation=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs'];assert b'frame.semantic_route_selected_path == frame.semantic_route_requested_path' in validation and b'validation.semantic_route_paths_exercised' in validation
# Magnitude comparison only: no asset, f32 kernel, product or runtime was run.
f32max=Fraction(2**24-1,2**23)*2**127;coefficient=2*10**38
assert coefficient<f32max and coefficient*(1-(-1))>f32max
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {401:[1404,1405,1406],402:[1407,1408,1409]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries']
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1201,1205)]==list(range(1406,1410))
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1201,1205))
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']=='REVIEW-SKELETAL-BASIS-RESULT-OVERFLOW-001' for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical reads and qualified proportion/route evidence; no product execution.')
