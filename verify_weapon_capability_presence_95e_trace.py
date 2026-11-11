#!/usr/bin/env python3
"""Verify selected source and preserved audit records without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-WEAPON-CAPABILITY-PRESENCE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1257-1264.json');assert a['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'] and len(raw.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);source[x['path']]=raw.decode()
module='crates/adventuresim-stdb-module/src/';core='crates/adventuresim-core/src/'
weapon=source[module+'weapon_instance.rs'];evaluation=source[module+'weapon_instance/evaluation.rs'];cap=source[module+'capability.rs'];mass=source[module+'capability/mass.rs'];projection=source[module+'capability/equipment_projection.rs'];smith=source[module+'smithing.rs'];item=source[module+'item.rs'];corecap=source[core+'capability.rs']
assert 'initialize_personal_weapon' in weapon and 'replace_design(ctx, object.id, &design)' in weapon
assert 'EvaluatedWeapon::decode(&instance.recipe).ok()?' in evaluation and '== *instance' in evaluation
assert 'parametric weapon {} has no valid physical recipe' in cap and 'parametric inventory weapon has valid physical recipe' in mass
assert 'fn weapon_weight(&self) -> f32' in cap and 'map_or(0.0, |item| item.weight)' in cap
assert 'weight: definition.weight_kg' in item and 'instance.map_or(item.weight, |value| value.mass_kg)' in projection
assert 'HEAVY_WEAPON_MIN_WEIGHT: f32 = 4.0' in corecap and 'equipment.weapon_weight() >= HEAVY_WEAPON_MIN_WEIGHT' in corecap
assert 'crate::weapon_instance::replace_design(ctx, object.id, &design)' in smith and 'capabilities.meets(role_requirements(&role))' in source[module+'strategic/governance.rs']
actor=source[module+'world_actor.rs'];presence=source[module+'world_actor/presence.rs'];roster=source[module+'world_actor/roster.rs'];assert 'contextual_treatment_claim_matches' in actor and 'Conflicting contextual contact retry' in actor and 'contextual_contact_decision' in actor;assert 'context_membership_valid_at' in presence and 'exact_case_site_visible_to_observer_view' in presence;assert 'if row.is_open()' in roster and 'row.left_at = Some' in roster
archive=tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz');roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1257,1265))
for x in h['historical_source_roles']:
 raw=archive.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and x['new_personal_read_ranges']==[[1,x['lines']]];role=roles[x['original_index']];assert role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1257-1264.json';assert role.get('current_observations',[])==x['prior_current_observations']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=json.loads(raw)
assert records['segovb/PARTIAL-L12-322-001.json']['fresh_personal_body_reads']==[[1,1000]] and records['segovb/PARTIAL-L12-322-001.json']['unread_tail']==[[1001,1353]]
assert records['segovb/PARTIAL-L12-322-002-EOF.json']['fresh_personal_body_reads']==[[1001,1353]] and records['segovb/PARTIAL-L12-322-002-EOF.json']['unread_tail']==[]
q=load('RECONCILIATION-059.json')['entries']
for i,ident in [(1040,'Q-L12-5724af7ed9632f9ad9e4'),(1046,'Q-L12-09e949f0c0a4f8354fa8')]:assert q[i]['contract_id']==ident and q[i]['current_reconciliation'].startswith('partial ')
b=load('BASELINE-95E-20261110.json');assert all(sum(x['id']==c['id'] for x in b['newly_assessed_ledger']['entries'])==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(b['newly_assessed_ledger']['entries'])
print('Verified eight historical bodies, earlier partial qualifications and selected source chains; no product execution.')
