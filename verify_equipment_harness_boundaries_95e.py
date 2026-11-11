#!/usr/bin/env python3
"""Verify bounded harness source and recovery evidence, without product execution."""
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
a=load('ASSESSMENT-EQUIPMENT-HARNESS-BOUNDARIES-95E.json')
h=load('ASSESSMENT-L01-HISTORICAL-1422-1426.json')
w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert a['conditional_findings']==[]
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1422,1427))
assert [x['lines'] for x in h['historical_source_roles']]==[511,29,285,48,66]
current={};originals={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']]
  assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1422-1426.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical']==(x['original_index']!=1422)
  assert sha(cur)==comp['sha256'] and len(cur)==comp['bytes'] and len(cur.splitlines())==comp['lines']
  assert blob(cur)==comp['git_blob']==tree[comp['path']]
  originals[x['original_index']]=raw;current[x['original_index']]=cur
comp=h['historical_source_roles'][0]['current_comparison'];assert comp['lines']==518
expected=''.join(difflib.unified_diff(originals[1422].decode().splitlines(keepends=True),current[1422].decode().splitlines(keepends=True),fromfile='historical-1422/'+comp['path'],tofile='95e/'+comp['path'])).encode()
assert (r/comp['diff']).read_bytes()==expected and sha(expected)==comp['diff_sha256']
assert b'GermanDressUpper' not in originals[1422] and b'GermanDressUpper' in current[1422]
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes())
 assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
readiness=current[1424];harness=current[1422];weight=current[1425];topology=current[1426]
assert b'skin.joints.is_empty() || skin.joints.len() != part.joint_names.len()' in readiness
assert b'Option<&\'static MeshMaterial3d<InteriorMaterial>>' in readiness and b'names: &[]' in harness and b'morph_targets: Some(0)' in harness
assert b'PRIMARY_WEIGHT_SUM_TOLERANCE: f32 = 1e-4' in weight and b'self.vertices > 0 && self.invalid_vertices == 0' in weight
assert b'.find_map(|(definition, entity, parent_placement)|' in topology and b'capacity_index: 0' in topology
skin=s['crates/adventuresim-tactical-client/src/equipment/skin.rs'];assert b'skin.joints != joints' in skin and b'inverse_bindposes: part.inverse_bindposes.clone()' in skin
material=s['crates/adventuresim-tactical-client/src/presentation/interior_lighting/material.rs'];assert b'InteriorMaterialSource(handle.0.clone()),\n                MeshMaterial3d(material.clone())' in material
assert b'.chain()' in s['crates/adventuresim-tactical-client/src/equipment/visuals.rs'] and b'PostUpdate' in s['crates/adventuresim-tactical-client/src/presentation/interior_lighting/mod.rs']
gen=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/generation.rs'];assert b'commands.entity(child).despawn()' in gen and b'commands.entity(entity).with_child(part.render_bundle(' in gen
assert b'self.pending.is_some()' in s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment.rs'] and b'!equipment.is_preparing()' in harness
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {409:[1422,1423],410:[1424,1425],411:[1426,1427,1428]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
eof=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1422-EOF-001.json']);assert eof['source_sha256']==sha(originals[1422]) and eof['actual_EOF']==511
assert eof['declared']==[[430,522]] and eof['actual_existing_body']==[[430,511]] and eof['nonexistent_tail']==[[512,522]] and eof['qualification_body_coverage']==eof['nonexistent_tail_coverage']==0
loc=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1426-LOCATOR.json']);assert loc['exact_source_sha256']==sha(originals[1426]) and loc['actual_EOF']==66 and loc['new_population_credit']==0 and loc['historical_locator_proof_body_not_read'] is True
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1213,1218,1219,1220,1221,1222,1223]
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1426,1422,1423,1422,1424,1425,1426]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical harness bodies, exact EOF/locator qualifications and bounded current connections; no product execution.')
