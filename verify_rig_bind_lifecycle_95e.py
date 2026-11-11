#!/usr/bin/env python3
"""Verify audit source and native archive connections, without executing product code."""
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
a=load('ASSESSMENT-RIG-BIND-LIFECYCLE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1389-1392.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1389,1393))
assert [x['lines'] for x in h['historical_source_roles']]==[3461,31,514,73]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1389-1392.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
rig=current[1391];assert b'(Added<AuthoredRigJoint>, Without<MhrBone>)' in rig
assert rig.index(b'commands.entity(entity).insert(SoleAxisCaptured)')<rig.index(b'if let Ok(mut rig) = rigs.get_mut(bone.owner)')
plugin=s['crates/adventuresim-tactical-client/src/animation/presentation/plugin.rs']
bind_pos=plugin.index(b'procedural::bind_humanoid_bones');cache_pos=plugin.index(b'procedural::cache_humanoid_rigs',bind_pos);capture_pos=plugin.index(b'capture_authored_bind_transforms',cache_pos);axis_pos=plugin.index(b'procedural::capture_humanoid_rig_axes',capture_pos)
assert bind_pos<cache_pos<capture_pos<axis_pos
bind=s['crates/adventuresim-tactical-client/src/animation/bind_pose.rs'];assert b'(Added<Transform>, Without<AuthoredBindTransform>)' in bind and b'roots.get(current)' in bind
loading=s['crates/adventuresim-tactical-client/src/animation/loading.rs'];assert b'AnimationRigScene(player)' in loading and b'WorldAssetRoot(scene.clone())' in loading
lock=s['Cargo.lock'];assert all(x in lock for x in [b'5fd2aa472d43b4d081ef41d26300d17bd22cba6d68df2a223eb358782fa40bc6',b'a350b2e4c2e47f7903d688affb8a723b31bca63af9789f1da88703f1ba04e354',b'36855264b9b5d229bde74f01730eb6251c72dc052404ae144a9295cc0abdefc1'])
with tarfile.open(r/'native-evidence/bevy_ecs-0.19.1/bevy_ecs-0.19.1.crate','r:gz') as ar:
 f=ar.extractfile('bevy_ecs-0.19.1/src/query/filter.rs').read();c=ar.extractfile('bevy_ecs-0.19.1/src/schedule/config.rs').read()
 assert b'tick.deref().is_newer_than(fetch.last_run, fetch.this_run)' in f and b'will be inserted on the edge' in c
with tarfile.open(r/'native-evidence/bevy_app-0.19.1/bevy_app-0.19.1.crate','r:gz') as ar:
 d=ar.extractfile('bevy_app-0.19.1/src/main_schedule.rs').read();assert d.index(b'Update.intern(),')<d.index(b'SpawnScene.intern(),')<d.index(b'PostUpdate.intern(),')
with tarfile.open(r/'native-evidence/bevy_world_serialization-0.19.1/bevy_world_serialization-0.19.1.crate','r:gz') as ar:
 lib=ar.extractfile('bevy_world_serialization-0.19.1/src/lib.rs').read();sp=ar.extractfile('bevy_world_serialization-0.19.1/src/world_asset_spawner.rs').read()
 assert b'(world_instance_spawner, world_instance_spawner_system)' in lib and b'pub fn world_instance_spawner_system(world: &mut World)' in sp
 assert sp.index(b'match Self::spawn_sync_internal(world, world_asset_handle.id()')<sp.index(b'Self::set_instance_parent_sync(world, &instance_info)',sp.index(b'match Self::spawn_sync_internal(world, world_asset_handle.id()'))
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {394:[1389],395:[1390,1391,1392]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert q[1186]['contract_id']==a['existing_contract_connections'][0]['id']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1184,1188))
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical reads and qualified rig lifecycle sources; no product execution.')
