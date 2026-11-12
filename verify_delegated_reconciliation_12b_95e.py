#!/usr/bin/env python3
"""Bind this bounded reconciliation to exact source identities and named readers."""
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
name='ASSESSMENT-DELEGATED-ROOFS-2546-2549.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)==set(range(2546,2550));cp=load('DELEGATED-EVIDENCE-CHECKPOINT-12.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('prworking-split-weather',[2546,2547],'findings_and_questions'),('shed-enclosure-hair',[2548,2549],'findings')]:
 b=bs[bundle];obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=(R/row['historical_member']).read_bytes() if i in (2548,2549) else t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read()
   assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and cc['personal_read_attribution']==b['agent']
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
   if i in (2546,2547):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='pr-working');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges'] and row['new_personal_read_attribution']==b['agent']
   else:
    a=next(x for x in receipt['roles'] if x['original_index']==i);other=row['separate_delegate_6533_comparison'];hr=[[1,len(old.splitlines())]];cr=a['current_display_windows'];assert not a['recovered_archive_personal_display_windows'] and not a['historical_unread_complement'] and not a['current_unread_complement'] and other['delegate']==b['agent'] and other['delegate_6533_display_ranges']==a['historical_display_windows'] and sha(t.extractfile(other['delegate_6533_source_member']).read())==other['delegate_6533_sha256']==sha(old) and row['new_personal_read_attribution']=='/root'
    hraw=(R/other['historical_registration']).read_bytes();hreg=json.loads(hraw);hrec=load(other['historical_receipt']);hrow=next(x for x in hreg['rows'] if x['original_role']==i);hread=next(x for x in hrec['reads'] if x['original_role']==i);assert sha(hraw)==hrec['registration_sha256'] and hrec['reader']=='/root' and hread['sha256']==sha(old) and hread['actual_display_ranges']==hrow['planned_display_ranges']==hr and hrow['source_id']==row['source_id'] and not hread['unread_ranges']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2546]['new_personal_display_ranges_outside_original_material']==rows[2547]['new_personal_display_ranges_outside_original_material']==[]
assert rows[2548]['new_personal_display_ranges_outside_original_material']==[[39,163]] and rows[2549]['new_personal_display_ranges_outside_original_material']==[[23,63]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-12B-95E.json');assert d['delegated_observations']==obs and len(obs)==9 and len(d['new_qualified_ledger_rows'])==8
src={}
for key,archive,count in [('central_current_display_evidence','delegated/central-12b-current-windows-proof.tar.gz',11),('central_supplement_display_evidence','delegated/central-12b-supplement-windows-proof.tar.gz',4)]:
 ev=d[key];raw=(R/ev['registration']).read_bytes();p=json.loads(raw);r=load(ev['receipt']);assert sha(raw)==r['registration_sha256'] and len(p['rows'])==len(r['reads'])==count
 for x,y in zip(p['rows'],r['reads']):assert all(x[k]==y[k] for k in ('path','revision','git_blob','sha256')) and x['planned_display_ranges']==y['actual_display_ranges']
 with tarfile.open(R/archive) as t:
  proof=json.load(t.extractfile('proof.json'))
  for x in proof['paths']:src[x['path']]=t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines()
pre='crates/adventuresim-building-generator/src/'
def line(p,n):return src[pre+p][n-1]
assert '0x6800' in line('generator/roof_split_drainage.rs',68) and 'link.split_eave_edges = ids.to_vec()' in line('generator/roof_cutting.rs',76)
assert 'SEAM_SEARCH_STEP_METRES: f32 = 0.01' in line('generator/shed_dormers.rs',6) and 'SEAM_SEARCH_STEPS: usize = 800' in line('generator/shed_dormers.rs',7) and 'child.faces[0]' in line('generator/shed_dormers.rs',35)
assert 'self.polygon[1]' in line('roof_enclosure.rs',24) and 'self.polygon[2]' in line('roof_enclosure.rs',25)
assert 'enclosure::audit(plan, issues)' in line('audit/core.rs',157) and 'audit_roof_assemblies' in line('audit/core.rs',162) and 'gable_enclosure::audit' in line('audit/enclosure.rs',87) and 'gable_openings::valid' in line('audit/gable_enclosure.rs',221) and 'face.normal()' in line('audit/gable_openings.rs',29)
assert 'face.polygon.len() < 3' in line('audit/gable_enclosure.rs',137) and 'continue' in line('audit/gable_enclosure.rs',138)
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-12B-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
for ident in d['further_evidence_for_existing_rows']:assert 'ASSESSMENT-DELEGATED-RECONCILIATION-12B-95E.json' in ledger[ident]['further_current_evidence']
assert 'Five of eleven checkpoint12 bundles' in d['open'] and '3713' in d['open']
print('Verified four attributed roof roles, two separately credited recovered objects, fifteen bounded current displays and eight qualified relationships; no product execution or lens closure.')
