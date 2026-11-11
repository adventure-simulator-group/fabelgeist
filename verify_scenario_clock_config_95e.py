#!/usr/bin/env python3
"""Verify source identities and symbolic timing evidence, without product execution."""
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
a=load('ASSESSMENT-SCENARIO-CLOCK-CONFIG-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1441-1445.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1441,1442,1443,1444,1445]
assert [x['lines'] for x in h['historical_source_roles']]==[1409,43,190,183,40]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1441-1445.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
main=s['crates/adventuresim-tactical-client/src/animation_viewer_main.rs'];assert b'combat_config.animation.locomotion.sample_hz' in main and b'.install_runtime_snapshot()' in main and b'combat_config,' in main
quick=current[1444];assert quick.count(b'let config = TacticalCombatConfig::default();')>=5 and b'quickstep_action_contact_ticks(config.movement.maneuvers.quickstep_duration_seconds)' in quick
subject=s['crates/adventuresim-tactical-client/src/animation_viewer/capture/subject_motion.rs'];assert b'start.saturating_add(quickstep_action_ticks().half())' in subject
server=s['crates/adventuresim-tactical-server/src/player_projection.rs'];assert b'config.movement.maneuvers.quickstep_duration_seconds' in server
cfg=s['crates/adventuresim-tactical-core/src/combat_config.rs'];validate=cfg[cfg.index(b'pub fn validate(&self)'):cfg.index(b'pub fn digest(&self)')];assert b'quickstep_duration_seconds' not in validate
assert b'defaults::canonical().clone()' in cfg and b'content/tactical/combat.yaml' in s['crates/adventuresim-tactical-core/src/combat_config/defaults.rs']
yaml=s['content/tactical/combat.yaml'];assert b'quickstep_duration_seconds: 0.50' in yaml and b'sample_hz: 64.0' in yaml
assert Fraction(1,2)*64/2==16 and Fraction(1,1)*64/2==32
scenario=current[1441]
for start,end in [(b'fn turning_scenario(',b'fn dynamics_turn_scenario('),(b'fn dynamics_turn_scenario(',b'fn guard_plant_turn_scenario('),(b'fn guard_plant_turn_scenario(',b'fn raised_guard_stationary_turn_scenario(')]:
 section=scenario[scenario.index(start):scenario.index(end)];assert b'frame as f32 / 64.0' in section and b'time_seconds: authored_frame_time(progress)' in section
assert Fraction(64,64)==1 and 64*Fraction(1,32)==2 and Fraction(1,64)*2==Fraction(1,32)
clock=s['crates/adventuresim-tactical-client/src/animation_viewer/capture/frame_clock.rs'];assert b'locomotion_sample_rate().interval().elapsed()' in clock
cap=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'time_seconds: frame.time_seconds' in cap
jitter=s['crates/adventuresim-tactical-client/src/animation/jitter.rs'];assert b'.abs_diff(b.time_seconds)' in jitter and b'/ dt(frames, index + 1).seconds()' in jitter
assert b'1000/64*Number(rate.value)' in s['crates/adventuresim-tactical-client/src/animation_viewer/report/html.rs']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {419:[1441],420:[1442,1443],421:[1444,1445,1446]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=list(range(1239,1244))
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1441,1442,1443,1444,1445]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ('REVIEW-CAPTURE-QUICKSTEP-DEFAULT-CONFIG-001','REVIEW-CAPTURE-SCENARIO-TIMESTAMP-RATE-001'):assert sum(x['id']==ident for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical scenario bodies and symbolic selected-config/clock connections; no product execution.')
