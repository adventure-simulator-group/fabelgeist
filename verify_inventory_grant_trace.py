"""Verify preserved inventory grant evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-INVENTORY-GRANT-95E.json').read_text())
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
if len(sources) != 8:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/item.rs': [
        'pub(crate) fn inventory_food_definition', 'pub(crate) fn add_inventory_item_checked',
        'let definition = ctx.db.item().id().find(item_id.to_owned())',
        'let item = ctx.db.inventory_item().insert(InventoryItem {',
        'crate::weapon_instance::initialize_personal_weapon(ctx, &item)?',
        'add_inventory_item_checked(ctx, character_id, item_id, quantity)\n        .ok()\n        .flatten()',
        'pub fn change_inventory_item', 'crate::character::require_living_character(ctx, character_id)?',
        'add_inventory_item(ctx, character_id, item_id, addition);', 'pub fn credit_personal_currency'],
    'crates/adventuresim-stdb-module/src/character.rs': [
        'pub fn require_living_character', 'if !character.alive'],
    'crates/adventuresim-stdb-module/src/inventory_amount.rs': [
        'pub fn is_measured_item', 'adventuresim_core::food::definition(item_id).is_some()'],
    'crates/adventuresim-stdb-module/src/weapon_instance.rs': [
        'pub(crate) fn initialize_personal_weapon', 'Unknown weapon definition'],
    'crates/adventuresim-core/src/durability.rs': ['pub fn bounded_durable_change'],
    'crates/adventuresim-stdb-client/src/change_inventory_item_reducer.rs': [
        'pub trait change_inventory_item', 'invoke_reducer_with_callback('],
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'pub(crate) fn require_strategic_character_authority', 'authority.identity != ctx.sender()'],
    'crates/adventuresim-stdb-module/src/lib.rs': ['pub mod item;', 'pub use item::*;'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
item = sources['crates/adventuresim-stdb-module/src/item.rs']
reducer = item.split('pub fn change_inventory_item', 1)[1].split('#[cfg(test)]', 1)[0]
if 'ctx.sender' in reducer or 'require_strategic_character_authority' in reducer:
    raise ValueError('bounded caller authority observation changed')
if item.index('let item = ctx.db.inventory_item().insert(InventoryItem {') > item.index('crate::weapon_instance::initialize_personal_weapon(ctx, &item)?'):
    raise ValueError('grant mutation/error order changed')

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
old = ledger['discovery_findings_and_dependencies']['SEGOVB-L12-INVENTORY-GRANT-ERROR-ERASURE']
if (historical['checkpoint'] != '059' or old['source_id'] != historical['original_source_id'] or
    old['source_sha256'] != historical['original_source_sha256'] or
    old['source_sha256'] != hashlib.sha256(item.encode()).hexdigest() or
    old['current_reconciled'] is not False):
    raise ValueError('historical source relationship mismatch')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 2:
    raise ValueError('ledger count mismatch')
for candidate in doc['candidates']:
    matches = [x for x in entries if x['id'] == candidate['id']]
    if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-INVENTORY-GRANT-95E.json' or
        candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
        not candidate['discovery'].startswith('partial')):
        raise ValueError('unqualified candidate')
print('verified eight Git-bound sources, native archive and separately attributed historical record')
