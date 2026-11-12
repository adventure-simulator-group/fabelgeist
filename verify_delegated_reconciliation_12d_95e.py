#!/usr/bin/env python3
"""Bind furniture reconciliation to original scopes and named readers."""
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
name='ASSESSMENT-DELEGATED-FURNITURE-P16-2552-2553.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)=={2552,2553};cp=load('DELEGATED-EVIDENCE-CHECKPOINT-12.json');b=next(x for x in cp['bundles'] if x['bundle_id']=='p16-furniture-room-index');work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
obs=[{'bundle_id':b['bundle_id'],'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])['findings_and_questions']]
with tarfile.open(R/b['archive']) as t:
 receipt=json.load(t.extractfile('read-receipt.json'))
 for i in (2552,2553):
  row=rows[i];rr=reg[i];cc=row['current_comparison'];a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');old=t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read();hr=a['actual_display_ranges'];cr=z['actual_display_ranges']
  assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and row['new_personal_read_attribution']==b['agent']
  assert row['historical_member']==a['archive'] and row['current_member']==z['archive'] and not a['unread_ranges'] and not z['unread_ranges']
  assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
  assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2552]['new_personal_display_ranges_outside_original_material']==[[13,345]] and rows[2553]['new_personal_display_ranges_outside_original_material']==[]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-12D-95E.json');assert d['delegated_observations']==obs and len(obs)==len(d['new_qualified_ledger_rows'])==5
ev=d['central_current_display_evidence'];raw=(R/ev['registration']).read_bytes();p=json.loads(raw);r=load(ev['receipt']);assert sha(raw)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==19
for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
with tarfile.open(R/'delegated/central-12d-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
def line(p,n):return src[p][n-1]
pre='crates/adventuresim-building-generator/src/'
assert 'pub struct RoomIndex(u16)' in line(pre+'model/room_index.rs',20) and 'serialized_ordinal()' in line(pre+'generator/room_allocation.rs',166) and 'inside_room: u16' in line(pre+'model/layout.rs',14)
assert '.find(|r|' in line(pre+'interior/footprints.rs',48) and 'room_nodes.push' in line(pre+'interior/navigation.rs',60)
assert "caller's local origin" in line(pre+'furniture/builder.rs',26) and 'let origin = Vec3::new(centre.x, 0.0, centre.z)' in line(pre+'furniture/builder.rs',261) and '.supports' in line(pre+'furniture/builder.rs',295)
assert 'recipe.bounds.min()' in line('crates/adventuresim-tactical-core/src/scene_input/furniture/candidates.rs',122) and 'recipe.clearances' in line('crates/adventuresim-tactical-core/src/scene_input/furniture/candidates.rs',129) and '.colliders' in line('crates/adventuresim-tactical-core/src/scene_input/furniture/collision.rs',12)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-12D-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
assert 'Seven of eleven checkpoint12 bundles' in d['open'] and '3713' in d['open']
print('Verified two attributed P16 roles, nineteen bounded current displays and five qualified relationships; no product execution or lens closure.')
