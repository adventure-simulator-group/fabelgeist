#!/usr/bin/env python3
"""Verify inspection/report source and recovery evidence, without product execution."""
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
a=load('ASSESSMENT-INSPECTION-REPORT-BOUNDARIES-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1427-1432.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1427,1433))
assert [x['lines'] for x in h['historical_source_roles']]==[67,179,220,133,138,99]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1427-1432.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
assert h['historical_source_roles'][0]['prior_role_status']==h['historical_source_roles'][1]['prior_role_status']=='bounded-current-trace-only'
assert all(x['prior_current_observations'] for x in h['historical_source_roles'][:2])
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
inspect=current[1427];assert b'item.id == "cuirass"' in inspect and b'material.base.metallic == 0.0 && material.base.base_color.alpha() == alpha' in inspect and b'material.base.perceptual_roughness = 1.0' in inspect
assert b'Self::Puffed => &["puffed_sleeve", "puffed_hose"]' in s['crates/adventuresim-tactical-client/src/animation_viewer/harness.rs']
recipe=json.loads(s['assets_src/equipment/armor-designs.json']);sleeve=recipe['puffed_sleeve']['PuffAndSlash']['pattern']
assert (sleeve['outer_fabric']['finish'],sleeve['outer_fabric']['opacity'])==('Wool',1000)
assert (sleeve['undercloth_fabric']['finish'],sleeve['undercloth_fabric']['opacity'])==('Silk',1000)
fabric=s['crates/fabelgeist-armor/src/textile_pattern.rs'];assert b'TextileFinish::Wool => 0.92' in fabric and b'TextileFinish::Silk => 0.46' in fabric and b'material.metallic = if self.finish == TextileFinish::MetallicThread' in fabric
assign=s['crates/adventuresim-character-creator/src/device_textile_materials.rs'];assert b'ArmorComponentRole::Undercloth => Some(&puff.pattern.undercloth_fabric)' in assign and b'ArmorComponentRole::OuterFabric => Some(&puff.pattern.outer_fabric)' in assign
surf=s['crates/adventuresim-character-creator/src/textile_surface.rs'];assert b'metallic: surface.metallic' in surf and b'perceptual_roughness: surface.roughness' in surf
assert b'let mut base = source.clone();' in s['crates/adventuresim-tactical-client/src/presentation/interior_lighting/material.rs']
assert b'if minimum.is_finite() && maximum.is_finite()' in current[1430] and b'        0.0\n    }\n}' in current[1430]
assert b'.sort_by_key(|(role, _)| report_bone_label(**role))' in current[1431] and b'uncaptured semantic bone in report' in current[1431]
report=s['crates/adventuresim-tactical-client/src/animation_viewer/report/mod.rs'];validation=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs'];model=s['crates/adventuresim-tactical-client/src/animation_viewer/report/model.rs']
assert b'REPORT_BONES\n' in report and b'.all(|(role, _)| frame.bones.contains_key(*role))' in report
assert b'frames.iter().all(FrameSample::has_finite_bone_transforms)' in validation and b'bone.position.into_iter().all(f32::is_finite)' in model
assert b'<button data-scenario=' in current[1432] and b'let frame_json = serde_json::to_string(&manifest.frames)?' in current[1432]
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {411:[1426,1427,1428],412:[1429,1430,1431],413:[1432]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
eof=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1428-EOF-001.json']);assert eof['source_sha256']==sha(current[1428]) and eof['actual_EOF']==179
assert eof['declared']==[[145,180]] and eof['actual_existing_body']==[[145,179]] and eof['nonexistent_tail']==[[180,180]] and eof['new_population_credit']==0
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=list(range(1224,1231))
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1427,1428,1428,1429,1430,1431,1432]
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in indices)
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id']=='REVIEW-CAPTURE-DIFFUSE-ROUGHNESS-SHORTCUT-001' for x in ledger)==1
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified six historical inspection/report bodies, exact EOF qualification and conditional material source trace; no product execution.')
