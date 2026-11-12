#!/usr/bin/env python3
"""Verify native, physics and P16 evidence without expanding original scopes."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
def norm(rs):
 out=[]
 for lo,hi in sorted(rs):
  assert 1<=lo<=hi
  if out and lo<=out[-1][1]+1:out[-1][1]=max(out[-1][1],hi)
  else:out.append([lo,hi])
 return out
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');bundles={b['bundle_id']:b for b in cp['bundles']};sel={'sdk-noiz-sats-hair':[2476,2477],'physics-collider-count':[2481,2482],'p16-config':[2483,2484,2485],'p16-body-target-hair':[2486,2487]};d=load('ASSESSMENT-DELEGATED-NATIVE-PHYSICS-P16-2476-2487.json');rows={x['original_index']:x for x in d['historical_source_roles']};work=load('COVERAGE-L01-ROLES.json')['roles'];registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];central=load('ASSESSMENT-DELEGATED-RECONCILIATION-10C-95E.json')
assert sorted(rows)==sorted(i for indexes in sel.values() for i in indexes) and len(central['delegated_observations'])==17
for name,indexes in sel.items():
 b=bundles[name];raw=(R/b['archive']).read_bytes();assert sha(raw)==b['archive_sha256'] and len(raw)==b['archive_bytes']
 with tarfile.open(R/b['archive']) as t:
  receipt_name='personal-read-receipt.json' if name=='physics-collider-count' else 'read-receipt.json';receipt=json.load(t.extractfile(receipt_name));report=json.load(t.extractfile(b['report_archive_member']));assert report==load(b['report_path'])
  key={'sdk-noiz-sats-hair':'observations','physics-collider-count':'findings_and_qualified_questions','p16-config':'findings_and_questions','p16-body-target-hair':'connections'}[name];observed=[x for x in central['delegated_observations'] if x['bundle_id']==name];assert [x['original_finding'] for x in observed]==report[key] and all(x['author']==b['agent'] for x in observed)
  for i in indexes:
   row=rows[i];rr=registry[i];cc=row['current_comparison'];assert row['new_personal_read_attribution']==b['agent'] and row['evidence_archive']==b['archive'] and row['receipt_member']==receipt_name and work[i]['status']=='full-historical-body-trace-open' and work[i]['historical_assessment']=='ASSESSMENT-DELEGATED-NATIVE-PHYSICS-P16-2476-2487.json'
   source=t.extractfile(row['historical_member']).read();assert sha(source)==row['sha256']==work[i]['historical_sha256'] and len(source.splitlines())==row['lines'];assert row['original_registry_scope']['original_material_ranges']==rr.get('material_ranges') and row['original_registry_scope']['original_source_or_other_lens_credit']==rr.get('source_or_other_lens_credit')
   if name=='sdk-noiz-sats-hair':
    r=next(x for x in receipt['reads'] if x['original_index']==i);old=r['actual_display_ranges'];assert cc['separate_current_body_display'] is False and row['current_member'] is None
    package=t.extractfile(cc['archive_member']).read();assert sha(package)==cc['archive_sha256']
    with tarfile.open(fileobj=io.BytesIO(package)) as native:assert native.extractfile(cc['source_member']).read()==source
    for member in ('lock/6533-Cargo.lock','lock/95e6-Cargo.lock'):
     lock=tomllib.loads(t.extractfile(member).read().decode());pkg=next(x for x in lock['package'] if x['name']==cc['package'] and x['version']==cc['version']);assert pkg['checksum']==cc['archive_sha256']
   else:
    current=t.extractfile(row['current_member']).read();assert sha(current)==cc['sha256'] and len(current.splitlines())==cc['lines']
    if name=='physics-collider-count':
     a=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['revision']==row['snapshot_head']);z=next(x for x in receipt['assigned_personal_reads'] if x['role']==i and x['revision']==cc['source_revision']);old=a['actual_full_source_displayed'];now=z['actual_full_source_displayed'];assert a['original_role_scope_personally_displayed']==rr['material_ranges'] and a['connected_personally_displayed_outside_original_scope']==row['new_connected_support_ranges_outside_original_role']
    elif name=='p16-config':
     a=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='physical-p16');z=next(x for x in receipt['reads'] if x['role']==i and x['kind']=='current');old=a['actual_display_ranges'];now=z['actual_display_ranges']
    else:
     a=next(x for x in receipt['reads'] if x['original_index']==i);old=a['p16_actual_display_ranges'];now=a['current_actual_display_ranges'];assert not a['base_actual_display_ranges'] and not a['older_origin_actual_display_ranges']
    assert now==cc['actual_personal_display_chunks'] and norm(now)==cc['personal_ranges']==[[1,cc['lines']]]
   assert old==row['actual_personal_display_chunks'] and norm(old)==row['new_personal_read_ranges']==[[1,row['lines']]]
   if i in range(2483,2488):assert row['original_registry_scope']['original_registry_status']=='navigation_only_no_body_analysis' and row['original_registry_scope']['original_source_or_other_lens_credit']=='none from metadata registration' and 'physical-p16 dirty snapshot' in row['new_personal_read_revision']
assert rows[2482]['original_registry_scope']['original_material_ranges']==[[1,133],[212,218]] and rows[2482]['new_connected_support_ranges_outside_original_role']==[[134,211]]
assert 'not a separate new delegate registration' in load('QUALIFICATION-COLLIDER-REGISTRATION-CLARIFICATION.json')['clarification']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];ids={x['id']:x for x in ledger};assert len(ids)==len(ledger) and len(central['new_qualified_ledger_rows'])==8
for ident in central['new_qualified_ledger_rows']:
 assert ids[ident]['record']=='ASSESSMENT-DELEGATED-RECONCILIATION-10C-95E.json' and ids[ident]['repair']=='not started' and ids[ident]['validation']=='not performed'
for ident in ('SEGOVB-L12-CONNECTED-PHYSICS-COLLIDER-NATIVE-COUNT-ADMISSION','SEGOVB-L12-CONNECTED-PHYSICS-COLLIDER-CAPACITY-ERROR-STATE','ADLER-L11-BOX-MEDIAL-NORMAL-PARITY','REVIEW-PHYSICS-SHORT-NONZERO-CAPSULE-95E','SEGOVB-L12-PHYSICS-COLLIDER-UPDATE-COUNT-REASON','REVIEW-BODY-REFERENCE-ACCELERATION-RATIO-001'):
 assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10C-95E.json' in ids[ident]['further_current_evidence']
progress=load('CONTINUATION-PROGRESS-95E.json');assert progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0 and progress['new_current_ledger_candidates']==len(ledger);assert 'ASSESSMENT-DELEGATED-RECONCILIATION-10C-95E.json' in progress['delegated_batch10_partial_reconciliation']
print('Verified nine attributed historical bodies, native lock/package identity, original material scopes, separated collider support and eight qualified ledger rows.')
