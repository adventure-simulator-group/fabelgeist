#!/usr/bin/env python3
"""Verify bounded opaque reflection and checked length source relationships."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
name='ASSESSMENT-DELEGATED-RECONCILIATION-13E-95E.json';a=load(name);assert a['new_qualified_ledger_rows']==[] and a['further_evidence_for_existing_rows']==[]
old=load('baseline-059/queue.json')['entries'];idx=load('RECONCILIATION-059.json')['entries'];assert old[2332]['contract_id']=='L12-CLOSURE-NATIVE-SATS-REFLECTION-ADAPTER-OWNERS' and old[3445]['contract_id']=='L12-CLOSURE-REPORT-GEOMETRY-TYPED-ERROR-NATIVE-PORT'
for i in (2332,3445):assert idx[i]['current_reconciliation'].startswith('partial ') and name in idx[i]['current_reconciliation']
reg=load(a['central_display_evidence']['registration']);assert a['source_references']==[{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']]
with tarfile.open(R/'delegated/central-13e-windows-proof.tar.gz') as t:
 p=json.load(t.extractfile('proof.json'));bodies={x['sha256']:t.extractfile(p['objects'][x['blob']]['archive']).read() for x in p['paths']};lines={x['identity']:bodies[x['sha256']].decode().splitlines() for x in reg['rows']}
def line(tag,n):return lines[tag][n-1]
lib='Bevy reflect derive lib 0.19.1';fr='Bevy reflect derive FromReflect 0.19.1';op='Bevy reflect derive opaque 0.19.1';owner='current PositiveLength';cargo='current geometry feature declaration'
assert 'ReflectDerive::Opaque(meta)' in line(lib,99) and 'should_auto_derive()' in line(lib,101)
assert 'try_downcast_ref' in line(fr,37) and 'try_downcast_ref' in line(fr,44) and 'FQClone::clone' in line(fr,43)
assert 'try_downcast_ref' in line(op,46) and '*self = #FQClone::clone(value)' in line(op,47) and 'MismatchedTypes' in line(op,99)
assert 'PositiveLength(f32)' in line(owner,15) and '!value.is_finite()' in line(owner,21) and 'value == 0.0' in line(owner,27) and 'Self::from_metres' in line(owner,38)
assert 'default = []' in line(cargo,10) and 'reflect = [' in line(cargo,11) and 'default-features = false' in line(cargo,16)
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-32.json');bundle=next(x for x in cp['bundles'] if x['bundle_id']=='reflect-admission-2563-2564-reconcile')
with tarfile.open(R/bundle['archive']) as t:
 r=json.load(t.extractfile('registration.json'));raw=t.extractfile(r['archive']['path']).read();assert sha(raw)==r['archive']['sha256'];lock=tomllib.loads(t.extractfile('sources/95e-Cargo.lock').read().decode());p=next(x for x in lock['package'] if (x['name'],x['version'])==('bevy_reflect_derive','0.19.1'));assert p['checksum']==sha(raw)
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as pkg:
  for role,member in [(2563,'src/lib.rs'),(2564,'src/from_reflect.rs'),(2565,'src/impls/opaque.rs')]:
   x=next(z for z in reg['rows'] if z.get('role')==role);body=pkg.extractfile('bevy_reflect_derive-0.19.1/'+member).read();assert sha(body)==x['sha256'] and body==bodies[x['sha256']]
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified locked opaque reflection source and checked length owner; no active feature, macro execution or lens closure inferred.')
