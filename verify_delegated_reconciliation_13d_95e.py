#!/usr/bin/env python3
"""Verify selected native scalar source and existing graphics ledger linkage."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-13D-95E.json';a=load(name);ident='ADLER-L11-GRAPHICS-CONFIG-NONFINITE-ADMISSION'
assert a['new_qualified_ledger_rows']==[] and a['further_evidence_for_existing_rows']==[ident]
ledger={x['id']:x for x in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']};assert name in ledger[ident]['further_current_evidence'] and ledger[ident]['repair']=='not started' and ledger[ident]['validation']=='not performed'
old=load('baseline-059/queue.json')['entries'];index=load('RECONCILIATION-059.json')['entries'];assert old[72]['contract_id']==ident
for i in (72,1303,1313):assert index[i]['current_reconciliation'].startswith('partial ') and name in index[i]['current_reconciliation']
reg=load(a['central_display_evidence']['registration']);assert a['source_references']==[{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']]
with tarfile.open(R/'delegated/central-13d-windows-proof.tar.gz') as t:
 proof=json.load(t.extractfile('proof.json')); bodies={x['sha256']:t.extractfile(proof['objects'][x['blob']]['archive']).read() for x in proof['paths']}
 lines={x['identity']:bodies[x['sha256']].decode().splitlines() for x in reg['rows']}
def line(tag,n):return lines[tag][n-1]
assert 'from_str_with_options(input, Options::default())' in line('serde-saphyr 1.2.0 API',88)
assert 'Concrete `f32`/`f64` targets are unaffected' in line('serde-saphyr 1.2.0 options',254) and 'reject_non_finite_typeless_float: true' in line('serde-saphyr 1.2.0 options',514)
assert 'parse_yaml12_float' in line('serde-saphyr 1.2.0 deserializer',981) and 'visitor.visit_f32(v)' in line('serde-saphyr 1.2.0 deserializer',982)
assert 'Ok(T::nan())' in line('serde-saphyr 1.2.0 scalar conversion',329) and 'Ok(T::infinity())' in line('serde-saphyr 1.2.0 scalar conversion',330)
assert 'Ok(v)' in line('serde_core 1.0.229 visitor support',161) and 'num_self!(f32:visit_f32)' in line('serde_core 1.0.229 visitor support',452)
assert 'serde_saphyr::from_str(text)' in line('pinned current graphics config support',209) and 'root_occlusion <= 0.0' in line('pinned current graphics config support',310)
assert 'interaction.maximum_push < interaction.minimum_push' in line('pinned current graphics config support',321) and 'value.is_finite()' in line('pinned current graphics config support',376)
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-31.json');b=next(x for x in cp['bundles'] if x['bundle_id']=='saphyr-2573-2576-reconcile');members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 lock=tomllib.loads(t.extractfile('source/root-current.lock').read().decode());packages={(x['name'],x['version']):x for x in lock['package']}
 for package,version,arch,changed,roles in [('serde-saphyr','1.2.0','source/serde-saphyr-1.2.0.crate',True,[(2573,'src/de/api.rs'),(2574,'src/de/options.rs'),(2575,'src/de/deserializer.rs'),(2576,'src/parse_scalars.rs')]),('serde_core','1.0.229','source/serde_core-1.0.229.crate',False,[(2588,'src/de/impls.rs')])]:
  m=members[arch];assert m['changed']==changed and packages[(package,version)]['checksum']==m['original_sha256']
  raw=t.extractfile(arch).read()
  if not changed:assert sha(raw)==m['original_sha256']
  else:assert 'Reconstructed public copy' in m['qualification']
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as archive:
   for role,path in roles:
    body=archive.extractfile(f'{package}-{version}/{path}').read();r=next(x for x in reg['rows'] if x.get('role')==role);assert sha(body)==r['sha256'] and body==bodies[r['sha256']]
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified locked native package/source identities and existing conditional graphics relationship; no parser, product execution or lens closure.')
