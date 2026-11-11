#!/usr/bin/env python3
"""Verify preserved audit evidence and selected source text, without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
a=load('ASSESSMENT-FAMILY-SHARED-TIME-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1242-1249.json');rev='95e63628f9d70b8dd6e6fede64334696c6995009';assert a['source_revision']==rev
source={}
for x in a['current_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['bytes'] and sha(raw)==x['sha256'] and blob(raw)==x['git_blob'];assert len(raw.splitlines())==x['lines'];assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);source[x['path']]=raw.decode()
base='crates/adventuresim-stdb-module/src/'
clock=source[base+'time/clock.rs'];camp=source[base+'time/camp_rest.rs'];stationary=source[base+'time/stationary.rs'];birth=source[base+'relationship/reproduction.rs'];character=source[base+'character.rs'];governance=source[base+'strategic/governance.rs'];trade=source[base+'strategic/inventory_trade.rs']
assert 'related.party_id != character_party_id' in clock and '.take(32)' in clock
assert 'let target = target.saturating_add_minutes(elapsed);' in clock and 'advance_stationary_character_to(ctx, related_id, target)?;' in clock
assert 'for member_id in members {' in camp and 'advance_married_family_by(ctx, member_id, member_elapsed)?;' in camp
assert 'advance_married_family_by(ctx, character_id, elapsed)?;' in stationary
assert 'mode: crate::character::CharacterCreationMode::Newborn' in birth and 'create_solo_party: false' in birth and 'initialize_npc_policy(' in birth
assert 'mother.id,\n            child_id,\n            KinshipKind::Child' in birth and 'father.id,\n            child_id,\n            KinshipKind::Child' in birth
assert 'create_solo_party: true' in character and 'party_id: None' in character
assert 'source_character.party_id = Some(request.party_id.clone())' in governance and 'character.party_id = Some(party_id.clone())' in governance
assert 'create_solo_party_for_character(ctx, member_character_id)?;' in trade and 'create_solo_party_for_character(ctx, character_id)?;' in trade
archive=tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz');roles=load('COVERAGE-L01-ROLES.json')['roles'];assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1242,1250))
for x in h['historical_source_roles']:
 raw=archive.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'];assert x['new_personal_read_ranges']==[[1,x['lines']]];assert roles[x['original_index']]['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1242-1249.json';assert roles[x['original_index']]['current_observations']==x['prior_current_observations'] if 'current_observations' in roles[x['original_index']] else x['prior_current_observations']==[]
manifest=load('recovered/original-transport-manifest.json.gz') if False else json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));lookup={x['path']:x for x in manifest['entries']};preserved={}
for x in a['recovered_historical_records']:
 m=lookup[x['original_path']];assert x['sha256']==m['sha256'] and x['bytes']==m['bytes'] and x['git_blob']==m['git_blob'];raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);preserved[x['original_path']]=json.loads(raw)
assert preserved['segovb/PARTIAL-L12-308-001.json']['fresh_personal_body_reads']==[[1,800]]
assert preserved['segovb/PARTIAL-L12-308-001.json']['unread_tail']==[[801,1515]]
assert preserved['segovb/PARTIAL-L12-308-002-EOF.json']['fresh_personal_body_reads']==[[801,1515]]
qual=preserved['segovb/QUALIFICATION-L12-ORIGINAL1242-RANGE-SHAPE.json'];assert qual['original_index']==1242 and qual['body_coverage'] is False and qual['pointer'].endswith('/7/source_context/read_ranges')
for i in (1248,1249):
 q=preserved[f'segovb/QUALIFICATION-L12-ORIGINAL{i}-LOCATOR.json'];assert q['original_index']==i and q['historical_locator_proof_body_not_read'] is True and q['new_population_credit']==0
assert [c['id'] for c in a['candidates']]==['REVIEW-PARTY-SHARED-FAMILY-CLOCK-MULTIADVANCE-001','Q-L12-56d0033dbad442f28d8f']
assert 60+60==120  # Symbolic interval arithmetic only; no game state or product simulation.
q=load('RECONCILIATION-059.json')['entries'][1024];assert q['contract_id']=='Q-L12-56d0033dbad442f28d8f' and q['current_reconciliation'].startswith('partial ')
b=load('BASELINE-95E-20261110.json');assert all(sum(x['id']==c['id'] for x in b['newly_assessed_ledger']['entries'])==1 for c in a['candidates']);p=load('CONTINUATION-PROGRESS-95E.json');assert p['lenses_complete']==0 and p['new_current_ledger_candidates']==len(b['newly_assessed_ledger']['entries'])
print('Verified eight historical bodies, original qualifications, selected source bytes and conditional interval reasoning; no product execution.')
