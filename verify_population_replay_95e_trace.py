"""Verify population replay source identities and earlier receipts, without product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L07-POPULATION-REPLAY-95E.json'
doc = load(name)
assert doc['source_revision'] == rev and doc['lens'] == 7
assert [r['id'] for r in doc['candidates']] == [
    'REVIEW-POPULATION-RESEED-HOUSEHOLD-001',
    'SEGOVB-L12-RESIDENT-PROFESSION-TEXT-AUTHORITY',
]
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == rev and all(1 <= lo <= hi <= row['lines'] for lo, hi in row['ranges'])
    contents[row['path']] = raw
pop = contents['crates/adventuresim-stdb-module/src/settlement_population.rs']
ensure = pop.split(b'pub fn ensure_settlement_population(', 1)[1].split(b'pub fn npc_is_present', 1)[0]
assert b'settlement_resident_drafts(ctx, settlement_id)?' in ensure
assert b'insert_resident_draft(ctx, settlement_id, draft)?' in ensure
assert b'ensure_seeded_family_households(ctx, settlement_id, &household_groups)?' in ensure
assert b'bind_generated_households(ctx, settlement_id)' in ensure
family = contents['crates/adventuresim-stdb-module/src/relationship/family.rs']
join = family.split(b'fn join_household(', 1)[1].split(b'pub fn record_character_birth', 1)[0]
assert b'member.household_id == household_id' in join
assert b'leave_household(ctx, character_id)' in join and b'joined_minute: minute' in join
seed = family.split(b'pub fn ensure_seeded_family_households(', 1)[1].split(b'fn father_of_at', 1)[0]
assert b'join_household(ctx, &plan.household_id, character_id, StrategicMinute::ZERO, role)' in seed
model = contents['crates/adventuresim-stdb-module/src/relationship/model.rs']
assert b'#[unique]\n    pub character_id: u64' in model and b'pub struct HouseholdMember' in model
marriage = contents['crates/adventuresim-stdb-module/src/relationship/marriage.rs']
assert b'let household_id = format!("household:{}", commitment.id)' in marriage
assert b'join_household(' in marriage and b'commitment.effective_minute' in marriage
assert b'member.household_id == marriage.household_id' in marriage
material = contents['crates/adventuresim-stdb-module/src/residence/properties/materialization.rs']
assert b'.generated_resident_home()' in material and b'for (_, members) in groups' in material
assert b'home.unmaterialized_residents >= required' in material
assert b'available.sort_by(|left, right| left.property_id.cmp(&right.property_id))' in material
assert b'home.unmaterialized_residents -= required' in material
households = contents['crates/adventuresim-stdb-module/src/residence/properties/households.rs']
assert b'let key = (member.household_id.clone(), occupant.property_id.clone())' in households
birth = contents['crates/adventuresim-stdb-module/src/relationship/reproduction.rs']
assert b'marriage_household.or_else' in birth
props = contents['crates/adventuresim-core/src/settlement_property.rs']
assert b'let mut abstract_population = self.population - known as u32' in props
assert b'home.resident_capacity' in props and b'capacity < u64::from(population)' in props
entry = contents['crates/strategic-web/src/routes/settlements/entry.rs']
assert b'tokio::spawn(async move' in entry and b'"ensure_settlement_activity"' in entry
boot = contents['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
assert b'pub fn ensure_settlement_activity(' in boot
assert b'crate::settlement_population::ensure_settlement_population(ctx, settlement_id)' in boot
request = contents['crates/adventuresim-stdb-module/src/tactical/request.rs']
assert b'crate::settlement_population::ensure_settlement_population(ctx, origin_settlement_id)?' in request
persistence = contents['crates/adventuresim-stdb-module/src/settlement_population/resident_persistence.rs']
assert b'const CLERIC: &str = "cleric"' in persistence and b'profession == CLERIC' in persistence
assert b'&existing.profession' in persistence and b'ensure_character_professional_role' in persistence
social = contents['crates/adventuresim-stdb-module/src/social_roles.rs']
assert b'definition.role(role_id).is_none()' in social and b'Character already has a different role' in social
assert b'pub fn character_social_precedence_view' in social
corepop = contents['crates/adventuresim-core/src/settlement_population.rs']
profession = corepop.split(b'pub enum Profession {', 1)[1].split(b'}', 1)[0]
assert b'Cleric' not in profession and b'ServiceProvider' in profession

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']; assert len(records) == 19
by_path = {}
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    original = transport[row['original_path']]
    assert row['sha256'] == original['sha256'] and row['git_blob'] == original['git_blob']
    assert row['bytes'] == original['bytes'] and sha(raw) == row['published_sha256']
    assert len(raw) == row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    by_path[row['original_path']] = raw
for number, indices in ((277,[1188]), (278,[1189,1190]), (279,[1191,1192,1193,1194])):
    receipt_name = f'segovb/READ-L12-{number}.json'; receipt = json.loads(by_path[receipt_name])
    assert receipt['lens'] == 12 and [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert receipt['plan'] in by_path and sha(by_path[receipt['plan']]) == receipt['plan_sha256']
    for row in receipt['rows']:
        assert row['registration'] in by_path and sha(by_path[row['registration']]) == row['registration_sha256']
        assert row['fresh_personal_body_reads'] == row['assessed_ranges']
        assert row['other_lens_verdict_transfer'] is False
    check = json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256'] == sha(by_path[receipt_name])
    assert check['state_sha256'] == sha(by_path[f'segovb/STATE-L12-{number}.json'])
histname = 'ASSESSMENT-L01-HISTORICAL-1188-1194.json'
history = load(histname)['historical_source_roles']
assert [r['original_index'] for r in history] == list(range(1188, 1195))
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in history:
        raw = archive.extractfile(row['sha256']).read()
        assert sha(raw) == row['sha256'] and len(raw.splitlines()) == row['lines']
        assert row['new_personal_read_ranges'] == [[1, row['lines']]]
        assert row['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[row['source_id'].split(':', 1)[1]]
        role = roles[row['original_index']]
        assert role['historical_assessment'] == histname and role['status'] == 'full-historical-body-trace-open'
q = load('RECONCILIATION-059.json')['entries']
for index, contract in [(226,'SEGOVB-L12-RESIDENT-PROFESSION-TEXT-AUTHORITY'),
                        (967,'Q-L12-0f6ddb4c5227797c70d3'),
                        (968,'Q-L12-5b1eaf15aa0aa2800b18'),
                        (969,'Q-L12-6ae40abc7dba7eb5d901'),
                        (2235,'PL13-202-HOUSEHOLD-FIRST-FIT-POLICY-PENDING')]:
    assert q[index]['contract_id'] == contract and q[index]['current_reconciliation'].startswith('partial ')
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for candidate in doc['candidates']:
    matches = [r for r in ledger if r['id'] == candidate['id']]
    assert len(matches) == 1 and matches[0]['record'] == name and matches[0]['lens'] == candidate['lens']
progress = load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_authority_traces']
assert progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert progress['new_current_ledger_candidates'] == len(ledger)
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified population replay, census limits and historical attributions; no product ran')
