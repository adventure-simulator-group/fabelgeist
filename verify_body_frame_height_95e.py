#!/usr/bin/env python3
"""Check historical/current sources and conditional policy evidence only."""
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
a=load('ASSESSMENT-BODY-FRAME-HEIGHT-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1356-1362.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1356,1363))
assert [x['lines'] for x in h['historical_source_roles']]==[1813,490,140,211,36,152,355]
historical={};current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1356-1362.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];historical[x['original_index']]=raw;current[x['original_index']]=cur
assert [x['original_index'] for x in h['historical_source_roles'] if not x['current_comparison']['byte_identical']]==[1356,1361,1362]
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
body=current[1357]
for x in [b'owner_transform.rotation.inverse()',b'inverse_body_rotation * skeleton.world_velocity',b'inverse_body_rotation * skeleton.world_acceleration',b'observation.velocity,',b'self.smoothed_body_acceleration,',b'.advance_for_presentation(skeleton, owner_transform, &procedural_clock, &time)']:
 assert x in body
sample=s['crates/adventuresim-tactical-client/src/animation/procedural/sample_state.rs'];assert b'pub(super) struct BodyObservation' in sample and b'pub(super) velocity: BodyLocalVelocity' in sample
assert b'previous_planar.dot(planar_acceleration) < 0.0' in current[1358]
projection=s['crates/adventuresim-tactical-core/src/animation/state/projection.rs']
assert b'(input.linear_velocity - previous_world_velocity) * locomotion_sample_rate().hertz()' in projection
assert b'skeleton.world_acceleration = world_acceleration' in projection and b'skeleton.world_velocity = input.linear_velocity' in projection
assert b'project_skeleton_locomotion_with_body_rotation(' in s['crates/adventuresim-tactical-server/src/player_projection.rs']
oldheight=historical[1362];newheight=current[1362]
assert b'/ run_locomotion_profile().flight_apex_metres.metres()' in oldheight and b'if !run_weight.is_finite()' in oldheight
assert b'let run_weight = speed.clamped_inverse_lerp(' in newheight
assert b'if profile.flight_apex_metres <= HeightDistance::ZERO' in newheight
config=s['crates/adventuresim-tactical-core/src/combat_config/locomotion.rs']
assert b'pub flight_apex_metres: HeightDistance' in config and b'flight_apex_metres' not in config.split(b'impl AnimationLocomotionConfig',1)[1]
assert b'zero-permitting height amplitude' in s['crates/adventuresim-tactical-core/src/animation/height_measurement.rs']
assert b'flight_apex_metres: 0.12' in s['content/tactical/combat.yaml']
assert b'authored_ordinary_passing_rise_metres: 0.033' in s['content/tactical/combat.yaml']
assert b'speed.clamped_inverse_lerp(walk.reference_speed, run_profile.reference_speed)' in s['crates/adventuresim-tactical-core/src/animation/locomotion.rs']
# Exact rational illustration only: unit endpoint weights retain the stated rise.
assert Fraction(33,1000)*1*1==Fraction(33,1000)
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
for batch,ids in {378:[1356],379:[1357,1358],380:[1359,1360,1361],381:[1362,1363,1364]}.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert all(x['original_index'] not in (1363,1364) for x in h['historical_source_roles'])
assert a['existing_finding_connections'][0]['id']=='ADLER-L11-BODY-OBSERVATION-FRAME'
assert a['policy_questions'][0]['id']=='REVIEW-ZERO-RUN-APEX-HEIGHT-POLICY-001' and a['policy_questions'][0]['lens']==8
q=load('RECONCILIATION-059.json')['entries'];assert q[39]['contract_id']==a['existing_finding_connections'][0]['id']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [39,*range(1151,1158)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==a['policy_questions'][0]['id'] for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified seven historical reads and qualified body-frame/height policy connections; no product execution.')
