#!/usr/bin/env python3
"""Check source and recovery lineage only; never run product code."""
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
a=load('ASSESSMENT-JITTER-RIG-CONNECTIONS-95E.json')
h=load('ASSESSMENT-L01-HISTORICAL-1319-1327.json')
w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1319,1328))
assert [x['lines'] for x in h['historical_source_roles']]==[78,92,117,312,141,124,608,390,175]
historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1319-1327.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert cur==raw and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines']
  historical[x['original_index']]=raw
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
jit=historical[1325]
assert b'diagnostics_complete: frames.iter().all(|frame| frame.bones.len() >= 3)' in jit
assert b'if frames.len() < 4' in jit and b'.collect::<Option<Vec<_>>>()' in jit
assert b'if !value.is_finite() || value <= limit.noise_floor || value <= threshold' in jit
cap=s['crates/adventuresim-tactical-client/src/animation_viewer/capture.rs']
assert b'.filter(|frame| !frame.scenario.is(ScenarioId::FullRagdoll))' in cap
assert b'previous_frame.contact_sequence != frame.contact_sequence' in cap
validation=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs']
assert b'jitter::validate(&jitter_frames(&frames)' in validation
assert b'validation.finite_transforms' in validation and b'validation.all_scenarios_complete' in validation and b'validation.all_artifacts_written' in validation
assert b'validation.jitter_validation.diagnostics_complete' in validation
report=s['crates/adventuresim-tactical-client/src/animation_viewer/report/mod.rs']
assert b'frames.len() == plan.len()' in report and b'frame.bones.contains_key(*role)' in report
model=s['crates/adventuresim-tactical-client/src/animation_viewer/report/model.rs']
assert b'bone.position.into_iter().all(f32::is_finite)' in model and b'bone.rotation_xyzw.into_iter().all(f32::is_finite)' in model
rig=historical[1326]
assert b'find_named_descendant(rig_root, RigJointName::SKELETON.as_ref(), &children, &names)' in rig
assert b'target: &str' in rig and b'name.as_str() == target' in rig and b'AuthoredRigJoint::new(name.as_str().into())' in rig
lib=s['crates/fabelgeist-rig/src/lib.rs'];names=s['crates/fabelgeist-rig/src/names.rs']
assert b'pub struct RigJointName' in lib and b'Empty and unfamiliar labels are retained' in lib
assert b'pub const SKELETON: Self' in names
identity=s['crates/adventuresim-tactical-client/src/animation/rig_identity.rs']
assert b'name: RigJointName' in identity and b'pub(crate) fn new(name: RigJointName)' in identity
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob'])
 records[x['original_path']]=json.loads(raw)
expected={360:[1317,1318,1319,1320],361:[1321,1322],362:[1323,1324],363:[1325],364:[1326,1327]}
for batch,ids in expected.items():
 receipt=records[f'segovb/READ-L12-{batch}.json'];assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
assert [x['id'] for x in a['findings']]==['ADLER-L11-JITTER-MEASUREMENT-COVERAGE','SEGOVB-L12-RIG-JOINT-DESCENDANT-PRIVATE-API']
assert a['new_unique_candidate_count']==0
q=load('RECONCILIATION-059.json')['entries']
assert q[35]['contract_id']==a['findings'][0]['id'] and q[254]['contract_id']==a['findings'][1]['id']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [35,254,*range(1113,1122)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified nine historical reads and two existing-finding connections; no product execution.')
