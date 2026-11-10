"""Verify creator inventory source and original receipts without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-INVENTORY-BOUNDARIES-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision


def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in result and kind in ('blob', 'commit')
        result[filename] = oid
    return result


def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw


def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)


current = tree('main-advance-95e/current-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == current[item['path']]
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 12
p = 'crates/adventuresim-character-creator/src/'
q = 'crates/adventuresim-core/src/'
inventory = sources[p + 'inventory.rs']
must(inventory, ['pub struct InventoryItemId(pub u64);', 'pub struct InventoryItem {',
    'pub id: InventoryItemId,', 'pub fn get_mut(&mut self, id: InventoryItemId)',
    '.map(|item| item.id.0 + 1)', '.max()', '.unwrap_or(1)',
    'self.items.push(InventoryItem {', 'pub fn validate(&self) -> Result<(), String>',
    'self.items[..index].iter().any(|other| other.id == item.id)',
    'item.article.validate()?;', 'WornGraph::build(candidates)'])
occupancy = sources[p + 'inventory/occupancy.rs']
must(occupancy, ['for requirement in &candidate.occupancy.parents',
    'let mut edge = self.support(candidate, requirement, placed)?;',
    'edge.capacity_index += edges', 'let used = self.children_on(parent.id, &point.id);',
    '(accepts && used < point.capacity).then(|| EquipmentGraphEdge {',
    'capacity_index: used,', '.equip(candidate.id.0, placement)'])
assert occupancy.index('self.support(candidate, requirement, placed)?') < occupancy.index('edge.capacity_index += edges')
must(sources[p + 'inventory/loadout.rs'], ['self.fit(catalog)?;', 'pub fn loadout'])
must(sources[p + 'inventory_acquire.rs'], ['let id = studio.recipe.inventory.add(article);',
    'studio.inventory.selected = Some(id);', 'super::wear(studio, catalog, id);'])
must(sources[p + 'lib.rs'], ['pub struct CharacterRecipe', 'self.inventory.validate()'])
must(sources[p + 'main.rs'], ['fn load_recipe(path: &str)',
    'serde_json::from_slice(&std::fs::read(path)?)?', 'recipe.validate().map_err(anyhow::Error::msg)?'])
must(sources[p + 'equipment_catalog.rs'], ['pub fn new(items: Vec<ItemDefinition>, designs: CatalogDesigns)',
    'validate_armor_recipes(&items)?;', 'pub fn placement(&self, item_id: &str, placement_id: &str)'])
graph = sources[q + 'equipment.rs']
must(graph, ['pub struct EquipmentGraphEdge', 'pub capacity_index: u16,',
    'let edge_keys = placement.parents.iter().cloned().collect::<BTreeSet<_>>();',
    'edge_keys.len() != placement.parents.len()', 'other.parents.iter().any(|edge| edge_keys.contains(edge))',
    '!self.nodes.contains_key(&edge.parent_inventory_item_id)',
    'self.would_cycle(', 'self.nodes.insert(inventory_item_id, placement);'])
must(sources[q + 'equipment_assets/selection.rs'], ['pub fn placement(self, id: &str)',
    '.find(|selection| selection.placement.id == id)'])
must(sources[q + 'item_catalog_schema.rs'], ['pub struct EquipmentPlacement',
    'pub parents: Vec<ParentRequirement>,', 'pub struct AttachmentPointDefinition', 'pub capacity: u16,'])
must(sources[q + 'item_catalog_validation_occupancy.rs'], ['for (index, parent) in parents.iter().enumerate()',
    'serde_json::from_value::<crate::item_catalog_schema::ParentRequirement>(parent.clone())'])
must(sources[q + 'item_catalog_validation.rs'], ['occupancy::validate_dependencies(&parents, placement, &placement_path, errors);',
    'capacity == 0 || capacity > u16::MAX.into()'])
# Evidence-only arithmetic for unchecked successor and prospective slots.
maximum = (1 << 64) - 1
assert max((x + 1) % (1 << 64) for x in (maximum, maximum - 1)) == maximum
used, capacity = 0, 1
assert used < capacity
slots = [used, used + 1]
assert slots == [0, 1] and len(set(slots)) == 2 and slots[1] >= capacity
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entries = {item['path']: item for item in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) == item['bytes']
    receipts[item['original_path'].rsplit('/', 1)[-1]] = (item, json.loads(raw))
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']
old = ledger['current_connected_dispositions'][276]
assert old['disposition']['id'] == doc['historical_records'][0]['id']
assert old['scope_revision'] == doc['historical_records'][0]['revision']
assert old['receipt']['sha256'] == receipts['READ-L11-716.json'][0]['sha256']
assert old['receipt']['verification_sha256'] == receipts['VERIFY-L11-716.json'][0]['sha256']
for index, outer, delegated, record in [(12, 'READ-L13-PARALLEL-114.json', 'READ-PL13-116.json', 1),
                                         (14, 'READ-L13-PARALLEL-122.json', 'READ-PL13-124.json', 2)]:
    finding = ledger['lens13']['delegated_findings_pending_primary_provenance_and_alias'][index]
    assert finding['receipt_sha256'] == receipts[outer][0]['sha256']
    assert finding['delegated_receipt']['sha256'] == receipts[delegated][0]['sha256']
    assert finding['finding']['proposed_id'] == doc['historical_records'][record]['id']
    assert finding['accepted_unique_count_change'] == 0 and finding['current_reconciled'] is False
    assert finding['finding']['primary_read']['sha256'] == receipts['READ-PL13-113.json'][0]['sha256']
    assert finding['finding']['primary_registration']['sha256'] == receipts['REGISTRATION-PL13-113.json'][0]['sha256']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id']
    assert role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
assert work['roles'][809]['current_observations']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
entries = base['newly_assessed_ledger']['entries']
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
    assert candidate['discovery'].startswith('partial')
    assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
print('verified twelve Git-bound sources, eight original receipts and bounded arithmetic; no product ran')
