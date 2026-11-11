#!/usr/bin/env python3
"""Verify camera history bodies and bounded source connections, without execution."""
import difflib,gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-CAMERA-HISTORY-POLICY-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1465-1469.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed' and not a['conditional_findings']
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1465,1470))
assert [x['lines'] for x in h['historical_source_roles']]==[97,170,105,269,105]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1465-1469.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1469:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-1469/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256'] and b'+    Fraction(#[from]' in d and len(cur.splitlines())==107
  else:
   assert raw==cur and x['prior_role_status']=='bounded-current-trace-only' and x['prior_current_observations']==role['current_observations']
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
rig=current[1465];assert rig.index(b'state.advance_profile(')<rig.index(b'let mut next = state.observe(')<rig.index(b'state.commit_observation(next)')<rig.index(b'camera.translation = observation.final_endpoint.metres()')
history=current[1467];assert b'Some(history) if !reset => history.clone()' in history and b'self.blend = blend;' in history and b'self.blend_velocity = velocity;' in history
obs=current[1466];assert b'let placement = history.boom.place(' in obs and b'let soft_occlusion = spatial.cast_ray_predicate(' in obs
collision=s['crates/adventuresim-tactical-client/src/camera/collision.rs'];assert b'let mut recovery = *self;' in collision and b'let final_sweep = volume.sweep(' in collision and b'*self = recovery;' in collision
player=s['crates/adventuresim-tactical-client/src/player.rs'];assert b'camera: Single<Entity, With<TacticalGameplayCamera>>' in player and b'.insert(CharacterControllerCameraOf::new(event.entity))' in player
assert b'TacticalGameplayCamera,\n        Camera3d::default(),\n        Projection::Perspective' in s['crates/adventuresim-tactical-client/src/presentation/environment.rs']
shaping=s['crates/adventuresim-tactical-core/src/animation/shaping_time.rs'];assert b'Each response kernel owns' in shaping and b'if !value.is_finite()' in shaping
smooth=current[1468];assert b'smooth_time.seconds().max(0.0001)' in smooth and b'Projection::Perspective(perspective)' in smooth and b'aspect: Ratio::from_ratio(16.0 / 9.0)' in smooth
values=current[1469];assert b'value.x < 0.0 || value.y < 0.0' in values and b'if !value.is_finite()' in values
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {433:[1465,1466,1467],434:[1468,1469]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=list(range(1264,1269));assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==list(range(1465,1470));assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical camera bodies and bounded staging/policy links; no product execution.')
