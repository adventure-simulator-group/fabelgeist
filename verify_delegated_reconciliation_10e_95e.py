#!/usr/bin/env python3
"""Bind building reconciliation to exact attributed displays and historical scopes."""
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
name='ASSESSMENT-DELEGATED-BUILDING-P16-2496-2503.json';assessment=load(name);rows={x['original_index']:x for x in assessment['historical_source_roles']};assert set(rows)==set(range(2496,2504))
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('building-roof-abutments-hair',[2496,2497],'connected_observations'),('p16-collision',[2498,2499],'findings_and_questions'),('p16-doors-builder',[2500,2501],'findings_and_questions'),('p16-finish-lod',[2502,2503],'findings')]:
 b=bs[bundle];report=load(b['report_path']);obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in report[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=t.extractfile(row['historical_member']).read();cur=t.extractfile(row['current_member']).read()
   assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(cur) and cc['byte_identical_to_historical']==(old==cur)
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   if i in (2496,2497):
    a=next(x for x in receipt['roles'] if x['index']==i);hr=a['historical_displayed_windows'];cr=a['displayed_windows'];assert not hr and a['historical_unread_complement']==row['historical_unread_ranges']==[[1,len(old.splitlines())]] and not a['unread_complement'] and cur==old and work[i]['status']=='bounded-current-trace-only'
   elif i in (2498,2499,2500,2501):
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges']
   else:
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']!=cc['source_revision']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cc['source_revision']);hr=a['contiguously_displayed_ranges'];cr=z['contiguously_displayed_ranges'];assert a['original_material_ranges']==rr['material_ranges'] and row['new_connected_support_ranges_outside_original_role']==a['personally_displayed_beyond_original_scope']
    if i==2503:assert row['historical_unread_ranges']==cc['unread_ranges']==[[1,93],[161,666]] and work[i]['status']=='source-metadata-partial' and work[i]['assessment']==name
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr) and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)
   if i in (2498,2499,2500,2501,2502):assert row['new_personal_read_ranges']==[[1,len(old.splitlines())]] and cc['personal_ranges']==[[1,len(cur.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2502]['new_connected_support_ranges_outside_original_role']==[[30,53],[89,193]] and rows[2503]['new_connected_support_ranges_outside_original_role']==[[94,94],[138,160]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-10E-95E.json');assert d['delegated_observations']==obs and len(obs)==15 and len(d['new_qualified_ledger_rows'])==11
registration=(R/d['central_registration']).read_bytes();receipt=load(d['central_receipt']);assert sha(registration)==receipt['registration_sha256'];plan=json.loads(registration)
assert len(plan['rows'])==len(receipt['reads'])==6
for p,r in zip(plan['rows'],receipt['reads']):assert p['path']==r['path'] and p['revision']==r['revision'] and p['git_blob']==r['git_blob'] and p['sha256']==r['sha256'] and p['planned_display_ranges']==r['actual_display_ranges']
ledger={e['id']:e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 e=ledger[ident];assert e['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10E-95E.json' and e['repair']=='not started' and e['validation']=='not performed' and 'open' in e['discovery']
assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10E-95E.json' in ledger['CURRENT-ROOF-CHECKED-POSITION-RETAINED-RAW-RELATION']['further_current_evidence']
assert 'seven checkpoint10 bundles' in d['open'] and '3713' in d['open'] and '2496/2497' in d['open']
print('Verified eight building roles, current-only and partial read limits, personal receipts and 11 connected ledger relationships; no lens or product validation claimed.')
