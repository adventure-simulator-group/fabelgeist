"""Verify a bounded travel-water source trace and earlier receipts without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-TRAVEL-WATER-FORECAST-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

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
assert len(sources) == 11
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
web = 'crates/strategic-web/src/'
provision = sources[core + 'provisioning.rs']
must(provision, [
    'pub const STRATEGIC_TRAVEL_WATER_ML_PER_DAY: f32 = 4_000.0;',
    'self.waterskin_count as f32 * self.waterskin_capacity_ml as f32',
    'ordinary_water_days: ordinary_water / water_per_day,',
    '(target_days * water_per_day - water).max(0.0) / self.waterskin_capacity_ml as f32'])
graph = sources[core + 'inventory_containers.rs']
must(graph, ['pub fn used_volume(', 'object.measured_volume.unwrap_or(object.exterior_volume)',
             'let Some(parent_capacity) = parent_object.capacity else',
             'next <= parent_capacity'])
container = sources[module + 'inventory_container.rs']
must(container, ['fn remaining_container_capacity_ml(',
                 '.checked_add(Milliliters::new(liquid_used(ctx, container_object_id)))',
                 'capacity\n        .checked_sub(used)',
                 'pub(crate) fn fill_carried_waterskins(',
                 '.filter(STANDARD_WATERSKIN_ID)',
                 'liquid.liquid_item_id != WATER_ITEM_ID',
                 'pub fn put_inventory_item_in_container(',
                 'pub fn discard_container_water(',
                 'pub(crate) fn consume_contained_water(',
                 '.filter(|liquid| liquid.liquid_item_id == WATER_ITEM_ID)',
                 'if resolved.root == *expected_custody'])
condition = sources[module + 'condition.rs']
must(condition, ['pub fn prepare_party_waterskins(', 'pub fn prepare_character_waterskins(',
                 'crate::inventory_container::fill_carried_waterskins(ctx, &custody)?;',
                 'crate::inventory_container::consume_contained_water('])
travel = sources[module + 'strategic/travel_reducers.rs']
must(travel, ['let departing_settlement = character.current_settlement_id.is_some();',
              'prepare_party_waterskins(ctx, &current_party.id, departing_settlement)?;',
              'prepare_character_waterskins(ctx, traveler_id, departing_settlement)?;'])
camp = sources[web + 'routes/settlements/camp.rs']
must(camp, ['water_reserve_ml += needs.water_balance_ml;',
            'let skins = owned(STANDARD_WATERSKIN_ID);',
            'if departing_settlement {\n            waterskin_count += skins;',
            'super::contained_water_ml_for_custody(&objects, &containment, &liquids, &custody)',
            'waterskin_count += party_skins;',
            'waterskin_capacity_ml: waterskin.water_capacity_ml,'])
must(sources[web + 'routes/settlements/overview.rs'],
     ['destination.provision_forecast = travel_provision_forecast(',
      '&living_party_members,\n                destination,\n                true,'])
must(sources[web + 'templates/settlement/travel.rs'],
     ['data-water-days=(forecast.water_days)', 'data-initial-waterskins=(forecast.waterskins_to_buy)'])
sim = sources['crates/adventuresim-strategic-sim/src/live_core/travel.rs']
must(sim, ['pub(super) fn provision_case_site_journey(',
           'TravelProvisionDeferralReason::RequiresSettlement',
           'waterskin_count: count_item(adventuresim_core::item_references::STANDARD_WATERSKIN_ID),'])
must(sources[module + 'item.rs'], ['item.water_capacity_ml = if definition.id == "waterskin" {',
                                   'container.capacity_ml',
                                   'item.container_capacity_ml = container.capacity_ml;'])
catalog = json.loads(sources['content/items/catalog.yaml'])
items = {item['id']: item for item in catalog['items']}
assert items['waterskin']['capabilities']['container']['capacity_ml'] == 4000
assert items['cooking_pan']['exterior_volume_ml'] == 3000
assert items['glass_bottle']['capabilities']['container']['capacity_ml'] == 250
# Source-derived arithmetic only, assuming the stated no-alcohol one-person state.
body, skin_capacity, occupied, water = 4000, 4000, 3000, 1000
assert skin_capacity - occupied - water == 0
assert (body + skin_capacity) / 4000 == 2
assert (body + water) / 4000 == 1.25
assert max(0, 2 * 4000 - body - skin_capacity) == 0
assert max(0, 2 * 4000 - body - water) == 3000
assert (body + 250) / 4000 == 1.0625
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
for item, lens in zip(doc['recovered_historical_receipts'], (12, 13)):
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    old = json.loads(raw)
    assert old['lens'] == lens and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
support = {}
for item in doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 8
old12 = json.loads(preserved(doc['recovered_historical_receipts'][0]))
proposal, proposal_doc = support[old12['proposal']]
assert proposal['sha256'] == old12['proposal_sha256']
row = next(row for row in old12['rows'] if row['original_index'] == 2374)
reg, reg_doc = support[row['registration']]
assert reg['sha256'] == row['registration_sha256']
qual, qual_doc = support[row['identity_specific_windows_qualification']['path']]
assert qual['sha256'] == row['identity_specific_windows_qualification']['sha256']
assert reg_doc['original_index'] == qual_doc['original_index'] == 2374
assert reg_doc['source_sha256'] == qual_doc['source_sha256'] == row['source_sha256']
connected = proposal_doc['connected_receipt']
assert support[connected['path']][0]['sha256'] == connected['sha256']
old13 = json.loads(preserved(doc['recovered_historical_receipts'][1]))
for ref in (old13['delegated_receipt'], old13['delegated_verification'],
            old13['agent_validation_manifest'], old13['evidence']['registration']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior12 = ledger['segovb_continuation']['lens12']['pending_connected_contracts'][983][0]
assert 'assumeswaterskinswillbefilled' in prior12 and 'noautomaticdoublewaterbug' in prior12
prior13 = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][346]
assert prior13['receipt'] == doc['recovered_historical_receipts'][1]['original_path']
assert prior13['contract']['proposed_id'] == 'PL13-256-CONTAINED-WATER-VOLUME-AND-PROVENANCE-PENDING'
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
assert name in progress['post_transition_bounded_producer_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 11 Git-bound sources, hypothetical supply arithmetic and 2 original receipts and 8 support records; no product ran')
