"""Verify selected point-prevention evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L09-POINT-PREVENTION-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 9 and doc['source_revision'] == revision

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
assert len(sources) == 9
module = 'crates/adventuresim-stdb-module/src/'
core = 'crates/adventuresim-core/src/'
disease = sources[module + 'disease.rs']
must(disease, ['pub(crate) fn protected_point_exposure(',
               'let historical = try_party_physiology_check_at(ctx, character_id, minute)?;',
               'let check = if historical > 0.0',
               'crate::capability::evaluate_character(ctx, character_id)?.physiology',
               'Party members still use pinned presence',
               'disease::historical_physiology_check_at(coverage, minute)'])
shared = sources[core + 'disease.rs']
must(shared, ['TransmissionVector::FoodWater => 0.94',
              'let competence = (physiology_check.clamp(0.0, 5.0) / 5.0).powf(1.25);',
              'pub fn historical_physiology_check_at(',
              'if minute < start || minute > end',
              'crate::capability::aggregate_bounded_party_check(contributors.into_values())'])
capability = sources[core + 'capability.rs']
must(capability, ['pub fn aggregate_bounded_party_check(',
                  '.filter(|value| value.is_finite() && *value > 0.0)',
                  'physiology: skills', 'Skill::Physiology,'])
social = sources[module + 'social.rs']
must(social, ['pub fn reset_familiarity_after_join(',
              'capability.physiology.round().clamp(0.0, 5.0) as u8',
              'low_observer_band: band(low_id)', 'high_observer_band: band(high_id)',
              'pub(crate) fn begin_physiology_presence_on_contact('])
governance = sources[module + 'strategic/governance.rs']
must(governance, ['crate::social::reset_familiarity_after_join(ctx, member_id);'])
skill = sources[core + 'skill.rs']
must(skill, ['#[assoc(max_hours = 10000.0, kind = SkillKind::Mental, is_trained = true)]\n    Physiology,',
             'MAX_CHECK * (hours / (hours + self.max_hours() * 0.5))',
             'self.training_rank(hours).min(aptitude)',
             'check *= body.body_part_health(BodyPart::Head).clamp(0.0, 1.0);'])
native_capability = sources[module + 'capability.rs']
must(native_capability, ['pub fn evaluate_character(', 'Skill::Physiology => self.physiology_hours',
                         'crate::social::close_physiology_presence(ctx, character_id);',
                         'crate::social::reset_familiarity_after_join(ctx, character_id);'])
food = sources[module + 'food/consumption.rs']
must(food, ['pub(crate) fn expose_food_water_dysentery(',
            'crate::disease::protected_point_exposure(', 'let minute = current_minute(ctx, character_id);',
            'pub fn eat_food('])
water = sources[module + 'outbreak.rs']
must(water, ['fn expose_to_water_contributions(', '.character_time()',
             'crate::food::expose_food_water_dysentery('])
# Elementary arithmetic derived from static source, not product execution.
check = 5 * 500 / (500 + 5000)
residual = 1 - .94 * (check / 5) ** 1.25
assert 0 < check < .5 and .953 < residual < .954
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
assert 'food/consumption.rs:65:' in raw.decode() and 'outbreak.rs:280:' in raw.decode()
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 8
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any(x['proposed_id'] == 'PL13-243-POINT-PREVENTION-ZERO-HISTORY-PENDING'
           for x in old['evidence']['connected_contracts'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
unions = old['evidence']['partial_read_union']
assert [(x['previous_personal_ranges'], x['new_personal_ranges']) for x in unions] == [
    ([[1, 680]], [[681, 1360]]), ([[1, 1360]], [[1361, 2040]]), ([[1, 2040]], [[2041, 2950]])]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']['lens13']
index = int(doc['historical_records'][0]['pointer'].rsplit('/', 1)[1])
prior = ledger['delegated_connected_contracts_pending_alias'][index]
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['proposed_id'] == 'PL13-243-POINT-PREVENTION-ZERO-HISTORY-PENDING'
history = json.loads((root / 'CHECKPOINT-INCREMENT-348.json').read_text())
assert 'covered-zero versus missing-history policy' in history['audit_gate_reconciliation']['checkpoint304']
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
assert name in progress['post_transition_bounded_enum_shape_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][2290]
assert queue['contract_id'] == 'PL13-243-POINT-PREVENTION-ZERO-HISTORY-PENDING' and name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected point-prevention source and eight old receipts; no product ran')
