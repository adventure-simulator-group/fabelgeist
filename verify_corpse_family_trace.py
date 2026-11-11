"""Verify selected corpse-family evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L07-CORPSE-FAMILY-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 7 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 10
base = 'crates/adventuresim-stdb-module/src/'
corpse = sources[base + 'corpse.rs']
must(corpse, ['const MAX_BOUND_FAMILY_MEMBERS: usize = 8;',
              'materialize_corpse_family_bindings(ctx, &corpse_id, &settlement_id, &[])?;',
              'if npc.home_settlement_id != settlement_id',
              'family_resident_character_ids: &[u64]',
              'fn permission_kind_for_npc(', 'fn apply_unauthorized_consequences(',
              'fn burning_social_penalty(', '.corpse_family_binding()'])
character = sources[base + 'character.rs']
must(character, ['pub fn transition_character_to_dead_at(',
                 'crate::corpse::persist_character_death_corpse(',
                 'crate::relationship::settle_relationship_lifecycle_for_death('])
assert character.index('crate::corpse::persist_character_death_corpse(') < character.index('crate::relationship::settle_relationship_lifecycle_for_death(')
outbreak = sources[base + 'outbreak.rs']
must(outbreak, ['fn materialize_patient_corpse(', 'crate::character::transition_character_to_dead_at(',
                'let canonical_family = ctx', '.filter(exposure.patient_character_id)',
                '.filter(|profile| profile.home_settlement_id == settlement_id)',
                'crate::corpse::materialize_corpse_family_bindings(',
                'pub(crate) fn discover_case_corpses(', 'corpse.discovering_party_id = party_id.clone();'])
assert outbreak.index('crate::character::transition_character_to_dead_at(') < outbreak.index('let canonical_family = ctx') < outbreak.index('crate::corpse::materialize_corpse_family_bindings(')
family = sources[base + 'relationship/family.rs']
must(family, ['if family.len() == roles.len()', 'KinshipKind::Parent', 'KinshipKind::Child',
              'KinshipKind::Sibling', 'ensure_kinship(ctx, subject_id, related_id, kind, StrategicMinute::ZERO);'])
model = sources[base + 'relationship/model.rs']
must(model, ['pub enum KinshipKind', 'pub struct CharacterKinship', 'pub established_minute: StrategicMinute'])
marriage = sources[base + 'relationship/marriage.rs']
must(marriage, ['pub fn settle_due_weddings(', 'KinshipKind::Spouse', 'fn resolve_marriage('])
population = sources[base + 'settlement_population.rs']
must(population, ['let (drafts, household_groups) = settlement_resident_drafts(ctx, settlement_id)?;',
                  'insert_resident_draft(ctx, settlement_id, draft)?;',
                  'ensure_seeded_family_households(ctx, settlement_id, &household_groups)?;'])
drafts = sources[base + 'settlement_population/resident_drafts.rs']
must(drafts, ['drafts.as_chunks_mut::<4>()', 'drafts\n        .chunks(4)',
              'households.extend(finalize_household_demographics(&mut ambient));'])
persistence = sources[base + 'settlement_population/resident_persistence.rs']
must(persistence, ['home_settlement_id: settlement_id.into()',
                   'let resident = insert_profile(ctx, settlement_id, &draft, &prepared);'])
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
assert 'outbreak.rs:968:' in raw.decode() and 'corpse.rs:1043:' in raw.decode()
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 7
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any(x['proposed_id'] == 'PL13-242-CORPSE-FAMILY-BINDING-POLICY-PENDING'
           for x in old['evidence']['connected_contracts'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
unions = old['evidence']['partial_read_union']
assert unions[0]['original'] == 1090 and unions[0]['previous_personal_ranges'] == [[1, 680]] and unions[0]['new_personal_ranges'] == [[681, 1700]]
assert unions[1]['original'] == 1090 and unions[1]['previous_personal_ranges'] == [[1, 1700]] and unions[1]['new_personal_ranges'] == [[1701, 2121]]
for item in doc['later_hash_matched_reconstructions']:
    raw = (root / item['path']).read_bytes()
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert item['revision'] == 'ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04'
assert len(doc['later_hash_matched_reconstructions']) == 4
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']['lens13']
index = int(doc['historical_records'][0]['pointer'].rsplit('/', 1)[1])
prior = ledger['delegated_connected_contracts_pending_alias'][index]
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['proposed_id'] == 'PL13-242-CORPSE-FAMILY-BINDING-POLICY-PENDING'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in baseline['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_authority_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][2289]
assert queue['contract_id'] == 'PL13-242-CORPSE-FAMILY-BINDING-POLICY-PENDING' and name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected corpse-family source, seven old receipts and four historical reconstructions; no product ran')
