#!/usr/bin/env python3
"""Verify bounded camera-look and fixed-tick readiness relationships."""
import gzip
import json
import tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda p:json.loads((R/p).read_text())
name='ASSESSMENT-DELEGATED-RECONCILIATION-13C-95E.json';a=load(name)
ids=['CONT-CAMERA-OWNERDEF-PP-F001','CURRENT-FIXED-TICK-POSE-CACHE-READINESS']
assert [x['id'] for x in a['central_observations']]==a['new_qualified_ledger_rows']==ids
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in ids:
 row=ledger[ident];assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed' and row['discovery'].startswith('partial')
with tarfile.open(R/'delegated/look-queue-2559-reconcile-evidence.tar.gz') as t:
 old=json.loads(gzip.decompress(t.extractfile('history/774/05-findings-supplement-002.json.gz').read()))
 assert old['id']==ids[0] and old['source_id']=='physical-p16:crates/adventuresim-tactical-client/src/animation/procedural.rs'
 assert old['classification']=='raw_producer_semantic_angle_pair_propagation_strengthening; not checked-owner loss'
reg=load(a['central_display_evidence']['registration']);assert a['source_references']==[{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']]
with tarfile.open(R/'delegated/central-13c-windows-proof.tar.gz') as t:
 p=json.load(t.extractfile('proof.json'));src={(x['revision'],x['path']):t.extractfile(p['objects'][x['blob']]['archive']).read().decode().splitlines() for x in p['paths']}
cur=a['source_revision'];root='crates/'
def line(path,n):return src[(cur,root+path)][n-1]
proc='adventuresim-tactical-client/src/animation/procedural.rs';resp='adventuresim-tactical-core/src/animation/response_parameters.rs';plugin='adventuresim-tactical-client/src/animation/presentation/plugin.rs';server='adventuresim-tactical-server/src/player_projection.rs';bot='adventuresim-tactical-server/src/bot.rs'
assert 'pub(super) struct FixedTickPoseCache' in line(proc,82) and 'cache.bones.clear()' in line(proc,118) and 'cache.tick != Some(tick)' in line(proc,121)
assert 'state.base_rotation' in line(proc,146) and 'guarded_camera_look' in line(proc,180) and '.collect::<BTreeMap' in line(proc,190)
assert 'Vec2::new(' in line(proc,705) and '.distribute_radians(yaw)' in line(proc,708) and '.clamp(-joint_limit, joint_limit)' in line(proc,709)
assert 'checked_nonnegative' in line(resp,128) and 'value == 0.0' in line(resp,129) and 'radians / self.0' in line(resp,138)
assert 'restore_procedural_look_base' in line(plugin,74) and 'apply_head_and_torso_look' in line(plugin,84) and '.chain()' in line(plugin,93)
assert '!look.is_finite()' in line(server,1386) and 'look.y.clamp(-1.5, 1.5)' in line(server,1408) and 'look.yaw =' in line(bot,306)
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified historical F001 attribution, bounded look/cache source lines and two partial ledger relationships; no product execution or lens closure.')
