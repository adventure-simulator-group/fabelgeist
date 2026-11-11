#!/usr/bin/env python3
"""Verify historical body identities and bounded static setup/view evidence."""
import difflib
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
a=load('ASSESSMENT-CAPTURE-SETUP-VIEW-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1446-1448.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed' and a['conditional_findings']==[]
assert [x['original_index'] for x in h['historical_source_roles']]==[1446,1447,1448]
assert [x['lines'] for x in h['historical_source_roles']]==[94,74,131]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1446-1448.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1448:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-1448/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256'] and b'+mod hud;\n' in d and len(cur.splitlines())==132
  else: assert raw==cur
v=h['historical_source_roles'][1];assert v['prior_role_status']=='bounded-current-trace-only' and v['prior_current_observations']==[{'assessment':'ASSESSMENT-L1-CAPTURE-ORBIT.json','ranges':[[1,74]]},{'assessment':'ASSESSMENT-L01-CAMERA-ROOT.json','ranges':[[1,74]]}]
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
setup=current[1446];view=current[1447];main=current[1448]
assert b'CharacterId(default_tactical_character_id())' in setup and b'if let Some(proportions) = proportions.0' in setup
assert b'CharacterProportions::from_seed(' in setup and b'Seed::from_u64(default_tactical_character_id())' in setup
assert b'CharacterProportions::from_seed(fabelgeist_determinism::Seed::from_u64(character_id.0))' in s['crates/adventuresim-tactical-client/src/animation/identity_morphs.rs']
assert b'CharacterProportions::from_seed(fabelgeist_determinism::Seed::from_u64(id.0))' in s['crates/adventuresim-tactical-client/src/animation/skeletal_proportions.rs']
assert b'setup_tactical_presentation,' in s['crates/adventuresim-tactical-client/src/presentation/mod.rs']
env=s['crates/adventuresim-tactical-client/src/presentation/environment.rs'];assert b'TacticalGameplayCamera,\n        Camera3d::default()' in env
assert b'(subjects.single(), cameras.single_mut())' in view and b'animation_capture_camera_offset(RigidRotation::IDENTITY)' in view and b'64 Hz frame {}' in view
schedule=s['crates/adventuresim-tactical-client/src/animation_viewer/capture/capture_schedule.rs'];assert b'.after(TacticalCameraSet::Offset)' in schedule and b'.before(TransformSystems::Propagate)' in schedule and b'Last,' in schedule and b'capture_frame' in schedule
assert b'combat_config.animation.locomotion.sample_hz' in main and b'1000/64*Number(rate.value)' in s['crates/adventuresim-tactical-client/src/animation_viewer/report/html.rs']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {421:[1444,1445,1446],422:[1447,1448]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1244,1245,1246]
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1446,1448,1447]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified three historical bodies and bounded static setup/view connections; no product execution.')
