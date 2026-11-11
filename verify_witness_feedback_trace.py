"""Verify preserved witness feedback evidence without executing product code."""
import gzip
import hashlib
import json
from decimal import Decimal
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-WITNESS-FAILED-FEEDBACK-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 4 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
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
assert len(sources) == 15
witness = sources['crates/adventuresim-core/src/social/witness.rs']
must(witness, ['Truthful claims always use the same',
               'let succeeded = !input.claim_is_factually_accurate && outcome.succeeded;',
               'outcome.morale_delta.min(0.0)', 'input.approach.failure_affinity_loss()'])
core = sources['crates/adventuresim-core/src/social.rs']
must(core, ['.clamp(0.05, 0.95)', 'attempt.roll.clamp(0.0, 1.0) < chance',
            '1.0 + 5.0 * risk', '-(0.5 + 5.5 * risk)',
            'Self {\n            risk: action.risk(),\n            effectiveness: 1.0'])
action = sources['crates/adventuresim-core/src/social/action_kind.rs']
must(action, ['Self::LightenMood => 0.45'])
resolution = sources['crates/adventuresim-core/src/social/claim_resolution.rs']
must(resolution, ['WitnessClaimOutcome::UsefulAnswer', 'WitnessClaimOutcome::DidNotYield',
                  'affinity_delta: realized_affinity_delta'])
native = sources['crates/adventuresim-stdb-module/src/social.rs']
must(native, ['claim_is_factually_accurate: claim.claim_is_factually_accurate',
              'let receipt_id = format!("{session_id}:{action_id}")',
              'if morale_delta != 0.0', 'outcome.morale_delta,',
              '&format!("npc-morale-approach:{receipt_id}")',
              'claim.resolution = Some(WitnessClaimResolution::from_challenge('])
condition = sources['crates/adventuresim-stdb-module/src/condition.rs']
must(condition, ['#[table(accessor = morale_event, public)]', 'pub source_id: Option<String>',
                 'if magnitude == 0.0 || !magnitude.is_finite()', 'ctx.db.morale_event().insert(MoraleEvent {'])
claim_projection = sources['crates/strategic-web/src/routes/dialogue/claims.rs']
must(claim_projection, ['resolution: Option<adventuresim_core::social::WitnessClaimResolution>',
                        'affinity_delta: resolution.affinity_delta'])
assert 'claim_is_factually_accurate' not in claim_projection.split('#[derive(Deserialize)]', 1)[0]
must(sources['crates/adventuresim-stdb-module/src/social/draws.rs'], ['StreamId::new("social.claim-challenge")', '.inclusive_unit_f32()'])
must(sources['crates/fabelgeist-determinism/src/lib.rs'], ['pub fn inclusive_unit_f32(value: u64) -> f32', 'INCLUSIVE_UNIT_F32_SCALE'])
must(sources['crates/fabelgeist-determinism/src/stream.rs'], ['crate::inclusive_unit_f32(self.next_u64())'])
must(sources['crates/adventuresim-core/src/quest_generation/model.rs'], ['Reliability::Truthful => TestimonyClaimAuthority', 'factually_accurate: true', 'factually_accurate: false'])
must(sources['crates/adventuresim-stdb-module/src/strategic/dialogue_bindings.rs'], ['claim_is_factually_accurate: authority.factually_accurate'])
must(sources['crates/strategic-web/src/live.rs'], ['.add_query(|query| query.from.morale_event())'])
must(sources['crates/adventuresim-stdb-client/src/morale_event_type.rs'], ['pub source_id: Option<String>', 'pub magnitude: f32'])
must(sources['crates/adventuresim-stdb-client/src/morale_event_table.rs'], ['pub trait MoraleEventTableAccess', 'fn iter(&self)'])
# Symbolic branch bounds, not a random draw, fixture or product execution.
assert Decimal('0') < Decimal('0.05') <= Decimal('0.95') < Decimal('1')
assert -(Decimal('0.5') + Decimal('5.5') * Decimal('0.45')) == Decimal('-2.975')
assert Decimal('1') + Decimal('5') * Decimal('0.45') > 0
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 14
old13 = support['segovb/READ-L13-PARALLEL-203.json'][1]
assert old13['lens'] == 13 and old13['accepted_unique_count_change'] == 0 and old13['current_source_reconciled'] is False
assert any('truthfulalwaysfailed andpositiveoutcomemorale clipped' in x for x in old13['evidence']['numeric_role_inventory'])
prefix = 'target/726-audit-adler-20261027/'
for ref in (old13['delegated_receipt'], old13['delegated_verification'], old13['agent_validation_manifest'], old13['evidence']['registration']):
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
old12 = support['segovb/READ-L12-280.json'][1]
assert old12['lens'] == 12 and old12['accepted_unique_count_change'] == 0 and old12['current_source_reconciled'] is False
assert support[old12['plan']][0]['sha256'] == old12['plan_sha256']
row = old12['rows'][0]
assert support[row['registration']][0]['sha256'] == row['registration_sha256']
assert row['source_sha256'] == hashlib.sha256(sources['crates/adventuresim-stdb-module/src/social.rs'].encode()).hexdigest()
cursor = 1
for part in row['function_contract']['read_attribution']['parts']:
    assert support[part['path']][0]['sha256'] == part['sha256']
    for lo, hi in part['fresh']:
        assert lo == cursor and hi >= lo
        cursor = hi + 1
assert cursor == 3801
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
for lens, index, path in [(13, 202, 'segovb/READ-L13-PARALLEL-203.json'), (12, 45, 'segovb/READ-L12-280.json')]:
    prior = ledger['segovb_continuation']['lens' + str(lens)]['discovery_receipts'][index]
    assert prior['path'] == path and prior['sha256'] == support[path][0]['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_operation_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified witness source, symbolic branch reasoning and 14 historical records; no product ran')
