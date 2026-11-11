"""Verify selected nested-vessel evidence without running product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L08-NESTED-VESSEL-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 8 and doc['source_revision'] == revision

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
assert len(sources) == 7
module = 'crates/adventuresim-stdb-module/src/'
cooking = sources[module + 'food/cooking.rs']
section = cooking.split('/// Starts the independent dish lane', 1)[1].split('fn add_fireplace_ingredients_at', 1)[0]
must(section, ['contained cookable food lot at any nesting depth is consumed in full',
               'Cooking intentionally sees only direct contents.',
               '.parent_object_id()', '.filter(container_object_id)',
               'let (Some(lot), Some(amount)) = (lot, amount) else',
               'Put at least one uncooked food lot in the container'])
assert 'subtree_object_ids' not in section
fixtures = sources[module + 'food/tests/fireplace_and_cooking.rs']
must(fixtures, ['fn vessel_selection_uses_direct_authoritative_food_lots()',
                'assert!(!reducer.contains("subtree_object_ids"));',
                'fn vessel_selection_is_direct_and_preparation_shortens_safety_time()'])
route = sources['crates/strategic-web/src/routes/settlements/party/cooking.rs']
must(route, ['Self::StartCooking => "start_fireplace_container_cooking"',
             'pub(super) async fn settlement_fireplace_container_start(',
             'pub(super) async fn camp_fireplace_container_start(',
             'vec![json!(form.container_object_id)]'])
template = sources['crates/strategic-web/src/templates/settlement/cooking.rs']
must(template, ['"Start cooking contents"', 'name="container_object_id"'])
container = sources[module + 'inventory_container.rs']
must(container, ['pub fn put_inventory_item_in_container(',
                 'parent.location.has_same_carried_custody(&child.location)',
                 'model\n        .insert(', 'Container capacity exceeded:',
                 'child_object_id: child.id,', 'parent_object_id: parent.id,'])
placement = sources[module + 'food/fireplace_custody.rs']
must(placement, ['Places one exact vessel and its entire subtree',
                 'Children retain their stable object edges.',
                 'pub fn place_fireplace_container(',
                 'if crate::inventory_container::object_is_nested(ctx, object.id)'])
comparison = doc['historical_function_comparison']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    old = archive.extractfile(comparison['historical_sha256']).read()
current = cooking.encode()
assert hashlib.sha256(old).hexdigest() == comparison['historical_sha256']
assert hashlib.sha256(current).hexdigest() == comparison['current_sha256']
assert old != current and comparison['whole_body_identical'] is False
def selected(body):
    return body.split(b'/// Starts the independent dish lane', 1)[1].split(b'fn add_fireplace_ingredients_at', 1)[0]
assert selected(old) == selected(current) and comparison['selected_section_identical'] is True
assert hashlib.sha256(selected(current)).hexdigest() == comparison['selected_section_sha256']
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
assert len(support) == 13
for filename in ('segovb/READ-L12-238.json', 'segovb/READ-L12-244.json'):
    record = support[filename][1]
    assert record['lens'] == 12 and record['accepted_unique_count_change'] == 0 and record['current_source_reconciled'] is False
    assert support[record['plan']][0]['sha256'] == record['plan_sha256']
    for row in record['rows']:
        assert support[row['registration']][0]['sha256'] == row['registration_sha256']
old = support['segovb/READ-L13-PARALLEL-245.json'][1]
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('Container doc says any nesting depth' in entry for entry in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'], old['evidence']['registration']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']
assert 'Nested vessel ingredient doc' in ledger['lens12']['pending_connected_contracts'][8]['contract']
for index, filename in [(3, 'segovb/READ-L12-238.json'), (9, 'segovb/READ-L12-244.json')]:
    assert ledger['lens12']['discovery_receipts'][index]['sha256'] == support[filename][0]['sha256']
assert ledger['lens13']['discovery_receipts'][244]['sha256'] == support['segovb/READ-L13-PARALLEL-245.json'][0]['sha256']
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
assert name in progress['post_transition_bounded_behavior_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][898]
assert queue['contract_id'] == 'Q-L12-f82afe3f9c507e185085' and name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected vessel sources, section comparison and 13 old records; no product ran')
