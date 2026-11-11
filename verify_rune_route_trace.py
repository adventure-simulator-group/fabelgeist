"""Verify selected rune authority evidence without running puzzle code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L05-RUNE-ROUTE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 5 and doc['source_revision'] == revision

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
assert len(sources) == 4
puzzle = sources['crates/adventuresim-puzzles/src/puzzles.rs']
must(puzzle, ['pub const RUNE_TRANSFORMATION_RULES_VERSION: u16 = 4;',
              'pub struct RuneTransformationSpec {', 'pub allow_repeated_route_gates: bool',
              'if !self.allow_repeated_route_gates && self.route_length > self.gate_count',
              'pub struct RuneTransformationPuzzle {', 'pub route: Vec<RuneGate>',
              'let mut shuffled = active_gates.clone();',
              'route.extend(shuffled.into_iter().take(usize::from(spec.route_length)))',
              'Self::ExchangeCrownHart => (Sigil::Crown, Sigil::Hart)',
              'Self::ExchangeHartMoon => (Sigil::Hart, Sigil::Moon)'])
validator = puzzle.split('impl RuneTransformationPuzzle {', 1)[1].split('    pub fn validate(&self)', 1)[1].split('\npub fn rune_operations_consistent_with', 1)[0]
must(validator, ['self.spec.validate()?', 'self.route.len() != usize::from(self.spec.route_length)',
                 'Some(self.solution) != apply_rune_route',
                 'rune_operations_consistent_with(&gate_examples) != vec![law.operation]'])
assert 'allow_repeated_route_gates' not in validator
assert 'allow_repeated_operations' not in validator
native = sources['crates/adventuresim-stdb-module/src/strategic/challenges.rs']
view = native.split('pub fn backend_challenges(', 1)[1].split('pub fn ', 1)[0]
must(view, ['serde_json::from_str(&challenge.puzzle_json)', 'puzzle.validate().ok()?',
            'serde_json::to_string(&puzzle.projection())'])
assert 'puzzle.replay()' not in view
must(native, ['.validate()\n        .map_err(|_| "Challenge authority failed deterministic replay")?',
              'let replay = puzzle.replay().map_err(str::to_string)?;',
              'if replay != puzzle {'])
cli = sources['crates/adventuresim-puzzles/src/bin/puzzle-lab.rs']
must(cli, ['Command::Replay { path }', 'puzzle.validate().map_err(str::to_owned)?',
           'if puzzle.replay().map_err(str::to_owned)? != puzzle'])
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 11
old11 = support['segovb/READ-L11-460.json'][1]
assert old11['lens'] == 11 and old11['accepted_unique_count_change'] == 0 and old11['current_reconciled'] is False
assert old11['sources'][0]['assessed_ranges'] == [[1, 944]]
assert any('allow_repeated_route_gatesfalse' in x for x in old11['role_dispositions'])
for ref in old11['sources']:
    path = ref['registration'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['registration_sha256']
old13 = support['segovb/READ-L13-PARALLEL-229.json'][1]
assert old13['lens'] == 13 and old13['accepted_unique_count_change'] == 0 and old13['current_source_reconciled'] is False
for ref in (old13['delegated_receipt'], old13['delegated_verification'], old13['agent_validation_manifest'],
            old13['evidence']['registration'], *old13['evidence']['partial_checkpoints']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
for suffix, expected in [('001', [[1, 460]]), ('002', [[1, 920]])]:
    part = support['parallel-l13-20261029/READ-PL13-231-PARTIAL-' + suffix + '.json'][1]
    assert part['rows'][0]['fresh_personal_assessed_ranges'] == expected
unions = old13['evidence']['partial_read_union']
assert unions[0]['new_personal_ranges'] == [[461, 920]] and unions[1]['new_personal_ranges'] == [[921, 1131]]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']
assert ledger['discovery_receipts'][65]['sha256'] == support['segovb/READ-L11-460.json'][0]['sha256']
prior = ledger['lens13']['delegated_connected_contracts_pending_alias'][313]
assert prior['receipt_sha256'] == support['segovb/READ-L13-PARALLEL-229.json'][0]['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['status'] == ref['role_status'] and role.get('historical_assessment') == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_aggregate_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][2221]
assert queue['contract_id'] == 'PL13-231-RUNE-ADMISSION-PENDING' and name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected rune source and 11 historical records; no product ran')
