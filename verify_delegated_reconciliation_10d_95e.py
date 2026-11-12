#!/usr/bin/env python3
"""Bind animation/roof reconciliation to exact attributed source receipts."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def norm(rs):
 out=[]
 for a,b in sorted(rs):
  assert 1<=a<=b
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out
assessment=load('ASSESSMENT-DELEGATED-ANIMATION-ROOF-2488-2495.json');rows={x['original_index']:x for x in assessment['historical_source_roles']};assert set(rows)==set(range(2488,2496))
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for name,ids,key in [('p16-animation',[2488,2489],'findings_and_questions'),('p16-ik-hair',[2490],'connections'),('p16-roof-geometry',[2491,2492,2493],'findings'),('roof-assemblies-cutting',[2494,2495],'findings')]:
 b=bs[name];report=load(b['report_path']);obs.extend({'bundle_id':name,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in report[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=t.extractfile(row['historical_member']).read();cur=t.extractfile(row['current_member']).read()
   assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(cur) and cc['byte_identical_to_historical']==(old==cur)
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_registry_status']==rr.get('status') and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   if i in (2488,2489):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges']
   elif i==2490:
    hr=receipt['p16_actual_display_ranges'];cr=receipt['current_actual_display_ranges'];assert row['historical_unread_ranges']==receipt['p16_unread_ranges']==[[371,2934]] and cc['unread_ranges']==receipt['current_unread_ranges']==[[371,1539],[1621,2479],[2551,2934]]
    assert work[i]['status']=='source-metadata-partial' and work[i]['assessment']=='ASSESSMENT-DELEGATED-ANIMATION-ROOF-2488-2495.json' and not receipt['base_actual_display_ranges'] and not receipt['pr_working_actual_display_ranges'] and not receipt['older_origin_actual_display_ranges']
   elif i in (2491,2492,2493):
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_revision']!=cc['source_revision']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_revision']==cc['source_revision']);hr=a['contiguously_displayed_body_ranges'];cr=z['contiguously_displayed_body_ranges'];assert a['original_material_scope_personally_displayed']==rr['material_ranges'] and row['new_connected_support_ranges_outside_original_role']==a['historical_excluded_personal_support_ranges']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['revision']==row['snapshot_head']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['revision']==cc['source_revision']);hr=a['contiguously_displayed_ranges'];cr=z['contiguously_displayed_ranges']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr) and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)
   if i!=2490:assert row['new_personal_read_ranges']==[[1,len(old.splitlines())]] and cc['personal_ranges']==[[1,len(cur.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']=='ASSESSMENT-DELEGATED-ANIMATION-ROOF-2488-2495.json'
   if i in (2488,2489,2490):assert rr['status']=='navigation_only_no_body_analysis' and rr['source_or_other_lens_credit']=='none from metadata registration'
assert rows[2491]['new_connected_support_ranges_outside_original_role']==[[81,123]] and rows[2492]['new_connected_support_ranges_outside_original_role']==[[35,55]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-10D-95E.json');assert d['delegated_observations']==obs and len(obs)==14 and len(d['new_qualified_ledger_rows'])==13
ledger={e['id']:e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 e=ledger[ident];assert e['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10D-95E.json' and e['repair']=='not started' and e['validation']=='not performed' and 'open' in e['discovery']
assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10D-95E.json' in ledger['P16-MUSCLE-ENVELOPE-AND-PHASE-LIMITS']['further_current_evidence']
assert 'tracked patch' in assessment['qualification'] and '11 checkpoint10 bundles' in d['open'] and '3713' in d['open']
print('Verified eight attributed animation/roof roles, explicit IK unread ranges, original scope exclusions and 13 qualified connected ledger relationships; no lens or product validation claimed.')
