"""Verify local-problem source evidence and arithmetic, without executing product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-LOCAL-PROBLEM-CAPS-95E.json'
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
assert len(sources) == 12
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
local = sources[core + 'local_problem.rs']
must(local, ['pub const MAX_ACTIVE_PER_SCOPE: usize = 3;',
 'pub const MAX_TRADE_BPS: i32 = 2_500;',
 'pub const INCIDENT_SEVERITY_STEP_BPS: u32 = 2_500;',
 'Cause::Bandits => Effects {', 'buy_bps: 1_200,',
 'Cause::Smugglers => Effects {', 'buy_bps: -200,',
 'pub fn aggregate_consequences', 'rows.sort_by(|a, b| a.id.cmp(&b.id));',
 'rows.truncate(MAX_ACTIVE_PER_SCOPE);', 'scale_i32_for_incidents(r.buy_bps, r.incident_count)',
 '.clamp(i64::from(-MAX_TRADE_BPS), i64::from(MAX_TRADE_BPS)) as i32',
 'pub fn adjust_price(base: u32, basis_points: i32)'])
must(sources['crates/adventuresim-world-schema/src/basis_points.rs'],
     ['pub const BASIS_POINTS_PER_WHOLE: u16 = 10_000;'])
authority = sources[module + 'local_problem.rs']
must(authority, ['pub fn backend_local_problem_trade_effects(',
 'pub(crate) fn materialize_generated_problem(', 'buy_bps: consequence.effects.buy_bps,',
 'pub fn ensure_settlement_problems(', 'if ctx', 'let (problem, explanation) = lp::generate(&context, 0, minute)?;',
 'buy_bps: problem.effects.buy_bps,', 'problem.incident_count = due;',
 'pub fn settlement_effects(', 'lp::aggregate_consequences(rows.iter(), minute)'])
solver = sources[core + 'quest_generation/solver.rs']
must(solver, ['fn consequence(', 'buy_bps: authored.buy_bps,'])
generation = json.loads(sources['content/quests/generation.yaml'])
profiles = {x['id']: x for x in generation['consequences']}
assert profiles['recurring_bandits']['buy_bps'] == 1200
assert profiles['loss_default']['buy_bps'] == 900
must(sources[core + 'quest_generation/assembly.rs'], ['let prefix = observer_scope(context);',
 'let problem_id = scoped_id(&prefix, "problem", "settlement");'])
must(sources[core + 'quest_generation/model.rs'], ['fn scoped_id(scope: &str, kind: &str, name: &str)',
 'format!("{kind}:{}",'])
must(sources[module + 'strategic/mission_bootstrap.rs'], ['materialize_generated_problem(ctx, generated, settlement_id)?'])
must(sources[module + 'strategic/inventory_trade.rs'], ['crate::local_problem::settlement_effects(ctx, &settlement_id, problem_minute)',
 'local_problem::adjust_price(quoted, problem_effects.buy_bps)'])
must(sources[module + 'disease.rs'], ['crate::local_problem::settlement_effects(ctx, &settlement_id, minute)',
 'local_problem::adjust_price(base, problem_effects.buy_bps)'])
must(sources['crates/strategic-web/src/routes/settlements/rendering.rs'],
     ['backend_local_problem_trade_effects WHERE character_id', 'problem_effects.buy_bps,'])
must(sources['crates/strategic-web/src/templates/settlement/trade/merchant.rs'],
     ['local_problem::adjust_price', 'problem_buy_bps'])

# Evidence-only integer arithmetic, with three active unmitigated hypothetical rows.
def fold(values):
    result = 0
    for value in values:
        result = min(2500, max(-2500, result + value))
    return result
assert 1200 * (10000 + 4 * 2500) // 10000 == 2400
assert fold([2400, 900, -200]) == 2300
assert fold([-200, 2400, 900]) == 2500
assert min(2500, 2400 + 900 - 200) == 2500
assert (100 * (10000 + 2300) + 9999) // 10000 == 123
assert (100 * (10000 + 2500) + 9999) // 10000 == 125

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
item = doc['recovered_historical_receipts'][0]
raw = preserved(item)
entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
old = json.loads(raw)
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][284]
assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
assert prior['contract']['proposed_id'] == 'PL13-178-LOCAL-PROBLEM-NATIVE-POLICY-PENDING'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
ref = doc['historical_source_roles'][0]
role = roles[ref['original_index']]
assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_operations_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 12 Git-bound sources, symbolic cap arithmetic and prior receipt; no product ran')
