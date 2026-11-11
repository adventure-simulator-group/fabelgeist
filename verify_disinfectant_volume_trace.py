"""Verify selected disinfectant evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-DISINFECTANT-VOLUME-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 4 and doc['source_revision'] == revision

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
assert len(sources) == 8
measure = sources['crates/adventuresim-core/src/inventory_measurement.rs']
must(measure, ['pub const MICROS_PER_WHOLE: u32 = 1_000_000;',
               'pub fn try_from_ratio(numerator: u64, denominator: u64)',
               'if denominator == 0', '.div_ceil(u128::from(denominator))',
               'fraction_micros.min(u128::from(Self::MICROS_PER_WHOLE))'])
# Fixed-point arithmetic derived from the source; no product helper is invoked.
requested = min((1_000_000 * 25 + 10 - 1) // 10, 1_000_000)
assert requested == 1_000_000 and 10 * requested // 1_000_000 == 10
alcohol = sources['crates/adventuresim-stdb-module/src/alcohol.rs']
must(alcohol, ['const SURGERY_DISINFECTANT_ML: u64 = 25;',
               'pub fn best_disinfectant(', 'pub fn disinfectant_count(',
               'pub fn consume_inventory_row(', 'u64::from(def.alcohol_serving_ml)',
               'u64::from(definition.alcohol_serving_ml)',
               '.unwrap_or(ConsumableFractionMicros::WHOLE)',
               'def.alcohol_disinfectant_effectiveness > 0',
               'consume_stack(ctx, Stack::Personal(row), requested)?'])
surgery = sources['crates/adventuresim-stdb-module/src/surgery.rs']
must(surgery, ['crate::alcohol::best_disinfectant(ctx, actor_id)',
               'crate::alcohol::consume_inventory_row(ctx, *inventory_id)?',
               'adventuresim_core::alcohol::surgery_control_bonus('])
assert surgery.index('crate::alcohol::consume_inventory_row(ctx, *inventory_id)?') < surgery.index('adventuresim_core::alcohol::surgery_control_bonus(')
amount = sources['crates/adventuresim-stdb-module/src/inventory_amount.rs']
must(amount, ['Row quantity stays', 'let consumed = requested_fraction.min(available)',
              'if remaining.is_zero()'])
validation = sources['crates/adventuresim-core/src/item_catalog_validation.rs']
must(validation, ['if serving == 0', 'serving > 10_000', 'hydration > serving',
                  '.get("disinfectant_effectiveness")'])
core = sources['crates/adventuresim-core/src/alcohol.rs']
must(core, ['pub const fn surgery_control_bonus', 'effectiveness as f32 / ALCOHOL_SURGERY_CONTROL_DIVISOR'])
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
lines = raw.decode().splitlines()
assert len(lines) == 6 and all(line.startswith(revision + ':content/items/catalog.yaml:') for line in lines)
for serving in (100, 500, 250):
    assert any('"serving_ml": ' + str(serving) in line for line in lines)
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
assert any('best_disinfectant ratio25/serving usesWHOLEfallback onErr' in x
           for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
for suffix, role in [('001', 1073), ('002', 1074)]:
    part = support['parallel-l13-20261029/READ-PL13-235-PARTIAL-' + suffix + '.json'][1]
    assert part['rows'][0]['original'] == role and part['rows'][0]['fresh_personal_assessed_ranges'] == [[1, 460]]
unions = old['evidence']['partial_read_union']
assert unions[0]['original'] == 1073 and unions[0]['new_personal_ranges'] == [[461, 677]]
assert unions[1]['original'] == 1074 and unions[1]['new_personal_ranges'] == [[461, 567]]
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']['lens13']
prior = ledger['delegated_connected_contracts_pending_alias'][314]
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
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
assert name in progress['post_transition_bounded_operations_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][2223]
assert queue['contract_id'] == 'PL13-235-DISINFECTANT-ADMISSION-PENDING' and name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected disinfectant source and 7 historical records; no product ran')
