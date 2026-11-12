#!/usr/bin/env python3
"""Bind BVH reconciliation to exact source identities and named readers."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cur='95e63628f9d70b8dd6e6fede64334696c6995009'
def norm(rs):
 out=[]
 for a,b in sorted(rs):
  assert 1<=a<=b
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
def complement(rs,n):
 used={i for a,b in rs for i in range(a,b+1)};return norm([[i,i] for i in range(1,n+1) if i not in used])
name='ASSESSMENT-DELEGATED-BVH-MESH-2550-2551.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)=={2550,2551};cp=load('DELEGATED-EVIDENCE-CHECKPOINT-12.json');b=next(x for x in cp['bundles'] if x['bundle_id']=='pr-bvh-physics-mesh');work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
obs=[{'bundle_id':b['bundle_id'],'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])['findings']]
with tarfile.open(R/b['archive']) as t:
 receipt=json.load(t.extractfile('personal-read-receipt.json'))
 for i in (2550,2551):
  row=rows[i];rr=reg[i];cc=row['current_comparison'];a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity'].startswith('pr-working'));z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cur);old=t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read();hr=a['display_chunks'];cr=z['display_chunks']
  assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and cc['personal_read_attribution']==row['new_personal_read_attribution']==b['agent']
  assert row['historical_member']==a['source_archive'] and row['current_member']==z['source_archive'] and not a['not_contiguously_displayed_ranges'] and not z['not_contiguously_displayed_ranges']
  assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
  assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2550]['new_personal_display_ranges_outside_original_material']==[[1,26],[43,61],[139,174],[300,376]] and rows[2551]['new_personal_display_ranges_outside_original_material']==[]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-12C-95E.json');assert d['delegated_observations']==obs and len(obs)==len(d['new_qualified_ledger_rows'])==4
ev=d['central_current_display_evidence'];raw=(R/ev['registration']).read_bytes();p=json.loads(raw);r=load(ev['receipt']);assert sha(raw)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==9
for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
with tarfile.open(R/'delegated/central-12c-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
def line(p,n):return src[p][n-1]
bvh='crates/fabelgeist-bvh/src/gpu/';mesh='crates/fabelgeist-physics/src/'
assert 'self.count = count' in line(bvh+'mod.rs',209) and 'count == 0' in line(bvh+'mod.rs',210) and 'return Ok(())' in line(bvh+'mod.rs',211)
assert 'stack[0] = 0u' in line(bvh+'shaders.rs',454) and '{callback}' in line(bvh+'shaders.rs',473)
assert 'pub thickness: f32' in line(mesh+'mesh.rs',23) and 'self.surface.thickness' in line(mesh+'mesh.rs',143) and '- margin' in line(mesh+'wgsl.rs',410) and '+ margin' in line(mesh+'wgsl.rs',411)
assert 'positions.len() as u32' in line(mesh+'mesh.rs',102) and 'positions.len() as u32' in line(mesh+'mesh.rs',121)
assert 'data.write_to' in line('crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs',128) and 'queue.write_buffer' in line('crates/fabelgeist-gpu/src/data/gpu/buffer/upload.rs',56)
assert 'self.record_build' in line(bvh+'mod.rs',311) and 'batch.submit' in line(bvh+'mod.rs',312)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-12C-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
assert 'Six of eleven checkpoint12 bundles' in d['open'] and '3713' in d['open']
print('Verified two attributed BVH roles, nine bounded current displays and four qualified relationships; no shader or product execution, or lens closure.')
