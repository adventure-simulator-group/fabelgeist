#!/usr/bin/env python3
"""Check bounded garment source and ledger relationships without execution."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();a=load('ASSESSMENT-CENTRAL-GARMENT-TUBE-13K-95E.json');reg=load('CENTRAL-13K-REGISTRATION.json');proof={}
assert a['source_references']==[{'revision':x['revision'],'path':x['path'],'lines':x['planned_display_ranges']} for x in reg['rows']] and len(reg['rows'])==12
with tarfile.open(R/'delegated/central-13k-windows-proof.tar.gz') as t:
 p=json.load(t.extractfile('proof.json'))
 for x in p['paths']:
  raw=t.extractfile(p['objects'][x['blob']]['archive']).read();assert sha(raw)==x['sha256'];proof[(x['revision'],x['path'])]=raw
cur='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e';armor='crates/fabelgeist-armor/src/';char='crates/adventuresim-character-creator/src/'
roles=load('COVERAGE-L01-ROLES.json')['roles']
for role,path in [(2623,armor+'garment_armor.rs'),(2624,armor+'garment_plate_design.rs')]:
 assert proof[(old,path)]==proof[(cur,path)] and sha(proof[(old,path)])==roles[role]['historical_sha256'] and roles[role]['status']=='unassessed' and roles[role]['snapshot']=='pr-working'
def line(path,n):return proof[(cur,path)].decode().splitlines()[n-1]
assert 'GarmentArmorKind::Fauld => Self::Fauld' in line(armor+'garment_plate_design.rs',41) and 'kind == GarmentArmorKind::Fauld' in line(armor+'garment_plate_design.rs',82)
assert 'self.plate_shape.validate(self.kind)' in line(armor+'garment_armor.rs',91) and 'front_arch.0 >= self.length.0' in line(armor+'garment_armor.rs',123)
assert 'pub mod gpu;' in line(armor+'lib.rs',109) and 'record_garment_tube' in line(armor+'gpu/mod.rs',81)
assert 'pub fn record_garment_tube' in line(armor+'gpu/garment.rs',195) and 'design.validate()' in line(armor+'gpu/garment.rs',201) and 'GarmentArmorKind::MailSkirt | GarmentArmorKind::PaddedSkirt' in line(armor+'gpu/garment.rs',204)
assert 'design.flare.unit()' in line(armor+'gpu/garment.rs',234) and 'design.wall_thickness.metres()' in line(armor+'gpu/garment.rs',237) and 'pub fn record_fauld' in line(armor+'gpu/garment.rs',270) and 'GarmentPlateShape::Fauld' in line(armor+'gpu/garment.rs',277)
assert 'Kind::MailSkirt | Kind::PaddedSkirt | Kind::Fauld | Kind::Tassets' in line(char+'device_garment.rs',267) and 'record_fauld' in line(char+'device_garment_skirt.rs',134) and 'record_garment_tube' in line(char+'device_garment_skirt.rs',136)
assert 'record_garment_tube' in line(char+'device_garment_tube.rs',404) and 'record_garment_tube' in line(char+'device_chausses.rs',240) and 'MailChausses' in line(char+'device_extremities.rs',24) and 'PaddedChausses' in line(char+'device_extremities.rs',25)
ident='local-candidate-garment-tube-kind-admission';rows=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(rows)==load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] and len(rows)>=1025 and sum(x['id']==ident for x in rows)==1
prior=load('delegated/armor-garment-shape-report.json');assert any(x['id']==ident for x in prior['candidate_findings']);assert a['new_qualified_ledger_rows']==[ident]
q=load('RECONCILIATION-059.json')['entries'][1818];assert q['historical_status']=='open-evidence-or-disposition' and 'exact alias is not established and q1818 remains open' in q['current_reconciliation']
assert a['repair']=='not started' and a['validation']=='not performed' and load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled']==0
print('Verified bounded garment-tube source relationships and earlier candidate identity; no device, geometry, product execution or queue closure inferred.')
