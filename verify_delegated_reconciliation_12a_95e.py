#!/usr/bin/env python3
"""Bind roof reconciliation to exact source identities and named readers."""
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
name='ASSESSMENT-DELEGATED-ROOFS-2540-2545.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)==set(range(2540,2546));cp=load('DELEGATED-EVIDENCE-CHECKPOINT-12.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('prworking-edge-material',[2540,2541],'findings_and_questions'),('roof-finalization-internal-hair',[2542,2543],'findings'),('pr-roof-outlets-sampling',[2544,2545],'findings')]:
 b=bs[bundle];obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=(R/row['historical_member']).read_bytes() if i in (2542,2543) else t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read()
   assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and cc['personal_read_attribution']==b['agent']
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
   if i in (2540,2541):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='pr-working');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges'] and row['new_personal_read_attribution']==b['agent']
   elif i in (2542,2543):
    a=next(x for x in receipt['roles'] if x['original_index']==i);other=row['separate_delegate_6533_comparison'];hr=[[1,len(old.splitlines())]];cr=a['current_display_windows'];assert not a['recovered_archive_personal_display_windows'] and not a['historical_unread_complement'] and not a['current_unread_complement'] and other['delegate']==b['agent'] and other['delegate_6533_display_ranges']==a['historical_display_windows'] and sha(t.extractfile(other['delegate_6533_source_member']).read())==other['delegate_6533_sha256']==sha(old) and row['new_personal_read_attribution']=='/root'
    hraw=(R/other['historical_registration']).read_bytes();hreg=json.loads(hraw);hrec=load(other['historical_receipt']);hrow=next(x for x in hreg['rows'] if x['original_role']==i);hread=next(x for x in hrec['reads'] if x['original_role']==i);assert sha(hraw)==hrec['registration_sha256'] and hrec['reader']=='/root' and hread['sha256']==sha(old) and hread['actual_display_ranges']==hrow['planned_display_ranges']==hr and hrow['source_id']==row['source_id'] and not hread['unread_ranges']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity'].startswith('pr-working'));z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cur);hr=a['display_chunks'];cr=z['display_chunks'];assert not a['not_contiguously_displayed_ranges'] and not z['not_contiguously_displayed_ranges'] and row['new_personal_read_attribution']==b['agent']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert all(not row['new_personal_display_ranges_outside_original_material'] for row in rows.values())
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-12A-95E.json');assert d['delegated_observations']==obs and len(obs)==15 and len(d['new_qualified_ledger_rows'])==14
ev=d['central_current_display_evidence'];registration=(R/ev['registration']).read_bytes();p=json.loads(registration);r=load(ev['receipt']);assert sha(registration)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==10
for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
with tarfile.open(R/'delegated/central-12a-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
pre='crates/adventuresim-building-generator/src/'
def line(p,n):return src[pre+p][n-1]
assert 'edge.adjacent_faces.push' in line('generator/roof_edge_binding.rs',88) and 'RoofEdgeKind::WallAbutment' in line('generator/roof_edge_binding.rs',160)
assert 'adjacency != expected' in line('audit/roofs.rs',1150) and '.solids' in line('generator/roof_edge_binding.rs',138) and 'weather_ids.contains' in line('generator/roof_edge_binding.rs',139)
assert 'clearance).max(0.0)' in line('generator/roof_internal_cut.rs',26) and 'for support in &second.supported_by' in line('generator/roof_internal_cut.rs',103)
assert 'initial_solids + added.len()' in line('generator/roof_internal_cut.rs',101) and 'second.id.0' in line('generator/roof_internal_cut.rs',105)
assert 'yaw_bounds(a)' in line('generator/roof_finalization.rs',100) and 'roof_bonds.len()' in line('generator/roof_finalization.rs',107)
assert 'recipient_y + 0.08 < source_low' in line('generator/roof_outlets.rs',224) and 'recipient_y + 0.06' in line('generator/roof_outlets.rs',492) and 'centre.y > fall_bottom + 0.08' in line('audit/roofs.rs',369)
assert 'plan_point_in_convex_polygon' in line('generator/roof_plan_sampling.rs',52) and 'sign * (end - *start).perp_dot' in line('audit/roofs.rs',812)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-12A-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
assert 'ASSESSMENT-DELEGATED-RECONCILIATION-12A-95E.json' in ledger['ROOF-JUNCTION-GUTTER-COLLISION-CLEANUP-RELATION']['further_current_evidence']
assert 'Three of eleven checkpoint12 bundles' in d['open'] and '3713' in d['open']
print('Verified six attributed roof roles, two separately credited recovered objects, ten bounded current displays and fourteen qualified relationships; no product execution or lens closure.')
