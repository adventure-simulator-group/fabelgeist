#!/usr/bin/env python3
"""Verify retained camera source identities, qualifications and symbolic bounds."""
import difflib,gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-CAMERA-CLEARANCE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1461-1464.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1461,1462,1463,1464]
assert [x['lines'] for x in h['historical_source_roles']]==[94,339,296,142]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1461-1464.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1463:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-1463/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256'] and b'-        .unwrap();' in d and len(cur.splitlines())==295
  else: assert raw==cur
for x in h['historical_source_roles']:
 if x['original_index']!=1463: assert x['prior_role_status']=='bounded-current-trace-only' and x['prior_current_observations']==w['roles'][x['original_index']]['current_observations']
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
native={}
for x in a['native_connected_sources']:
 if 'archive' in x:
  archive=(r/x['archive']).read_bytes();assert len(archive)==x['archive_bytes'] and sha(archive)==x['archive_sha256']
  with tarfile.open(r/x['archive'],'r:gz') as ar: raw=ar.extractfile(x['member']).read()
  assert len(raw)==x['member_bytes'] and sha(raw)==x['member_sha256'];key=x['member']
 else:
  raw=(r/x['preserved_bytes']).read_bytes();assert len(raw)==x['bytes'] and sha(raw)==x['sha256'];key=x['preserved_bytes']
 assert all(1<=lo<=hi<=len(raw.splitlines()) for lo,hi in x['ranges']);native[key]=raw
direction=native['bevy_math-0.19.1/src/direction.rs'];assert b'let length = value.length();' in direction and b'(length.is_finite() && length > 0.0)' in direction
glam=native['glam-0.32.1/src/f32/vec3.rs'];assert b'math::sqrt(self.dot(self))' in glam
rust=native['native-evidence/rust-nightly-2026-07-13/library/core/src/num/f32.rs'];assert b'pub const fn clamp(mut self, min: f32, max: f32)' in rust and b'min <= max' in rust
assert b'nightly-2026-07-13' in s['rust-toolchain.toml']
collision=current[1462];assert b'let Ok(direction) = Dir3::new(delta) else {' in collision and b'fraction: UnitInterval::from_fraction(1.0)?' in collision and b'hit: None' in collision
assert b'let mut recovery = *self;' in collision and b'let final_sweep = volume.sweep(' in collision and b'*self = recovery;' in collision
rig=s['crates/adventuresim-tactical-client/src/camera/rig.rs'];assert rig.index(b'state.advance_profile(')<rig.index(b'let mut next = state.observe(')<rig.index(b'state.commit_observation(next)')<rig.index(b'camera.translation = observation.final_endpoint.metres()')
assert b'let subject_direction = Dir3::new(subject_delta).unwrap_or(Dir3::NEG_Z)' in s['crates/adventuresim-tactical-client/src/camera/rig/observation.rs'] and b'let muzzle_direction = Dir3::new(muzzle_delta).unwrap_or(camera_direction)' in current[1461]
config=current[1464];assert b'distance: authored_metres(3.75)' in config and b'start_distance: authored_metres(1.8)' in config and b'full_distance: authored_metres(0.65)' in config
measure=s['crates/fabelgeist-geometry/src/measurement.rs'];assert b'pub const fn from_metres(value: f32)' in measure and b'if !value.is_finite()' in measure
smooth=s['crates/adventuresim-tactical-client/src/camera/smoothing.rs'];assert b'let retained_x = x.clamp(-allowed_x, allowed_x);' in smooth
assert Fraction(1,10**30)**2 < Fraction(1,2**150)
allowed_x=Fraction(-1)*Fraction(1,1)*Fraction(16,9)*Fraction(9,100);assert -allowed_x>allowed_x
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {430:[1459,1460,1461],431:[1462],432:[1463,1464]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-207.json']);state=json.loads(records['adler/STATE-L11-207.json']);check=json.loads(records['adler/VERIFY-L11-207.json'])
assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert state['receipt_sha256']==check['receipt_sha256']==m['adler/READ-L11-207.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-207.json']['sha256']
null=check['null_source_range_qualifications'];assert a['historical_null_range_qualification']==null and len(null)==1 and null[0]['original_index']==1463 and null[0]['original_record']['ranges'] is None and null[0]['original_record']['source_read_status'] is None and null[0]['original_record']['trace_refs']==[]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1260,1261,1262,1263];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1461,1462,1463,1464];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [53,54,55]+indices)
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
for x in a['conditional_findings']:assert x['id'] in historical and sum(y['id']==x['id'] for y in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical camera bodies, null-range qualification and bounded source arithmetic; no product execution.')
