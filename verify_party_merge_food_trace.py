"""Verify party-merge food evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-PARTY-MERGE-FOOD-95E.json'
doc = json.loads((root / name).read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if doc['source_revision'] != revision or doc['lens'] != 1:
    raise ValueError('wrong source or lens')

def tree(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in result:
            raise ValueError('duplicate tree path')
        result[path] = oid
    return result

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe or incorrect source reference')
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
        hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + item['path'])
    if item['path'] in sources:
        raise ValueError('duplicate source')
    for first, last in item['ranges']:
        if not 1 <= first <= last <= item['lines']:
            raise ValueError('invalid read range')
    sources[item['path']] = raw.decode()
if len(sources) != 10:
    raise ValueError('incorrect source count')
governance = sources['crates/adventuresim-stdb-module/src/strategic/governance.rs']
merge = governance.split('pub fn accept_party_join_request(', 1)[1].split('pub fn reject_party_join_request(', 1)[0]
for value in ('require_strategic_character_authority(ctx, leader_id)?',
              'if item_is_durable(ctx, &entry.item_id)',
              'crate::inventory_container::rehome_subtree(ctx, object.id, &destination)?',
              'add_to_party_inventory(ctx, &request.party_id, &entry.item_id, entry.quantity);',
              'ctx.db.party_inventory_item().id().delete(entry.id);',
              'normalize_and_elect_party_leader(ctx, &request.party_id)?'):
    if value not in merge:
        raise ValueError('merge source marker missing')
branch = merge.split('if item_is_durable(ctx, &entry.item_id)', 1)[1].split('for stake in ctx', 1)[0]
if not (branch.index('rehome_subtree') < branch.index('add_to_party_inventory') <
        branch.index('party_inventory_item().id().delete(entry.id)')):
    raise ValueError('merge branch ordering changed')
trade = sources['crates/adventuresim-stdb-module/src/strategic/inventory_trade.rs']
for value in ('fn item_is_durable(', '.is_some_and(|definition| definition.repairable)',
              'let _ = add_to_party_inventory_checked(ctx, party_id, item_id, quantity)',
              'requires_stable_object(definition.as_ref(), food, measured)',
              'crate::inventory_amount::initialize_party(ctx, row.id)',
              'crate::food::create_party_food_lot(ctx, row.id, item_id, 1, minute)',
              'crate::inventory_container::rehome_subtree(ctx, object.id, &destination)?'):
    if value not in trade:
        raise ValueError('grant or deposit source marker missing')
item = sources['crates/adventuresim-stdb-module/src/item.rs']
for value in ('#[derive(Clone, Debug, Default)]', '..Item::default()',
              'if let Some(durable) = definition.capabilities.durability', 'item.repairable = true',
              'food || measured', 'pub(crate) fn inventory_food_definition('):
    if value not in item:
        raise ValueError('item source marker missing')
amount = sources['crates/adventuresim-stdb-module/src/inventory_amount.rs']
for value in ('definition.kind == crate::CatalogItemKind::Food', 'pub fn initialize_party(',
              'remaining_fraction_micros: ConsumableFractionMicros::WHOLE.get()'):
    if value not in amount:
        raise ValueError('amount source marker missing')
lot = sources['crates/adventuresim-stdb-module/src/food/lot_inventory.rs']
for value in ('pub fn create_party_food_lot(', 'party_inventory_item_id: Some(inventory_item_id)',
              'nutrition_kcal: definition.kcal_per_unit * quantity as f32',
              'created_at_minute: minute', 'concentration_anchor: food::deterministic_initial_contamination',
              'lot.nutrition_kcal = food::retained_component(lot.nutrition_kcal, retained)',
              'source.party_inventory_item_id = Some(destination_party_id)'):
    if value not in lot:
        raise ValueError('food lot source marker missing')
consumption = sources['crates/adventuresim-stdb-module/src/food/consumption.rs']
for value in ('pub fn consume_travel_food_to_zero(', 'let retained = 1.0 - ratio',
              'retain_lot_fraction(&mut lot, retained)?', 'remaining_fraction_micros: remaining.get()'):
    if value not in consumption:
        raise ValueError('partial consumption source marker missing')
model = sources['crates/adventuresim-stdb-module/src/food/model.rs']
for value in ('pub struct FoodLot', 'pub ingredient_quantities: Vec<f32>',
              'pub nutrition_kcal: f32', 'pub struct FoodContaminationProvenance'):
    if value not in model:
        raise ValueError('food material model marker missing')
container = sources['crates/adventuresim-stdb-module/src/inventory_container.rs']
for value in ('pub(crate) fn rehome_subtree(', 'prevalidate_rehome_subtree(ctx, root_object_id, destination)?',
              'row.party_id = party_id.as_str().into()', 'object.location = InventoryLocation::party'):
    if value not in container:
        raise ValueError('stable custody source marker missing')
core = sources['crates/adventuresim-core/src/food.rs']
if 'kcal_per_unit: food.nutrition_kcal' not in core or 'pub fn definition(id: &str)' not in core:
    raise ValueError('catalog food projection marker missing')
catalog = json.loads(sources['content/items/catalog.yaml'])
rations = [x for x in catalog['items'] if x['id'] == 'travel_ration']
if len(rations) != 1 or rations[0]['kind'] != 'food' or rations[0]['capabilities']['food']['nutrition_kcal'] != 2500 or 'durability' in rations[0]['capabilities']:
    raise ValueError('ration source premise changed')

native = doc['native_source']
manifest_path = (root / native['package_manifest']).resolve()
if not manifest_path.is_relative_to(root):
    raise ValueError('unsafe native path')
manifest = json.loads(manifest_path.read_text())
raw = (manifest_path.parent / manifest['archive']).read_bytes()
if len(raw) != manifest['bytes'] or hashlib.sha256(raw).hexdigest() != manifest['sha256']:
    raise ValueError('native archive mismatch')
with tarfile.open(manifest_path.parent / manifest['archive'], 'r:gz') as tar:
    source = tar.extractfile(native['archive_member']).read()
expected = next(x for x in manifest['inspected_members'] if x['path'] == native['archive_member'])
if len(source) != expected['bytes'] or hashlib.sha256(source).hexdigest() != expected['sha256']:
    raise ValueError('native member mismatch')
if b'only committed if the reducer returns successfully' not in source:
    raise ValueError('native transaction documentation missing')
for first, last in native['ranges']:
    if not 1 <= first <= last <= len(source.splitlines()):
        raise ValueError('invalid native range')

historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies']['SEGOVB-L12-PARTY-GRANT-ERROR-ERASURE']
if (historical['checkpoint'] != '059' or old['source_original_index'] != historical['matching_grant_role_index'] or
    old['source_sha256'] != historical['matching_grant_sha256'] or old['accepted_unique_count_change'] != 0 or
    old['current_reconciled'] is not False or old['runtime_validation'] != 'Not run'):
    raise ValueError('historical grant record qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
identities = [(historical['matching_governance_role_index'], historical['matching_governance_sha256']),
              (historical['different_physical_p16_role_index'], historical['different_physical_p16_sha256']),
              (historical['matching_grant_role_index'], historical['matching_grant_sha256'])]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index, sha in identities:
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
if (historical['matching_governance_sha256'] != hashlib.sha256(governance.encode()).hexdigest() or
    historical['matching_grant_sha256'] != hashlib.sha256(trade.encode()).hexdigest() or
    historical['different_physical_p16_sha256'] == historical['matching_governance_sha256']):
    raise ValueError('historical current/different distinction changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
    candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified ten Git-bound food/merge sources, native documentation and separately attributed grant record')
