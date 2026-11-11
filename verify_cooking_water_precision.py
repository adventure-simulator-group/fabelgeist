"""Verify frozen source, prior receipts and binary32 arithmetic without product code."""
import gzip
import hashlib
import json
import struct
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-COOKING-WATER-PRECISION-95E.json'
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
assert len(sources) == 8
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
must(sources[core + 'material.rs'], [
    'pub const MICROLITERS_PER_MILLILITER: u64 = 1_000;',
    'pub fn try_from_milliliters(milliliters: Milliliters)',
    '.checked_mul(Self::MICROLITERS_PER_MILLILITER)',
    'pub fn try_from_nonnegative_milliliters_rounded(milliliters: f32)',
    '(milliliters * Self::MICROLITERS_PER_MILLILITER as f32).round()'])
must(sources[core + 'water_source.rs'], [
    'pub fn proportional_material_transfer(', 'if moved == public_total {',
    'return Some((contribution, contaminant_load_microunits));'])
cooking = sources[module + 'food/cooking.rs']
must(cooking, [
    '.map_or(0, |liquid| liquid.water_ml);',
    'let water_ml = contained_water_ml as f32;',
    'Microliters::try_from_nonnegative_milliliters_rounded(water_ml)',
    'if contributed_water_microliters > public_water_microliters {',
    'Contained water material exceeds its public volume',
    '.delete(instrument_object_id);'])
assert cooking.index('if contributed_water_microliters > public_water_microliters') < cooking.index('.delete(instrument_object_id);')
must(sources[module + 'food/fireplace_custody.rs'], [
    'Some("cooking_pan") => Ok(CookingMethod::PanFry)',
    'Some("cooking_pot") => Ok(CookingMethod::Stew)',
    'Some("portable_oven") => Ok(CookingMethod::Bake)'])
outbreak = sources[module + 'outbreak.rs']
must(outbreak, [
    'pub fn collect_fixture_water_into_container(',
    'require_container_capacity(ctx, object.id, requested_ml)',
    'let requested_microliters = Microliters::try_from_milliliters(Milliliters::new(requested_ml))',
    'amount_microliters: requested_microliters.get(),',
    'pub(crate) fn contained_water_contamination(',
    'Microliters::try_from_milliliters(Milliliters::new(liquid.water_ml))',
    'moved.min(source_total),', 'available_ml: 500_000,'])
must(sources[module + 'inventory_container.rs'], [
    'pub struct ContainerLiquid {', 'pub water_ml: u64,',
    'pub(crate) fn require_container_capacity(',
    'next > Milliliters::new(u64::from(definition.container_capacity_ml))'])
must(sources[core + 'item_catalog_validation.rs'], [
    'if capacity == 0 || capacity > 1_000_000 {', 'expected 1..1000000'])
catalog = json.loads(sources['content/items/catalog.yaml'])
items = {item['id']: item for item in catalog['items']}
capacities = {key: items[key]['capabilities']['container']['capacity_ml']
              for key in ('cooking_pan', 'cooking_pot', 'portable_oven')}
assert capacities == {'cooking_pan': 2500, 'cooking_pot': 8000, 'portable_oven': 12000}
assert 0 < 200000 <= 1000000
# Integers up to 2^24 and integer results up to 2^24 are exactly binary32.
assert max(capacities.values()) < 2**24
assert max(capacities.values()) * 1000 < 2**24
f32 = lambda n: struct.unpack('>f', struct.pack('>f', n))[0]
assert int(f32(134221)) == 134221
assert int(f32(f32(134221) * f32(1000))) == 134220992
assert 134221 * 1000 == 134221000 > 134220992
assert int(f32(f32(134219) * f32(1000))) == 134219008
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior_list = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
for item, index, old_id in zip(doc['recovered_historical_receipts'], [334, 346],
                               ['PL13-247-COOKING-MATERIAL-AMOUNT-NATIVE-PENDING',
                                'PL13-256-CONTAINED-WATER-VOLUME-AND-PROVENANCE-PENDING']):
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    old = json.loads(raw)
    assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
    prior = prior_list[index]
    assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
    assert prior['contract']['proposed_id'] == old_id
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
print('verified 8 Git-bound sources, binary32 arithmetic and 2 original receipts; no product ran')
