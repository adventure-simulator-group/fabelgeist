#!/usr/bin/env python3
"""Bind checkpoint11 reconciliation to the named readers and exact scopes."""
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
def complement(rs,n):
 covered={i for a,b in rs for i in range(a,b+1)};return norm([[i,i] for i in range(1,n+1) if i not in covered])
name='ASSESSMENT-DELEGATED-P16-2519-2523.json';assessment=load(name);rows={x['original_index']:x for x in assessment['historical_source_roles']};assert set(rows)==set(range(2519,2524));cp=load('DELEGATED-EVIDENCE-CHECKPOINT-11.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('p16-review-openings-readiness',[2519,2520],'findings_and_questions'),('p16-viewer-building-hair',[2521],'observations'),('p16-compound-boundary',[2522,2523],'findings_and_questions')]:
 b=bs[bundle];report=load(b['report_path']);obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in report[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=t.extractfile(row['historical_member']).read();cur=t.extractfile(row['current_member']).read();assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(cur) and cc['byte_identical_to_historical']==(old==cur)
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   assert row['new_personal_display_ranges_outside_original_material']==complement(rr['material_ranges'],len(old.splitlines()))
   if i in (2519,2520):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges']
   elif i==2521:
    a=next(x for x in receipt['roles'] if x['original_index']==i);hr=a['p16_display_windows'];cr=a['current_display_windows'];assert not a['p16_unread_complement'] and not a['current_unread_complement'] and not a['0393_a120_6533_personal_display_windows']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']!=cc['source_revision']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cc['source_revision']);hr=a['display_chunks'];cr=z['display_chunks'];assert a['p16_original_material_ranges']==rr['material_ranges'] and a['separately_accounted_support_ranges']==row['new_personal_display_ranges_outside_original_material']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr)==[[1,len(old.splitlines())]] and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)==[[1,len(cur.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2522]['new_personal_display_ranges_outside_original_material']==[[1,36],[50,57],[68,93]] and rows[2523]['new_personal_display_ranges_outside_original_material']==[[125,180]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-11A-95E.json');assert d['delegated_observations']==obs and len(obs)==15 and len(d['new_qualified_ledger_rows'])==7
registration=(R/d['central_registration']).read_bytes();receipt=load(d['central_receipt']);assert sha(registration)==receipt['registration_sha256'];plan=json.loads(registration);assert len(plan['rows'])==len(receipt['reads'])==9
for p,r in zip(plan['rows'],receipt['reads']):assert all(p[k]==r[k] for k in ('path','revision','git_blob','sha256')) and p['planned_display_ranges']==r['actual_display_ranges']
with tarfile.open(R/'delegated/central-11a-current-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json'));src={x['path']:t.extractfile(proof['objects'][x['blob']]['archive']).read().decode().splitlines() for x in proof['paths']}
 assert 'Never use `unwrap` or `expect` outside tests.' in src['crates/AGENTS.md'][125]
 readiness=src['crates/adventuresim-tactical-client/src/tactical_scene_viewer/building_review/readiness.rs'];gpu=src['crates/adventuresim-tactical-client/src/tactical_scene_viewer/gpu_readiness.rs'];viewer=src['crates/adventuresim-tactical-client/src/tactical_scene_viewer.rs'];buildings=src['crates/adventuresim-tactical-client/src/tactical_scene_viewer/buildings.rs']
 assert '.unwrap()' in readiness[296] and '.expect(' in readiness[298] and '.expect(' in gpu[36] and '.expect(' in gpu[46] and '.expect(' in viewer[1245]
 assert 'captured_views_ready' in readiness[292] and '.run_if(building_review::ready)' in viewer[962] and 'is_sign || self.boundaries.contains' in readiness[162]
 assert 'spawn_openings' in buildings[53] and 'building.transform()?' in buildings[54] and 'compile_tactical_building_collider' in buildings[68]
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 e=ledger[ident];assert e['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-11A-95E.json' and e['repair']=='not started' and e['validation']=='not performed' and 'open' in e['discovery']
assert 'ASSESSMENT-DELEGATED-RECONCILIATION-11A-95E.json' in ledger['P16-CITY-GATE-OPENING-NAMESPACE']['further_current_evidence']
assert 'three of nine checkpoint11' in d['open'] and '3713' in d['open']
print('Verified five attributed P16 roles, original exclusions, current source-rule mismatches and seven qualified ledger relationships; no product validation or lens closure.')
