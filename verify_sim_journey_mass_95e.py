#!/usr/bin/env python3
"""Verify source-byte evidence and rational illustration; run no product code."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-SIM-JOURNEY-MASS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1283.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed';s={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'] and x['revision']==rev;assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
base='crates/adventuresim-strategic-sim/src/live_core/';journey=s[base+'journey_policy.rs'];survival=s[base+'survival.rs'];native=s['crates/adventuresim-stdb-module/src/capability/mass.rs'];weapon=s['crates/adventuresim-stdb-module/src/weapon_instance.rs'];boot=s[base+'bootstrap.rs'];policy=s[base+'departure_policy.rs'];core=s['crates/adventuresim-core/src/equipment.rs'];fraction=s['crates/adventuresim-core/src/inventory_measurement.rs'];amount=s['crates/adventuresim-stdb-module/src/inventory_amount.rs'];travel=s[base+'travel.rs'];expedition=s[base+'expedition.rs'];rules=s['crates/AGENTS.md']
assert b'item.weight.max(0.0) * public_effective_inventory_quantity(quantity, remaining)' in survival and b'lot.mass_kg.max(0.0)' in survival and b'public_party_load_and_capacity(party_id)' in survival
assert b'crate::weapon_instance::combat_geometry(ctx, inventory.id, &item.id)' in native and b'crate::weapon_instance::fitted_holder_mass(ctx, inventory.id, &item.id)' in native and b'lot.mass_kg.max(0.0)' in native
assert b'pub fn backend_weapon_instances' in weapon and b'pub fn backend_weapon_holder_instances' in weapon and b'pub mass_grams: u32' in weapon and b'evaluated.derived().mass_kg' in weapon
assert b'query.from.backend_weapon_instances()' not in boot and b'query.from.backend_weapon_holder_instances()' not in boot and b'query.from.inventory_object()' in boot
assert b'MIN_DEPARTURE_ENCUMBRANCE_REMAINING_BPS: u32 = 2_000' in policy and b'if remaining_bps < MIN_DEPARTURE_ENCUMBRANCE_REMAINING_BPS' in survival and b'(1.0 - burden_kg / capacity_kg).clamp(0.0, 1.0)' in core
assert Fraction(1)-Fraction(77+2,100)==Fraction(21,100) and Fraction(1)-Fraction(77+4,100)==Fraction(19,100)
assert b'ConsumableFractionMicros::try_new(value)' in journey and b'.expect("public consumable fraction must not exceed one whole")' in journey and b'fraction_micros > Self::MICROS_PER_WHOLE' in fraction and b'ConsumableFractionMicros::WHOLE.get()' in amount and b'.checked_sub(consumed)' in amount and b'Never use `unwrap` or `expect` outside tests.' in rules
assert b'classify_post_rest_progress(' in travel and b'requested_rest_minutes={rest_minutes};actual_rest_minutes={actual_rest_minutes}' in travel
assert b'.map_err(|reason| self.public_camp_coherence_error(party_id, reason))' in travel and b'party_member()' in expedition and b'alive: character.alive' in expedition
assert b'teardown_all_dead_strategic_party(ctx, &party_id)?' in s['crates/adventuresim-stdb-module/src/time/camp_rest.rs'];gov=s['crates/adventuresim-stdb-module/src/strategic/governance.rs'].splitlines()[332:418];assert b'finish_party_journey(ctx, party_id)' in b'\n'.join(gov) and b'party_member().id().delete' not in b'\n'.join(gov)
x=h['historical_source_roles'][0];assert x['original_index']==1283 and x['lines']==216 and x['new_personal_read_ranges']==[[1,216]];roles=load('COVERAGE-L01-ROLES.json')['roles'];assert roles[1283]['status']=='full-historical-body-trace-open' and roles[1283]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1283.json'
with tarfile.open(r/x['evidence_archive'],'r:gz') as ar:
 raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==216 and raw==journey
previous=load('ASSESSMENT-INVESTIGATION-METHOD-RANK-95E.json');rec={x['original_path']:x for x in previous['recovered_historical_records']};raw=gzip.decompress((r/rec['segovb/READ-L12-337.json']['preserved_bytes']).read_bytes());receipt=json.loads(raw);row=next(x for x in receipt['rows'] if x['original_index']==1283);assert row['fresh_personal_body_reads']==[[1,216]] and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
q=load('RECONCILIATION-059.json')['entries'];assert q[1075]['current_reconciliation'].startswith('partial ') and q[1070]['current_reconciliation'].startswith('partial ')
b=load('BASELINE-95E-20261110.json');ledger=b['newly_assessed_ledger']['entries'];assert all(sum(x['id']==c['id'] for x in ledger)==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified role 1283, selected mass and fraction evidence, qualified safeguards and symbolic arithmetic; no product execution.')
