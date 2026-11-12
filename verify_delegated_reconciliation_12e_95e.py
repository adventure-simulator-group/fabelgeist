#!/usr/bin/env python3
"""Bind storey and plan reconciliation to original scopes and named readers."""
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
name='ASSESSMENT-DELEGATED-STOREY-PLAN-P16-2554-2555.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)=={2554,2555};cp=load('DELEGATED-EVIDENCE-CHECKPOINT-12.json');b=next(x for x in cp['bundles'] if x['bundle_id']=='p16-storey-plan-hair');work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
obs=[{'bundle_id':b['bundle_id'],'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])['findings']]
with tarfile.open(R/b['archive']) as t:
 receipt=json.load(t.extractfile('read-receipt.json'))
 for i in (2554,2555):
  row=rows[i];rr=reg[i];cc=row['current_comparison'];a=next(x for x in receipt['roles'] if x['original_index']==i);old=t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read();hr=a['p16_display_windows'];cr=a['current_display_windows']
  assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and row['new_personal_read_attribution']==b['agent']
  assert not a['p16_unread_complement'] and not a['current_unread_complement'] and not a['0393_a120_6533_personal_display_windows']
  assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
  assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2554]['new_personal_display_ranges_outside_original_material']==[[1,78],[91,137]] and rows[2555]['new_personal_display_ranges_outside_original_material']==[[50,341]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-12E-95E.json');assert d['delegated_observations']==obs and len(obs)==3 and len(d['new_qualified_ledger_rows'])==2
ev=d['central_current_display_evidence'];raw=(R/ev['registration']).read_bytes();p=json.loads(raw);r=load(ev['receipt']);assert sha(raw)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==9
for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
with tarfile.open(R/'delegated/central-12e-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
def line(p,n):return src[p][n-1]
pre='crates/adventuresim-building-generator/src/'
assert 'signed_area(&vertices) == 0.0' in line(pre+'plan_geometry.rs',127) and 'cross(hull' in line(pre+'plan_geometry.rs',206) and 'as_dvec2() - a.plan_metres().as_dvec2()' in line(pre+'plan_geometry.rs',227)
assert 'PlanHull::from_points' in line(pre+'collision/footprint.rs',23) and 'PlanHull::Area(polygon)' in line(pre+'collision/footprint.rs',24) and 'PlanHull::Segment' in line(pre+'collision/footprint.rs',25)
assert 'pub struct StoreyIndex(usize)' in line(pre+'model/storey_index.rs',10) and 'u16::try_from(self.0)' in line(pre+'model/storey_index.rs',21) and 'pub struct RoomIndex(u16)' in line(pre+'model/room_index.rs',20)
assert 'inside_room: Option<u16>' in line(pre+'model/walls.rs',124)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-12E-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
for ident in d['further_evidence_for_existing_rows']:assert 'ASSESSMENT-DELEGATED-RECONCILIATION-12E-95E.json' in ledger[ident]['further_current_evidence']
assert 'Eight of eleven checkpoint12 bundles' in d['open'] and '3713' in d['open']
print('Verified two attributed P16 roles, nine bounded current displays and two qualified relationships; no product execution or lens closure.')
