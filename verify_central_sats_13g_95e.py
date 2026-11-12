#!/usr/bin/env python3
"""Verify bounded SATS source, owner and selected caller evidence without execution."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();a=load('ASSESSMENT-CENTRAL-SATS-13G-95E.json');assert a['new_qualified_ledger_rows']==[] and a['further_evidence_for_existing_rows']==['ADLER-L11-BASIS-POINTS-WIRE-ADMISSION']
refs=[];bodies={}
for tag,archive in [('13G','delegated/central-13g-windows-proof.tar.gz'),('13G-SUPPORT','delegated/central-13g-support-windows-proof.tar.gz')]:
 reg=load('CENTRAL-'+tag+'-REGISTRATION.json');refs.extend({'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows'])
 with tarfile.open(R/archive) as t:
  p=json.load(t.extractfile('proof.json'))
  for x in p['paths']:
   raw=t.extractfile(p['objects'][x['blob']]['archive']).read();assert sha(raw)==x['sha256'];bodies[x['path']]=raw
assert refs==a['source_references'] and len(refs)==7
native=next(v for k,v in bodies.items() if k.endswith('bf26e6331c7cf9a560ecec507b5e06b11bc77460dec17ac0d553cff5278a5bc4'))
archive=(R/'recovered/l2-native-registration-evidence/archives/spacetimedb-bindings-macro-2.6.1.crate').read_bytes();assert sha(archive)=='fce86c7657dcb42a33ce55f0100f98964dc15b64e79f06803a478676b74d6c82'
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:assert t.extractfile('spacetimedb-bindings-macro-2.6.1/src/sats.rs').read()==native
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-13.json');b=next(x for x in cp['bundles'] if x['bundle_id']=='stdb-macro')
with tarfile.open(R/b['archive']) as t:
 lockraw=t.extractfile('lock/95e6-Cargo.lock').read();assert sha(lockraw)=='cb227fddbf85f10ea6d3bf929827ea395f240eae2d948a2c4c590ae526ba1f73';lock=tomllib.loads(lockraw.decode());pkg=next(x for x in lock['package'] if (x['name'],x['version'])==('spacetimedb-bindings-macro','2.6.1'));assert pkg['checksum']==sha(archive)
def line(path,n):return bodies[path].decode().splitlines()[n-1]
sats=native.decode().splitlines();assert 'Ok(#name {' in sats[431] and 'next_element' in sats[433] and 'duplicate_field' in sats[454] and 'missing_field' in sats[462]
basis='crates/adventuresim-world-schema/src/basis_points.rs';assert 'pub const fn new(value: u16)' in line(basis,24) and 'value <= BASIS_POINTS_PER_WHOLE' in line(basis,25) and 'is_finite' in line(basis,45) and 'pub const fn new(value: i16)' in line(basis,114)
equip='crates/adventuresim-core/src/equipment.rs';assert 'value.is_nan() || value <= 0.0' in line(equip,502) and 'f32::MAX' in line(equip,505) and 'capacity_kg <= 0.0' in line(equip,524) and '.clamp(0.0, 1.0)' in line(equip,527)
enc='crates/adventuresim-stdb-module/src/strategic/encounters.rs';assert 'encumbrance_remaining_multiplier' in line(enc,168) and 'from_unit_f32(remaining.clamp(0.0, 1.0))' in line(enc,173)
assert 'spacetimedb = ["dep:spacetimedb-lib"]' in line('crates/adventuresim-world-schema/Cargo.toml',11) and 'features = ["spacetimedb"]' in line('crates/adventuresim-stdb-module/Cargo.toml',19)
q=load('RECONCILIATION-059.json')['entries'];assert q[131]['contract_id']=='ADLER-L11-BASIS-POINTS-WIRE-ADMISSION' and q[2332]['contract_id']=='L12-CLOSURE-NATIVE-SATS-REFLECTION-ADAPTER-OWNERS' and all('ASSESSMENT-CENTRAL-SATS-13G-95E.json' in q[i]['current_reconciliation'] for i in (131,2332))
assert a['repair']=='not started' and a['validation']=='not performed'
print('Verified conditional native SATS source and selected direct-caller guard; no feature resolution, macro expansion or product execution inferred.')
