#!/usr/bin/env python3
"""Verify preserved audit evidence and bounded source connections only."""
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
a=load('ASSESSMENT-IK-SUPPORT-CONNECTIONS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1363-1369.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1363,1370))
assert [x['lines'] for x in h['historical_source_roles']]==[39,23,118,54,391,526,121]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1363-1369.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
loc=current[1368];leg=current[1369]
for x in [b'fn owns(skeleton: &SkeletonState)',b'let solve_weights = if enabled.is_enabled() && terrain.is_some()',b'FootIkInfluence::NONE',b'let mut support = if enabled.is_enabled()',b'.and_then(|point| terrain.height_at(point))',b'ordinary_terrain_target(foot_position, height, weight)',b'support[index] = observed',b'memory.publish(owner, &mut diagnostics, &mut commands)']:
 assert x in loc,x
assert loc.index(b'apply_root_vertical_shift(rig, pelvis_shift')<loc.index(b'memory.publish(owner')<loc.index(b'*current = state')
assert leg.index(b'target.distance(foot_snapshot.global.translation()) <= 0.0001')<leg.index(b'return Some(self.weight)')<leg.index(b'self.align_surface(')
assert b'let solution = solve_two_bone_with_reach(' in leg
obs=s['crates/adventuresim-tactical-client/src/animation/procedural/ik/observation.rs']
assert obs.index(b'any(|rotation| !rotation.is_finite())')<obs.index(b'self.left_last_rendered_world = left_last_rendered_world')
assert b'if locomotion::owns(skeleton)' in obs and b'refresh_run_support(skeleton, speed, rig, terrain' in obs
assert b'crate::animation::TerrainIkMode::Enabled,' in obs and b'terrain?.height_at(point)?' in obs and b'        nominal_support\n' in obs
plugin=s['crates/adventuresim-tactical-client/src/animation/presentation/plugin.rs']
assert plugin.index(b'procedural::apply_terrain_leg_ik')<plugin.index(b'.before(TransformSystems::Propagate)')<plugin.index(b'procedural::refresh_raised_support_after_propagation')<plugin.index(b'.after(TransformSystems::Propagate)')
queries=s['crates/adventuresim-tactical-core/src/scene/queries.rs'];assert b'pub fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation>' in queries
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {381:[1362,1363,1364],382:[1365,1366,1367],383:[1368,1369]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
for n in (70,72,73,74,75):
 receipt=json.loads(records[f'root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-{n:03}.json'])
 assert receipt['lens']==1 and receipt['accepted_unique_count_change']==0
 assert receipt['registration_sha256']==original[receipt['registration']]['sha256']
e275='root-l1-current-e275-20261029/objects/'
assert records[e275+sha(loc)]==loc and records[e275+sha(obs)]==obs
assert [x['id'] for x in a['existing_contract_connections']]==['L1-P06-IK-TYPED-SUPPORT-PROPAGATION','L1-P19-RAISED-SUPPORT-OBSERVATION-DISPATCH']
q=load('RECONCILIATION-059.json')['entries'];assert [q[i]['contract_id'] for i in (2130,2131)]==[x['id'] for x in a['existing_contract_connections']]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [*range(1158,1165),2130,2131])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified seven historical reads and partial IK support connections; no product execution.')
