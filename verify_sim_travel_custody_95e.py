#!/usr/bin/env python3
"""Verify source identities, prior read qualifications and symbolic examples only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

r = Path(__file__).resolve().parent
load = lambda p: json.loads((r / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
base = 'crates/adventuresim-strategic-sim/src/live_core/'
a = load('ASSESSMENT-SIM-TRAVEL-CUSTODY-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1295-1297.json')
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
work = load('COVERAGE-L01-ROLES.json')
tree = {}
for line in (r / work['current_tree_inventory']).read_text().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, oid = header.split()
    tree[path] = oid
s = {}
for item in a['current_sources']:
    raw = gzip.decompress((r / item['preserved_bytes']).read_bytes())
    assert item['revision'] == rev and len(raw) == item['bytes'] and sha(raw) == item['sha256']
    assert blob(raw) == item['git_blob'] == tree[item['path']] and len(raw.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    s[item['path']] = raw
assert [x['original_index'] for x in h['historical_source_roles']] == [1295,1296,1297]
assert [x['lines'] for x in h['historical_source_roles']] == [957,343,1587]
for item in h['historical_source_roles']:
    role = work['roles'][item['original_index']]
    assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1295-1297.json'
    assert item['new_personal_read_ranges'] == [[1,item['lines']]]
    with tarfile.open(r/item['evidence_archive'], 'r:gz') as archive:
        raw = archive.extractfile(item['sha256']).read()
    assert sha(raw) == item['sha256'] and len(raw.splitlines()) == item['lines']
    assert raw == s[item['source_id'].split(':',1)[1]]

survival = s[base+'survival.rs']; travel = s[base+'travel.rs']; thermal = s[base+'thermal_projection.rs']
item = s['crates/adventuresim-stdb-module/src/item.rs']; amount = s['crates/adventuresim-stdb-module/src/inventory_amount.rs']
container = s['crates/adventuresim-stdb-module/src/inventory_container.rs']; lookup = s['crates/adventuresim-stdb-module/src/inventory_container/object_lookup.rs']
mass = s['crates/adventuresim-stdb-module/src/capability/mass.rs']; fireplace = s['crates/adventuresim-stdb-module/src/food/fireplace_custody.rs']
consumer = s['crates/adventuresim-stdb-module/src/food/consumption.rs']; provision = s['crates/adventuresim-core/src/provisioning.rs']
assert b'[] | [_, _, ..] => false' in survival
assert b'row.character_id == character_id && self.public_row_is_carried(&custody, row.id)' in survival
assert b'row.party_id == party_id && self.public_row_is_carried(&party_custody, row.id)' in survival
assert b'pub(super) fn validate_party_departure_readiness' in survival and b'public_party_load_and_capacity(party_id)' in survival
assert b'let individual = requires_stable_object(definition.as_ref(), food, measured);' in item
assert b'quantity: if individual { 1 } else { quantity }' in item and b'if individual {\n            crate::inventory_container::insert_personal_object' in item
assert b'pub fn is_measured_definition' in amount and b'SMITHING_MATERIAL_IDS.contains' in amount
assert b'map(|object| object.is_some_and(|object| ancestry_reaches_fireplace(ctx, object.id)))' in container
assert b'Ok(first)' in lookup and b'return None;' in mass and b'unit_mass * effective_quantity' in mass
assert b'crate::inventory_container::row_is_fireplace_rooted' in mass
assert b'let personal_inventory_ids = personal_inventory' in travel and b'let party_inventory_ids = party_inventory' in travel
assert b'.is_some_and(|id| personal_inventory_ids.contains(&id))' in travel and b'.is_some_and(|id| party_inventory_ids.contains(&id))' in travel
assert b'object.location = InventoryLocation::fireplace(fireplace_fixture_id.clone());' in fireplace
assert b'ctx.db.inventory_item().id().delete(location.row_id);' in fireplace
assert b'Children retain their stable object edges' in fireplace
assert consumer.count(b'row_is_fireplace_rooted') >= 2
assert b'pub fn put_inventory_item_in_container' in container and b'if used > Milliliters::new(u64::from(definition.container_capacity_ml))' in container
assert b'fn require_clear_current_camp_fireplace' in fireplace
native_travel = s['crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs']
assert b'if party_journey_is_current_camp(&party, &journey)' in native_travel and b'require_clear_current_camp_fireplace(ctx, &camp_place)?' in native_travel
assert b'target_surplus_days: TRAVEL_PROVISION_RESERVE_DAYS' in travel
assert b'TRAVEL_PROVISION_RESERVE_DAYS: f32 = 1.0' in s[base+'departure_policy.rs']
assert b'STRATEGIC_TRAVEL_KCAL_PER_DAY: f32 = 6_000.0' in s['crates/adventuresim-core/src/provisioning.rs']
assert b'if itinerary.truncated' in thermal and b'itinerary.total_elapsed_minutes > MAX_CASE_SITE_THERMAL_FORECAST_MINUTES' in thermal
catalog = json.loads(s['content/items/catalog.yaml']); byid = {x['id']:x for x in catalog['items']}
arrow, pan, ration = (byid[x] for x in ('arrow','cooking_pan','travel_ration'))
assert arrow['weight_kg'] == 0.05 and arrow['kind'] == 'simple' and 'equipment' not in arrow and 'capabilities' not in arrow
assert pan['capabilities']['container']['capacity_ml'] == 2500 and ration['exterior_volume_ml'] == 813
assert ration['capabilities']['food']['nutrition_kcal'] == 2500
gameplay = json.loads(s['crates/adventuresim-weapon-model/catalog/gameplay.json'])
assert not any(x['catalog_id'] == 'arrow' for x in gameplay)
assert b'item("arrow", if adult { 24 } else { 40 }, None)' in s['crates/adventuresim-core/src/starting_character.rs']
assert b'add_inventory_item(ctx, character.id, &item.item_id, item.quantity)' in s['crates/adventuresim-stdb-module/src/character.rs']
assert 24 * 5 == 120 and 4 * 2500 + 2500 == 12500 and 2 * 6000 == 12000 and 4 * 2500 == 10000

manifest = json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()))
lookup_manifest = {x['path']:x for x in manifest['entries']}; records = {}
for item in a['recovered_historical_records']:
    orig = lookup_manifest[item['original_path']]
    assert item['sha256'] == orig['sha256'] and item['bytes'] == orig['bytes'] and item['git_blob'] == orig['git_blob']
    raw = gzip.decompress((r/item['preserved_bytes']).read_bytes())
    assert len(raw) == item['published_bytes'] and sha(raw) == item['published_sha256']
    assert item['redacted_local_path_occurrences'] or (sha(raw) == item['sha256'] and blob(raw) == item['git_blob'])
    records[item['original_path']] = json.loads(raw)
for batch, index, lines in [(347,1295,957),(348,1296,343),(349,1297,1587)]:
    receipt = records[f'segovb/READ-L12-{batch}.json']
    assert receipt['lens'] == 12 and receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['rows'][0]['original_index'] == index and receipt['rows'][0]['fresh_personal_body_reads'] == [[1,lines]]
    assert records[f'segovb/VERIFY-L12-{batch}.json']['receipt_sha256'] == lookup_manifest[f'segovb/READ-L12-{batch}.json']['sha256']
for num, span, tail in [(1,[1,600],[[601,1587]]),(2,[601,1200],[[1201,1587]]),(3,[1201,1587],[])]:
    suffix = '-EOF' if num == 3 else ''
    part = records[f'segovb/PARTIAL-L12-349-{num:03d}{suffix}.json']
    assert part['fresh_personal_body_reads'] == [span] and part['unread_tail'] == tail
    assert part['other_lens_coverage_transferred'] is False and part['new_original_overlay_merged_shared_population_credit'] == 0
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1086,1087,1089))
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert all(sum(x['id'] == c['id'] for x in ledger) == 1 for c in a['candidates'])
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
print('Verified three historical reads, prior partials and source-bound custody traces; no product execution.')
