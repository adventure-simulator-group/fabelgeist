#!/usr/bin/env python3
"""Bind window reconciliation to personal receipts and distinct revisions."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cur='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e'
def norm(rs):
 out=[]
 for a,b in sorted(rs):
  assert 1<=a<=b
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
def complement(rs,n):
 used={i for a,b in rs for i in range(a,b+1)};return norm([[i,i] for i in range(1,n+1) if i not in used])
name='ASSESSMENT-DELEGATED-WINDOWS-2526-2533.json';rows={x['original_index']:x for x in load(name)['historical_source_roles']};assert set(rows)=={2526,2527,2528,2529,2530,2533};cp=load('DELEGATED-EVIDENCE-CHECKPOINT-11.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('p16-scene-window-spatial',[2526,2527],'findings_and_questions'),('p16-identity-server-door-hair',[2528,2529],'observations'),('p16-window-apse',[2530,2533],'findings_and_questions')]:
 b=bs[bundle];obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in load(b['report_path'])[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=(R/row['historical_member']).read_bytes() if i==2533 else t.extractfile(row['historical_member']).read();now=t.extractfile(row['current_member']).read();assert row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(now) and cc['byte_identical_to_historical']==(old==now) and cc['personal_read_attribution']==b['agent']
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit') and row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
   if i in (2526,2527):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges'] and row['new_personal_read_attribution']==b['agent']
   elif i in (2528,2529):
    a=next(x for x in receipt['roles'] if x['original_index']==i);hr=a['p16_display_windows'];cr=a['current_display_windows'];assert not a['p16_unread_complement'] and not a['current_unread_complement'] and not a['0393_a120_6533_personal_display_windows'] and row['new_personal_read_attribution']==b['agent']
   elif i==2530:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']!=cur);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cur);hr=a['display_chunks'];cr=z['display_chunks'];assert a['original_material_ranges']==rr['material_ranges'] and a['separately_accounted_support_ranges']==row['new_personal_display_ranges_outside_original_material'] and row['new_personal_read_attribution']==b['agent']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==pr);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cur);hr=[[1,75]];cr=z['display_chunks'];other=row['separate_delegate_6533_comparison'];assert other['delegate']==b['agent'] and other['delegate_6533_display_ranges']==a['display_chunks'] and sha(t.extractfile(a['source_archive']).read())==other['delegate_6533_sha256']==sha(old) and row['new_personal_read_attribution']=='/root'
    raw=(R/other['historical_registration']).read_bytes();hreg=json.loads(raw);hrec=load(other['historical_receipt']);assert sha(raw)==hrec['registration_sha256'] and hrec['reader']=='/root' and hrec['sha256']==sha(old) and hrec['actual_display_ranges']==hreg['planned_display_ranges']==hr and hreg['source_id']==row['source_id'] and not hrec['unread_ranges']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(now.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2530]['new_personal_display_ranges_outside_original_material']==[[151,294]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-11B-95E.json');assert d['delegated_observations']==obs and len(obs)==15 and len(d['new_qualified_ledger_rows'])==6
registration=(R/d['central_registration']).read_bytes();receipt=load(d['central_receipt']);assert sha(registration)==receipt['registration_sha256'];plan=json.loads(registration);assert len(plan['rows'])==len(receipt['reads'])==11
for p,r in zip(plan['rows'],receipt['reads']):assert all(p[k]==r[k] for k in ('path','revision','git_blob','sha256')) and p['planned_display_ranges']==r['actual_display_ranges']
with tarfile.open(R/'delegated/central-11b-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']};server=src['crates/adventuresim-tactical-server/src/windows.rs'];doors=src['crates/adventuresim-tactical-core/src/doors.rs'];equip=src['crates/adventuresim-tactical-server/src/equipment.rs'];rule=src['crates/AGENTS.md'];setup=src['crates/adventuresim-tactical-server/src/scene_setup.rs']
 assert 'Never use `unwrap` or `expect` outside tests.' in rule[125] and '.expect(' in setup[20]
 assert 'can_grab_window_from_inside(' in server[43] and 'controller.open = !controller.open' in server[52] and 'offset.dot(outward)' in doors[23] and 'offset.dot(tangent)' in doors[24]
 assert 'EquipmentAction::ToggleWindow' in equip[236] and 'try_toggle_from_inside' in equip[240]
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 x=ledger[ident];assert x['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-11B-95E.json' and x['repair']=='not started' and x['validation']=='not performed' and 'open' in x['discovery']
for ident in ('P16-WINDOWSPEC-AGGREGATE','P16-DOORSPEC-CROSS-FIELD-POLICY','WINDOW-CONTROLLER-CHECKED-HINGE-ANGLE-PROPAGATION','REVIEW-GATE-RIGHT-HINGE-INTERACTION-OFFSET-001'):assert 'ASSESSMENT-DELEGATED-RECONCILIATION-11B-95E.json' in ledger[ident]['further_current_evidence']
assert 'Six of nine checkpoint11' in d['open'] and '3713' in d['open']
print('Verified six attributed source roles, the separate pr-working and 6533 reads, and six qualified window/door relationships; no product execution or lens closure.')
