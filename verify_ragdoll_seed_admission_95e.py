#!/usr/bin/env python3
"""Verify preserved source, identity and symbolic bounds without product execution."""
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
a=load('ASSESSMENT-RAGDOLL-SEED-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1393-1397.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1393,1398))
assert [x['lines'] for x in h['historical_source_roles']]==[602,237,359,66,336]
assert h['historical_source_roles'][2]['prior_role_status']=='bounded-current-trace-only'
assert h['historical_source_roles'][2]['prior_current_observations']==w['roles'][1395]['current_observations']
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1393-1397.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
entry=current[1395];geometry=current[1396];mod=current[1397]
assert b'(position_next - position_previous) / sample_interval.seconds()' in entry
assert b'linear: if kinematic { Vec3::ZERO } else { linear }' in entry
assert entry.index(b'RagdollGeometry::prepare(')<entry.index(b'let bodies = spawn_bodies(')
assert b'if seed.is_none()' in geometry and b'PositiveLength::from_metres' in geometry and b'SignedLength::from_metres' in geometry
assert b'interval.seconds().max(MINIMUM_VELOCITY_INTERVAL_SECONDS)' in entry
assert b'sample_hz: 30.0' in s['content/tactical/combat.yaml'] and b'maximum_seed_speed_metres_per_second: 10.0' in s['content/tactical/combat.yaml']
rate=s['crates/adventuresim-tactical-core/src/animation/sample_rate.rs'];assert b'if !hertz.is_finite()' in rate and b'if hertz <= 0.0' in rate and b'AnimationElapsed::from_seconds(hertz.recip())' in rate
config=s['crates/adventuresim-tactical-core/src/combat_config/pose_buffer.rs'];assert b'pub sample_hz: AnimationSampleRate' in config and b'pose.maximum_authored_foot_range_metres < pose.minimum_authored_foot_range_metres' in config
lock=s['Cargo.lock'];assert b'glam' in lock and b'0.32.1' in lock
with tarfile.open(r/'native-evidence/glam-0.32.1/glam-0.32.1.crate','r:gz') as ar:
 v=ar.extractfile('glam-0.32.1/src/f32/vec3.rs').read();assert b'max * (self / math::sqrt(length_sq))' in v and b'x: self.x.div(rhs)' in v
# Exact rational inequalities support the magnitude argument, not a runtime result.
f=8*10**37;f32max=Fraction(2**24-1,2**23)*2**127;minimum_normal=Fraction(1,2**126)
assert f<f32max and Fraction(1,f)>minimum_normal and 5*f>f32max
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {396:[1393],397:[1394,1395,1396],398:[1397,1398]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert 'adler/READ-L11-197.json' in records
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries']
assert q[44]['contract_id']==a['existing_finding_connections'][0]['id']=='ADLER-L11-RAGDOLL-SEED-PREFLIGHT'
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1188,1193)]==list(range(1393,1398))
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [44,*range(1188,1193)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical reads and qualified ragdoll seed source connection; no product execution.')
