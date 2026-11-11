"""Verify bounded inventory audit evidence and source relationships only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-INVENTORY-MUTATION-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
assert doc['source_revision'] == rev and doc['lenses'] == [1, 3, 4, 5]
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert row['revision'] == rev and len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert all(1 <= a <= b <= row['lines'] for a, b in row['ranges'])
    contents[row['path']] = raw

module = 'crates/adventuresim-stdb-module/src/'
core = 'crates/adventuresim-core/src/'
item = contents[module + 'item.rs']
change = item.split(b'pub fn change_inventory_item(', 1)[1].split(b'\n#[cfg(test)]', 1)[0]
assert b'require_living_character(ctx, character_id)?' in change
assert b'require_strategic_character_authority' not in change and b'ctx.sender()' not in change
assert b'.is_some_and(|definition| definition.repairable)' in change
assert b'let measured = crate::inventory_amount::is_measured_item(ctx, item_id)' in change
assert b'item.quantity.checked_add(addition)' in change and b'item.quantity = quantity;' in change
assert b'add_inventory_item(ctx, character_id, item_id, addition);' in change
assert b'delete_carried_object_for_row' in change  # selected durable branch only
non_durable = change.split(b'let food = ctx', 1)[1]
assert b'delete_carried_object_for_row' not in non_durable and b'reconcile_consumed_row' not in non_durable
grant = item.split(b'pub(crate) fn add_inventory_item_checked(', 1)[1].split(b'pub(crate) fn add_foraged_inventory_item_checked_rows', 1)[0]
assert grant.index(b'ctx.db.inventory_item().insert') < grant.index(b'initialize_personal_weapon(ctx, &item)?')
assert b'if quantity == 0' in grant and b'let individual = requires_stable_object' in grant
assert b'insert_personal_object(ctx, &item)?' in grant
wrapper = item.split(b'pub fn add_inventory_item(', 1)[1].split(b'\npub fn is_currency', 1)[0]
assert b'.ok()' in wrapper and b'.flatten()' in wrapper
weapon = contents[module + 'weapon_instance.rs'].split(b'fn initialize_personal_weapon(', 1)[1].split(b'\npub(crate) fn initialize_party_weapon', 1)[0]
assert b'Unknown weapon definition' in weapon and b'let Some(definition)' in weapon
amount = contents[module + 'inventory_amount.rs']
assert b'pub const SMITHING_MATERIAL_IDS: [&str; 4]' in amount
assert b'definition.id == adventuresim_core::item_references::SOFT_SOAP_ID' in amount
assert b'|| adventuresim_core::food::definition(item_id).is_some()' in amount
for name, end in ((b'consume_personal(', b'\npub fn consume_party('), (b'consume_party(', b'\npub fn move_personal_to_party(')):
    body = amount.split(b'pub fn ' + name, 1)[1].split(end, 1)[0]
    assert b'if remaining.is_zero()' in body and b'.delete(' in body
    assert b'reconcile_consumed_row' not in body and b'inventory_object' not in body
filth = contents[module + 'filth.rs']
wash_plan = filth.split(b'fn plan_party_wash(', 1)[1].split(b'\npub fn preview_party_wash', 1)[0]
assert b'.filter((id, SOFT_SOAP_ID))' in wash_plan and b'personal_fraction(ctx, stack.id)' in wash_plan
wash = filth.split(b'pub fn wash_party_before_explicit_rest(', 1)[1].split(b'\npub fn wash_before_explicit_rest', 1)[0]
assert b'consume_personal(ctx, key.id, *quantity)?' in wash and b'consume_party(ctx, key.id, *quantity)?' in wash
assert b'reconcile_consumed_row' not in filth
smith = contents[module + 'smithing.rs']
assert b'inventory_amount::consume_personal(ctx, row.id, requested_fraction)?' in smith
assert b'reconcile_consumed_row' not in smith and b'consume(ctx, character_id, item_id, *required)?' in smith
assert b'wash_party_before_explicit_rest(ctx, &members)?' in contents[module + 'time/camp_rest.rs']
assert b'wash_before_explicit_rest(ctx, character_id)?' in contents[module + 'time/settlement_rest.rs']
assert b'consume_personal_soap_points(ctx, soap.id, 1)?' in contents[module + 'surgery.rs']
container = contents[module + 'inventory_container.rs']
insert = container.split(b'pub(crate) fn insert_personal_object(', 1)[1].split(b'\npub(crate) fn insert_party_object', 1)[0]
assert b'if row.quantity != 1' in insert
merge = container.split(b'pub(crate) fn merge_empty_container(', 1)[1].split(b'\n#[cfg(test)]', 1)[0]
assert b'object_is_nonempty' in merge and b'object_is_nested' in merge
assert b'item_condition()' in merge and b'inventory_object().id().delete(emptied.id)' in merge
assert b'pub(crate) fn reconcile_consumed_row(' in container
assert b'pub(crate) fn delete_carried_object_for_row(' in container
fill = container.split(b'pub(crate) fn fill_carried_waterskins(', 1)[1].split(b'\npub(crate) fn consume_contained_water', 1)[0]
assert fill.index(b'resolve_object_custody(ctx, &object)?') < fill.index(b'.root != *custody')
assert b'.filter(STANDARD_WATERSKIN_ID)' in fill
custody = contents[module + 'object_custody.rs']
assert b'row.quantity != 1' in custody and b'Inventory object conflicts with its personal row custody' in custody
assert b'Inventory object personal row is missing' in custody
condition = contents[module + 'condition.rs']
assert condition.count(b'fill_carried_waterskins(ctx, &custody)?') >= 3
assert b'replenish_needs_at_settlement(ctx, traveler_id)?' in contents[module + 'strategic/travel_reducers.rs']
actor = contents[module + 'character.rs'].split(b'pub fn require_living_character(', 1)[1].split(b'\n}', 1)[0]
assert b'if !character.alive' in actor and b'ctx.sender()' not in actor
policy = contents[module + 'strategic/authority_model.rs']
assert b'authority.identity != ctx.sender()' in policy and b'sender_owns_simulation_character(ctx, character_id)' in policy
binding = contents['crates/adventuresim-stdb-client/src/change_inventory_item_reducer.rs']
assert b'pub trait change_inventory_item' in binding and b'character_id: u64' in binding and b'by_quantity: i32' in binding
catalog = json.loads(contents['content/items/catalog.yaml'])
items = {x['id']: x for x in catalog['items']}
assert {k: items[k] for k in ('waterskin', 'soft_soap')} == doc['authored_content_inventory']['selected']
assert items['waterskin']['kind'] == 'container' and items['waterskin']['capabilities']['container']['capacity_ml'] == 4000
assert 'durability' not in items['waterskin']['capabilities'] and items['soft_soap']['kind'] == 'simple'
assert b'SOAP_CLEANSING_CAPACITY: u16 = 25' in contents[core + 'filth.rs']
assert b'MICROS_PER_WHOLE: u32 = 1_000_000' in contents[core + 'inventory_measurement.rs']
assert 1_000_000 // 25 * 25 == 1_000_000  # symbolic source arithmetic only

native = load('native-evidence/spacetimedb-2.6.1/MANIFEST.json')
archive = root / 'native-evidence/spacetimedb-2.6.1' / native['archive']
raw = archive.read_bytes()
assert sha(raw) == native['sha256'] and len(raw) == native['bytes']
with tarfile.open(archive, 'r:gz') as t:
    for suffix in ('src/lib.rs', 'src/table.rs'):
        entry = next(x for x in native['inspected_members'] if x['path'].endswith('/' + suffix))
        data = t.extractfile(entry['path']).read()
        assert sha(data) == entry['sha256'] and len(data) == entry['bytes']
        if suffix.endswith('lib.rs'):
            assert b'that can be invoked remotely by [clients]' in data
            assert b'updates to the database are only committed if the reducer returns successfully' in data
        else:
            assert b'datastore_delete_by_index_scan_point_bsatn(index_id, &point)' in data
transport = {x['path']: x for x in json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']
assert len(records) == 12
for row in records:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    original = transport[row['original_path']]
    assert row['sha256'] == original['sha256'] and row['git_blob'] == original['git_blob']
    assert row['bytes'] == original['bytes'] and sha(raw) == row['published_sha256'] and len(raw) == row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
assert not any(r['redacted_local_path_occurrences'] for r in records)
old = {r['original_path']: json.loads(gzip.decompress((root / r['preserved_bytes']).read_bytes()))
       for r in records if r['original_path'] in ('segovb/READ-L12-258.json', 'segovb/READ-L11-659.json')}
assert old['segovb/READ-L12-258.json']['accepted_unique_count_change'] == 0
assert old['segovb/READ-L11-659.json']['source_revision'] == 'e2755a732c84e67296b5e3441a4259a84f461b2f'
assert old['segovb/READ-L11-659.json']['accepted_unique_count_change'] == 0
history = load('ASSESSMENT-L01-HISTORICAL-1136-1138.json')['historical_source_roles']
assert [x['original_index'] for x in history] == [1136, 1137, 1138]
roles = load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as t:
    for h in history:
        raw = t.extractfile(h['sha256']).read()
        assert sha(raw) == h['sha256'] and len(raw.splitlines()) == h['lines']
        assert h['new_personal_read_ranges'] == [[1, h['lines']]]
        assert roles[h['original_index']]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1136-1138.json'
        assert roles[h['original_index']]['status'] == 'full-historical-body-trace-open'
        assert h['current_comparison']['byte_identical'] is True
        assert blob(raw) == tree[h['source_id'].split(':', 1)[1]]
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len(doc['candidates']) == 4
for c in doc['candidates']:
    assert len([x for x in ledger if x['id'] == c['id'] and x['record'] == 'ASSESSMENT-INVENTORY-MUTATION-95E.json']) == 1
queue = load('RECONCILIATION-059.json')['entries']
for i in (218, 946, 947):
    assert queue[i]['current_reconciliation'].startswith('partial ')
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger)
assert progress['queue_records_semantically_reconciled'] == 0 and progress['lenses_complete'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified inventory source, archive and prior evidence; no product ran')
