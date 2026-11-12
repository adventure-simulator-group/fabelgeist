#!/usr/bin/env python3
"""Bind closure reconciliation to attributed source displays and scopes."""
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
name='ASSESSMENT-DELEGATED-CLOSURES-P16-2513-2518.json';assessment=load(name);rows={x['original_index']:x for x in assessment['historical_source_roles']};assert set(rows)==set(range(2513,2519))
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('p16-closure-door-presentation',[2513,2514],'findings_and_questions'),('p16-shutters-mesh',[2515,2516],'findings_and_questions'),('p16-window-strategic-instances-hair',[2517,2518],'connected_observations')]:
 b=bs[bundle];report=load(b['report_path']);obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in report[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=t.extractfile(row['historical_member']).read();cur=t.extractfile(row['current_member']).read()
   assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(cur) and cc['byte_identical_to_historical']==(old==cur)
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   if i in (2513,2514):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges']
   elif i in (2515,2516):
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']!=cc['source_revision']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cc['source_revision']);hr=a['display_chunks'];cr=z['display_chunks'];assert a['p16_original_material_ranges']==rr['material_ranges'] and row['new_connected_support_ranges_outside_original_role']==a['separately_accounted_support_ranges']
   else:
    a=next(x for x in receipt['roles'] if x['original_index']==i);hr=a['p16_display_windows'];cr=a['current_display_windows'];assert not a['p16_unread_complement'] and not a['current_unread_complement'] and not a['0393_a120_6533_personal_display_windows']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(cur.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2515]['new_connected_support_ranges_outside_original_role']==[[80,115]] and rows[2516]['new_connected_support_ranges_outside_original_role']==[[1,39]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-10G-95E.json');assert d['delegated_observations']==obs and len(obs)==13 and len(d['new_qualified_ledger_rows'])==7
registration=(R/d['central_registration']).read_bytes();receipt=load(d['central_receipt']);assert sha(registration)==receipt['registration_sha256'];plan=json.loads(registration);assert len(plan['rows'])==len(receipt['reads'])==7
for p,r in zip(plan['rows'],receipt['reads']):assert p['path']==r['path'] and p['revision']==r['revision'] and p['git_blob']==r['git_blob'] and p['sha256']==r['sha256'] and p['planned_display_ranges']==r['actual_display_ranges']
with tarfile.open(R/'delegated/central-10g-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));source={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
 assert 'Never use `unwrap` or `expect` outside tests.' in source['crates/AGENTS.md'][125]
 code=source['crates/adventuresim-tactical-client/src/presentation/recipe_mesh.rs'];assert '.expect(' in code[35] and '.expect(' in code[83]
ledger={e['id']:e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 e=ledger[ident];assert e['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10G-95E.json' and e['repair']=='not started' and e['validation']=='not performed' and 'open' in e['discovery']
for ident in ('P16-WINDOW-LEAF-UNBOUNDED-WORK','P16-TINY-SHUTTER-COMPILATION','CURRENT-LOD-MESH-INDEX-NATIVE-BOUNDARY-RELATION','CURRENT-VISUAL-SEED-OWNER-RELATION','CURRENT-FACADE-CLOSURE-PROJECTION-TRACE'):assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10G-95E.json' in ledger[ident]['further_current_evidence']
assert 'All 27 checkpoint10 bundles' in d['open'] and '3713' in d['open'] and 'product failure established' in ledger['P16-CLOSURE-RECIPE-TANGENT-EXPECT-RULE']['relationship_qualification']
print('Verified six closure roles, original exclusions, source-rule mismatch and seven connected ledger relationships; checkpoint10 bounded review is not lens or product validation.')
