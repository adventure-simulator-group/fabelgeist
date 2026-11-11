#!/usr/bin/env python3
"""Verify preserved audit evidence and bounded IK source connections only."""
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
a=load('ASSESSMENT-IK-YAW-ORIENTATION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1370-1374.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1370,1375))
assert [x['lines'] for x in h['historical_source_roles']]==[2973,453,530,165,44]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1370-1374.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines']
  before=raw.splitlines(keepends=True);after=cur.splitlines(keepends=True);assert len(before)==len(after) and [j+1 for j,(x,y) in enumerate(zip(before,after)) if x!=y]==comp['differing_line_numbers'];current[x['original_index']]=cur
assert [x['original_index'] for x in h['historical_source_roles'] if not x['current_comparison']['byte_identical']]==[1370]
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
coord=current[1370];orientation=current[1372];yaw=current[1373];observation=current[1371]
assert b'Without<crate::animation::pose_buffer::PoseCulled>' in coord and b'runtime_inverse_kinematics_config()' in coord
assert coord.count(b'finalize_leg_rotation_chains(')==2 and coord.index(b'if locomotion::owns(skeleton)')<coord.index(b'finalize_leg_rotation_chains(')
assert b'let mut final_offsets = [AngularMagnitudeDegrees::ZERO; 2]' in yaw and b'Ok(None) => {}' in yaw
assert b'left_knee_foot_yaw_offset_degrees = final_offsets[0]' in yaw and b'right_knee_foot_yaw_offset_degrees = final_offsets[1]' in yaw
assert b'if airborne_orientation_owned[leg_index] || contact_blend_active' in orientation
assert b'local_rotation_for_world(foot_snapshot.parent_rotation, bounded_world)' in orientation
assert b'memory.left_foot_orientation_world = Some(final_world)' in orientation
assert b'if locomotion::owns(skeleton)' in observation
foot=load('ASSESSMENT-L01-HISTORICAL-1363-1369.json')['historical_source_roles'][2];roll=gzip.decompress((r/foot['current_comparison']['preserved_bytes']).read_bytes())
assert foot['original_index']==1365 and b'skeleton.action_kind() != SkeletonAction::None' in roll and b'skeleton.weapon_guard() != WeaponGuardState::Lowered' in roll
core=s['crates/adventuresim-tactical-core/src/animation/state.rs'];assert b'Self(ActionKind::Dodge { .. }) => SkeletonAction::Dodge' in core and b'pub fn is_quickstep(&self)' in core
state=s['crates/adventuresim-tactical-client/src/animation/procedural/ik/state.rs'];assert b'pub left_knee_foot_yaw_offset_degrees: AngularMagnitudeDegrees' in state and b'left_knee_foot_yaw_offset_degrees: self.0.left_knee_foot_yaw_offset_degrees' in state
capture=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'leg_ik.map(LegIkState::diagnostics).unwrap_or_default()' in capture and b'ik_left_knee_foot_yaw_offset_degrees: leg_ik' in capture
metrics=s['crates/adventuresim-tactical-client/src/animation_viewer/report/metrics.rs'];assert b'pub(super) fn maximum_knee_foot_yaw_offset' in metrics and b'.fold(0.0, f32::max)' in metrics
validation=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs'];assert b'metrics.maximum_knee_foot_yaw_offset_degrees <= 22.6' in validation and b'&& validation.biomechanics_within_review_bounds' in validation
report=s['crates/adventuresim-tactical-client/src/animation_viewer/report/mod.rs'];assert b'.all(|(role, _)| frame.bones.contains_key(*role))' in report
bones=s['crates/adventuresim-tactical-client/src/animation_viewer/report/bones.rs'];assert all(x in bones for x in [b'BoneRole::ThighLeft',b'BoneRole::ShinLeft',b'BoneRole::FootLeft',b'BoneRole::ToeLeft',b'BoneRole::ToeRight'])
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {384:[1370],385:[1371],386:[1372,1373],387:[1374,1375]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
prior=json.loads(records['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-087.json']);text=' '.join(prior['source_contract_assessment']['observations']);assert 'cannotreceive nonzero runroll' in text
assert [x['id'] for x in a['existing_finding_connections']]==['ADLER-L11-KNEE-YAW-MISSING-OBSERVATION-ZERO','ADLER-L11-FOOT-ORIENTATION-REQUEST-AS-COMMITTED']
q=load('RECONCILIATION-059.json')['entries'];assert [q[i]['contract_id'] for i in (42,43)]==[x['id'] for x in a['existing_finding_connections']]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [42,43,*range(1165,1170)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical reads and qualified yaw/orientation source connections; no product execution.')
