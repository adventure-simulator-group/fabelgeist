#!/usr/bin/env python3
"""Bind wall/window reconciliation to exact attributed displays and scopes."""
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
name='ASSESSMENT-DELEGATED-BUILDING-P16-2504-2512.json';assessment=load(name);rows={x['original_index']:x for x in assessment['historical_source_roles']};assert set(rows)=={2504,2505,2506,2507,2508,2509,2511,2512}
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles'];reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];obs=[]
for bundle,ids,key in [('p16-building-member-shapes-hair',[2504,2505],'connected_observations'),('p16-walls-windows',[2506,2507],'findings_and_questions'),('p16-window-leaf-spec',[2508,2509],'findings_and_questions'),('p16-building-presentation-hair',[2511,2512],'connected_observations')]:
 b=bs[bundle];report=load(b['report_path']);obs.extend({'bundle_id':bundle,'author':b['agent'],'report':b['report_path'],'original_finding':x} for x in report[key])
 with tarfile.open(R/b['archive']) as t:
  for i in ids:
   row=rows[i];rr=reg[i];cc=row['current_comparison'];receipt=json.load(t.extractfile(row['receipt_member']));old=t.extractfile(row['historical_member']).read();cur=t.extractfile(row['current_member']).read()
   assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['sha256']==sha(old)==work[i]['historical_sha256'] and cc['sha256']==sha(cur) and cc['byte_identical_to_historical']==(old==cur)
   assert row['original_registry_scope']['original_material_ranges']==rr['material_ranges'] and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   if i in (2504,2505,2511,2512):
    a=next(x for x in receipt['roles'] if x['original_index']==i);hr=a['p16_display_windows'];cr=a['current_display_windows'];assert row['historical_unread_ranges']==a['p16_unread_complement'] and cc['unread_ranges']==a['current_unread_complement'] and not a['0393_a120_6533_personal_display_windows']
   elif i in (2506,2507):
    a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']!=cc['source_revision']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['source_identity']==cc['source_revision']);hr=a['display_chunks'];cr=z['display_chunks'];assert a['p16_original_material_ranges']==rr['material_ranges'] and row['new_connected_support_ranges_outside_original_role']==a['separately_accounted_support_ranges']
   else:
    a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');hr=a['actual_display_ranges'];cr=z['actual_display_ranges'];assert not a['unread_ranges'] and not z['unread_ranges']
   assert row['actual_personal_display_chunks']==hr and row['new_personal_read_ranges']==norm(hr) and cc['actual_personal_display_chunks']==cr and cc['personal_ranges']==norm(cr)
   if i==2512:assert row['historical_unread_ranges']==[[101,199],[231,429],[526,568]] and not cc['unread_ranges'] and cc['personal_ranges']==[[1,556]] and work[i]['status']=='source-metadata-partial' and work[i]['assessment']==name
   else:assert row['new_personal_read_ranges']==[[1,len(old.splitlines())]] and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']==name
assert rows[2504]['current_comparison']['unread_ranges']==[[45,68]] and rows[2511]['current_comparison']['unread_ranges']==[[60,150]]
assert rows[2505]['current_comparison']['unread_ranges']==[[31,198],[231,249],[321,343],[392,392],[429,429]]
d=load('ASSESSMENT-DELEGATED-RECONCILIATION-10F-95E.json');assert d['delegated_observations']==obs and len(obs)==16 and len(d['new_qualified_ledger_rows'])==11
registration=(R/d['central_registration']).read_bytes();receipt=load(d['central_receipt']);assert sha(registration)==receipt['registration_sha256'];plan=json.loads(registration);assert len(plan['rows'])==len(receipt['reads'])==6
for p,r in zip(plan['rows'],receipt['reads']):assert p['path']==r['path'] and p['revision']==r['revision'] and p['git_blob']==r['git_blob'] and p['sha256']==r['sha256'] and p['planned_display_ranges']==r['actual_display_ranges']
ledger={e['id']:e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']}
for ident in d['new_qualified_ledger_rows']:
 e=ledger[ident];assert e['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10F-95E.json' and e['repair']=='not started' and e['validation']=='not performed' and 'open' in e['discovery']
for ident in ('P16-COLLISION-BAR-SOURCE-IDENTITY','P16-WINDOW-LEAF-UNBOUNDED-WORK','BUILDING-COLLISION-AND-GLAZING-TYPED-LEAVES','CURRENT-WEAR-REGION-FRAME-OWNER-RELATION','ADLER-L11-CITY-SURFACE-MATERIAL-CONTRACT'):assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10F-95E.json' in ledger[ident]['further_current_evidence']
assert 'same window operation' in ledger['P16-WINDOW-CLOSURE-REFERENCE-RESOLUTION']['relationship_qualification'] and 'three checkpoint10 bundles' in d['open'] and '3713' in d['open']
print('Verified eight wall/window roles, exact current and historical unread limits, attributed receipts and 11 connected ledger relationships; no lens or product validation claimed.')
