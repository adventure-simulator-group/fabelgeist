#!/usr/bin/env python3
"""Verify source identities, recovery lineage and symbolic bounds only."""
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
a=load('ASSESSMENT-POSE-LEG-BOUNDS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1328-1333.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1328,1334))
assert [x['lines'] for x in h['historical_source_roles']]==[810,43,155,1892,240,59]
historical={};current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1328-1333.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines']
  historical[x['original_index']]=raw;current[x['original_index']]=cur
assert [x['original_index'] for x in h['historical_source_roles'] if not x['current_comparison']['byte_identical']]==[1331]
assert h['historical_source_roles'][0]['prior_role_status']=='bounded-current-trace-only'
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
pose=current[1331]
for fragment in [b'let direction = offset.try_normalize()?;',b'(upper_length - lower_length).abs() + 0.0001,',b'upper_length + lower_length - 0.0001,',b'let upper_length = hip.distance(knee);',b'let lower_length = knee.distance(ankle);',b'if terrain_world.distance(foot_world) <= 0.0001',b'contact_plants: ContactPlantPolicy::Reset',b'pose_cull::cull_pose_rig']:
 assert fragment in pose
assert b'PoseCulled' not in historical[1331]
rig=s['crates/adventuresim-tactical-client/src/animation/pose_buffer/rig.rs']
assert b'LocalPose::from_transform(bind.local)' in rig
assert b'if found.is_empty()' in rig
assert b'let current = definition' in rig
assert b'local: *transform' in s['crates/adventuresim-tactical-client/src/animation/bind_pose.rs']
assert b'local_pose_global' in s['crates/adventuresim-tactical-client/src/animation/pose_buffer/hierarchy.rs']
assert b'pose.translation += offset.0' in s['crates/adventuresim-tactical-client/src/animation/pose_buffer/proportions.rs']
assert b'pub(in crate::animation::procedural) fn owns' in s['crates/adventuresim-tactical-client/src/animation/procedural/ik/locomotion.rs']
assert b'ik_tuning().measured_ankle_sole_offset_metres' in s['crates/adventuresim-tactical-client/src/animation/procedural/ik/terrain_contacts/foot_clearance.rs']
assert b'commands.entity(owner).insert(PoseCulled)' in s['crates/adventuresim-tactical-client/src/animation/pose_buffer/pose_cull.rs']
primitive=(r/'native-evidence/rust-nightly-2026-07-13/library/core/src/num/f32.rs').read_bytes()
assert b'Panics if `min > max`, `min` is NaN, or `max` is NaN.' in primitive
assert b'pub const fn clamp(mut self, min: f32, max: f32)' in primitive
with tarfile.open(r/'native-evidence/glam-0.32.1/glam-0.32.1.crate','r:gz') as ar:
 vec=ar.extractfile('glam-0.32.1/src/f32/vec3.rs').read()
assert b'pub fn try_normalize(self) -> Option<Self>' in vec and b'if rcp.is_finite() && rcp > 0.0' in vec
with tarfile.open(r/'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate','r:gz') as ar:
 math=ar.extractfile('bevy_math-0.19.1/src/lib.rs').read()
assert b'pub use glam::*;' in math
# Exact rational argument only: the chosen margin is larger than an f32 ULP
# around one, so opposite sides of one remain ordered after representation.
margin=Fraction(1,10000)
assert Fraction(1,1)+margin>Fraction(1,1)-margin and margin>Fraction(1,2**23)
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob'])
 records[x['original_path']]=json.loads(raw)
for batch,ids in {365:[1328],366:[1329,1330],367:[1331],368:[1332,1333]}.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert a['candidates'][0]['id']=='REVIEW-POSE-LEG-ZERO-SEGMENT-CLAMP-001' and a['candidates'][0]['lens']==4
q=load('RECONCILIATION-059.json')['entries'];assert q[36]['contract_id']=='ADLER-L11-POSE-LEG-ROLE-REDECODING'
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [36,*range(1123,1129)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==a['candidates'][0]['id'] for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical reads and conditional leg-range source trace; no product execution.')
