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
name='ASSESSMENT-DELEGATED-ROOFS-2534-2539.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)==set(range(2534,2540));cp=load('DELEGATED-EVIDENCE-CHECKPOINT-11.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('prworking-defense-dormer',[2534,2535],'findings_and_questions'),('roof-child-contacts-hair',[2536,2537],'findings'),('pr-roof-dormers-drainage',[2538,2539],'findings')]:
 b=bs[bundle];obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=(R/row['historical_member']).read_bytes() if i in (2536,2537) else t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read()
   assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and cc['personal_read_attribution']==b['agent']
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
   if i in (2534,2535):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='pr-working');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges'] and row['new_personal_read_attribution']==b['agent']
   elif i in (2536,2537):
    a=next(x for x in receipt['roles'] if x['original_index']==i);other=row['separate_delegate_6533_comparison'];hr=[[1,len(old.splitlines())]];cr=a['current_display_windows'];assert not a['recovered_archive_personal_display_windows'] and not a['historical_unread_complement'] and not a['current_unread_complement'] and other['delegate']==b['agent'] and other['delegate_6533_display_ranges']==a['historical_display_windows'] and sha(t.extractfile(other['delegate_6533_source_member']).read())==other['delegate_6533_sha256']==sha(old) and row['new_personal_read_attribution']=='/root'
    hraw=(R/other['historical_registration']).read_bytes();hreg=json.loads(hraw);hrec=load(other['historical_receipt']);hrow=next(x for x in hreg['rows'] if x['original_role']==i);hread=next(x for x in hrec['reads'] if x['original_role']==i);assert sha(hraw)==hrec['registration_sha256'] and hrec['reader']=='/root' and hread['sha256']==sha(old) and hread['actual_display_ranges']==hrow['planned_display_ranges']==hr and hrow['source_id']==row['source_id'] and not hread['unread_ranges']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity'].startswith('pr-working'));z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cur);hr=a['display_chunks'];cr=z['display_chunks'];assert not a['not_contiguously_displayed_ranges'] and not z['not_contiguously_displayed_ranges'] and row['new_personal_read_attribution']==b['agent']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2534]['new_personal_display_ranges_outside_original_material']==[[10,443]] and rows[2536]['new_personal_display_ranges_outside_original_material']==[[43,78]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-11C-95E.json');assert d['delegated_observations']==obs and len(obs)==12 and len(d['new_qualified_ledger_rows'])==9
src={}
for ev,expected in zip(d['central_current_display_evidence'],[15,6,5]):
 registration=(R/ev['registration']).read_bytes();p=json.loads(registration);r=load(ev['receipt']);assert sha(registration)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==expected
 for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
 filename={'CENTRAL-11C-CURRENT-WINDOWS-PROOF.json':'delegated/central-11c-current-windows-proof.tar.gz','CENTRAL-11C-SUPPLEMENT-WINDOWS-PROOF.json':'delegated/central-11c-supplement-windows-proof.tar.gz','CENTRAL-11C-PROXY-WINDOWS-PROOF.json':'delegated/central-11c-proxy-windows-proof.tar.gz'}[ev['git_path_proof']]
 with tarfile.open(R/filename) as t:
  proof=json.load(t.extractfile('proof.json'))
  for x in proof['paths']:src[x['path']]=t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines()
pre='crates/adventuresim-building-generator/src/'
def line(p,n):return src[pre+p][n-1]
assert '=> [a0, a1]' in line('generator/defense_geometry.rs',75) and 'walk_junction_centre(walks[walk_a], walks[walk_b])' in line('generator/defensive_circuits.rs',83)
assert '.expect(' in line('generator/projected_defenses.rs',268) and 'supported_by[0]' in line('generator/defense_geometry.rs',201)
assert 'normal_distance <=' in line('generator/roof_contacts.rs',85) and 'signed_normal - wall.thickness_metres * 0.5' in line('generator/roof_contacts.rs',78)
assert '.map(ResolvedSolid::yaw_bounds)' in line('generator/roof_contacts.rs',33) and 'self.size.metres().y * 0.5' in line('geometry_index.rs',23) and 'self.crossfall_radians' in line('geometry_index.rs',54)
assert 'unwrap_or(fallback_depth)' in line('generator/dormer_layout.rs',67) and '&assemblies[0]' in line('generator/roof_dormers.rs',21) and 'unwrap_or(parent_base)' in line('generator/roof_child_enclosure.rs',26)
assert 'left_edge_index as u64' in line('generator/roof_edge_binding.rs',43) and 'face_index as u64 * 8' in line('generator/roof_drainage.rs',59) and '!old_spouts.contains(&solid.id)' in line('generator/roof_outlets.rs',804)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-11C-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
for ident in ('ROOF-DORMER-PARENT-SOURCE-CONDITION','CURRENT-ROOF-CHECKED-POSITION-RETAINED-RAW-RELATION','CURRENT-ROOF-ABUTMENT-MASKED-IDENTITY-RELATION'):assert 'ASSESSMENT-DELEGATED-RECONCILIATION-11C-95E.json' in ledger[ident]['further_current_evidence']
assert 'All nine checkpoint11 bundles have bounded central reconciliation' in d['open'] and '3713' in d['open']
print('Verified six attributed roof roles, separately credited historical/6533 readers, 26 bounded current displays and nine qualified relationships; no product execution or lens closure.')
