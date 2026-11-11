#!/usr/bin/env python3
"""Verify source lineage and exact symbolic reasoning without product execution."""
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
a=load('ASSESSMENT-INERTIAL-COEFFICIENTS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1334-1339.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1334,1340))
assert [x['lines'] for x in h['historical_source_roles']]==[447,17,146,443,42,110]
historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1334-1339.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines']
  historical[x['original_index']]=raw
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
channel=historical[1337]
for fragment in [b'let (t2, t3) = (duration * duration, duration * duration * duration);',b'let a0 = (-8.0 * v0 * duration - 20.0 * x0) / t2;',b'let a = -(a0 * t2 +',b'duration: AnimationElapsed::from_seconds(duration)?',b'let translation = QuinticChannel::new(',b'self.translation = translation;',b'translation: input.translation + self.translation.offset(self.elapsed)']:
 assert fragment in channel
config=s['crates/adventuresim-tactical-core/src/combat_config/pose_buffer.rs']
assert b'pub inertial_blend_seconds: PositiveAnimationDuration' in config
assert b'inertial_blend_seconds' not in config.split(b'impl PoseBufferConfig',1)[1]
measurement=s['crates/adventuresim-tactical-core/src/animation/measurement.rs']
assert b'if value.is_zero()' in measurement and b'Self::from_elapsed(AnimationElapsed::from_seconds(value)?)' in measurement
elapsed=s['crates/adventuresim-core/src/elapsed.rs'];assert b'if !seconds.is_finite()' in elapsed and b'if seconds < 0.0' in elapsed
assert b'animation.pose_buffer.validate()?' in s['crates/adventuresim-tactical-core/src/combat_config.rs']
runtime=s['crates/adventuresim-tactical-core/src/combat_config/runtime.rs'];assert b'pub fn install_runtime_snapshot(&self)' in runtime and b'self.validate()?;' in runtime
assert b'snapshot.0.install_runtime_snapshot()' in s['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'inertial_blend_seconds: 0.2' in s['content/tactical/combat.yaml']
pose=s['crates/adventuresim-tactical-client/src/animation/pose_buffer.rs']
for fragment in [b'pose_tuning().inertial_blend_seconds',b'rig.capture_transition_offsets(&target, &target_velocities)',b'rig.offsets[joint].peek(input)',b'transform.translation = pose.translation;']:
 assert fragment in pose
assert b'let sample_phase = self.locomotion_strides.sample_phase(sample.pose, phase)?' in s['crates/adventuresim-tactical-client/src/animation/resolver.rs']
assert b'weighted.locomotion_phase.is_some()' in pose
assert b'amount: UnitInterval::from_fraction(t * t * (3.0 - 2.0 * t))?' in historical[1339]
assert b'sparse anchor blending advances linearly with phase' in s['crates/adventuresim-tactical-client/README.md']
assert Fraction(1,4)*Fraction(1,4)*(3-2*Fraction(1,4))==Fraction(5,32)
primitive=gzip.decompress((r/'native-evidence/rust-nightly-2026-07-13/library/core/src/primitive_docs.rs.gz').read_bytes())
assert b'When a primitive operation (addition, subtraction, multiplication' in primitive
assert b"the result is \xe2\x88\x9e or \xe2\x88\x92\xe2\x88\x9e" in primitive
# Exact integer range check only; no f32 multiplication or product code runs.
f32_max=(2**24-1)*2**104
assert f32_max*f32_max>f32_max+2**103
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
for batch,ids in {369:[1334,1335],370:[1336,1337],371:[1338,1339,1340]}.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert all(x['original_index']!=1340 for x in h['historical_source_roles'])
assert a['candidates'][0]['id']=='REVIEW-INERTIAL-BLEND-COEFFICIENT-OVERFLOW-001' and a['candidates'][0]['lens']==4
assert a['existing_finding_connections'][0]['id']=='ADLER-L11-LOCOMOTION-ANCHOR-REPARAMETERIZATION'
q=load('RECONCILIATION-059.json')['entries'];assert q[37]['contract_id']==a['existing_finding_connections'][0]['id']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [37,*range(1129,1135)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']==a['candidates'][0]['id'] for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical reads, inertial source trace and earlier policy connection; no product execution.')
