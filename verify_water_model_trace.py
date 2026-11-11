"""Verify isolated water model evidence without invoking product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-WATER-MODEL-ATOMICITY-95E.json'
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
assert len(sources) == 2
owner = sources['crates/adventuresim-core/src/water_source.rs']
draw = owner.split('    pub fn draw(', 1)[1].split('    pub fn transfer_all', 1)[0]
must(draw, ['conserved_collection(self.source_ml, self.holding_ml, amount_ml)',
            'self.source_ml = source_after;', 'self.holding_ml = holding_after;',
            '.checked_add(load_microunits)', '.ok_or(OutbreakWaterFlowError::InsufficientWater)?',
            'self.receipts.insert(request_id.into(), receipt)'])
assert draw.index('self.source_ml = source_after;') < draw.index('.checked_add(load_microunits)')
assert draw.index('self.holding_ml = holding_after;') < draw.index('.checked_add(load_microunits)')
assert draw.index('.checked_add(load_microunits)') < draw.index('self.receipts.insert(request_id.into(), receipt)')
must(owner, ['self.holding_load_microunits * kill_numerator / kill_denominator',
             'let child_ml = self.food_ml * numerator / denominator;',
             'if denominator == 0 || numerator > denominator'])
report = sources['crates/adventuresim-strategic-sim/src/investigation_eval/report.rs']
must(report, ['#[cfg(test)]\nmod tests {', 'OutbreakWaterFlow::new(500_000)',
              'source.draw("draw-1", 1_000, 12_000_000)', 'pot.cook_all(1, 10_000)',
              'pot.split_food(1, 2)'])
assert report.index('#[cfg(test)]\nmod tests {') < report.index('OutbreakWaterFlow::new(500_000)')
# Symbolic capacity comparisons only; the model is never constructed or run.
maximum = 2**64 - 1
assert maximum + 0 == maximum and maximum + 1 > maximum
assert 2 - 1 - 1 == 0 and 0 + 1 + 1 == 2
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
paths = set()
for line in raw.decode().splitlines():
    assert line.startswith(revision + ':')
    paths.add(line[len(revision)+1:].split(':', 1)[0])
assert paths == {'crates/adventuresim-core/src/water_source.rs',
                 'crates/adventuresim-strategic-sim/src/investigation_eval/report.rs'}
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 6
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('draw mutatessource/holding beforecheckedloadaddcanErr' in x for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
assert old['evidence']['partial_read_union'][0]['previous_personal_ranges'] == [[1, 460]]
assert old['evidence']['partial_read_union'][0]['new_personal_ranges'] == [[461, 500]]
part = support['parallel-l13-20261029/READ-PL13-215-PARTIAL-001.json'][1]
assert part['rows'][0]['fresh_personal_assessed_ranges'] == [[1, 460]]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][307]
assert prior['receipt'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['accepted_unique_count_change'] == 0 and prior['contract']['current_reconciled'] is False
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
print('verified isolated water model source and 6 historical records; no product ran')
