#!/usr/bin/env python3
"""Verify preserved scenario admission evidence without product or macro execution."""
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
a=load('ASSESSMENT-SCENARIO-LABEL-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1401-1405.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1401,1406))
assert [x['lines'] for x in h['historical_source_roles']]==[277,119,206,238,115]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1401-1405.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
metadata=current[1405];catalog=current[1402];policy=current[1403];grammar=current[1404];classification=current[1401]
assert b'label.parse::<ScenarioId>().ok()' in metadata and b'ScenarioFeature::from_open_label(label)' in metadata and b'self.label == other.label' in metadata
assert b'#[strum(serialize = "terrain-turn-90")]' in catalog and b'TerrainTurn90' in catalog
assert b'ScenarioLabelPattern::Substring("run")' in grammar and b'ScenarioLabelPattern::Prefix("terrain-")' in grammar
assert b'Self::RaisedGuardTapStopLeft | Self::RaisedGuardTapStopRight' in policy and b'ScenarioFeature::RaisedTapStop' in policy
assert b'ScenarioTerrain::admit(&name, kind)' in classification and b'ScenarioId::FullRagdoll) => Some(BodyState::Ragdolled)' in classification
cli=s['crates/adventuresim-tactical-client/src/animation_viewer_main.rs'];assert b'scenario: Option<String>' in cli and b'scenario: args.scenario' in cli
mod=s['crates/adventuresim-tactical-client/src/animation_viewer/mod.rs'];assert b'scenario.map(ScenarioMetadata::from_label)' in mod and b'CaptureSequence::new' in mod and b'AppExit::Error' in mod
capture=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs'];assert b'.is_none_or(|scenario| &frame.scenario == scenario)' in capture and b'requested animation capture scenario is unknown' in capture
assert capture.index(b'capture_plan()?')<capture.index(b'!plan.is_empty()')
terrain=s['crates/adventuresim-tactical-client/src/animation_viewer/scenarios/terrain_policy.rs'];assert b'ScenarioTerrain::RaisedStop { .. }' in terrain and b'ScenarioTerrain::Absent => TerrainIkMode::Disabled' in terrain
lock=s['Cargo.lock'];assert b'af23d6f6c1a224baef9d3f61e287d2761385a5b88fdab4eb4c6f11aeb54c4bcf' in lock and b'7695ce3845ea4b33927c055a39dc438a45b059f7c1b3d91d38d10355fb8cbca7' in lock
with tarfile.open(r/'native-evidence/strum_macros-0.27.2/strum_macros-0.27.2.crate','r:gz') as ar:
 f=ar.extractfile('strum_macros-0.27.2/src/macros/strings/from_string.rs').read();n=ar.extractfile('strum_macros-0.27.2/src/helpers/variant_props.rs').read();o=ar.extractfile('strum_macros-0.27.2/src/macros/strings/as_ref_str.rs').read()
 assert b'get_serializations(type_properties.case_style)' in f and b'quote! { #serialization => #name::#ident #params, }' in f
 assert b'pub fn get_preferred_name' in n and b'pub fn get_serializations' in n and b'pub fn as_static_str_inner' in o
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {400:[1401,1402,1403],401:[1404,1405,1406]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
for path in ('adler/READ-L11-202.json','adler/READ-L11-204.json','segovb/READ-L11-535.json'):assert path in records
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries']
assert q[46]['contract_id']==a['existing_finding_connections'][0]['id']=='ADLER-L11-CAPTURE-SCENARIO-ADMISSION-PANIC'
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1196,1201)]==list(range(1401,1406))
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [46,*range(1196,1201)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical reads and qualified scenario admission sources; no product execution.')
