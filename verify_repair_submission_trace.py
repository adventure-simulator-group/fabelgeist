"""Verify preserved repair submission evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-REPAIR-SUBMISSION-95E.json'
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
if len(sources) != 13:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/repair.rs': [
        'pub fn submit_item_for_repair', 'pub fn submit_all_repairable_items',
        'if submit(ctx, character_id, &settlement_id, service, id).is_ok()',
        'if submitted == 0', 'pub fn retrieve_repaired_item',
        'if order.owner_character_id != character_id',
        'crate::inventory_container::detach_if_nested(ctx, object.id)?'],
    'crates/adventuresim-stdb-module/src/repair/smiths.rs': [
        'fn service_skill(', 'require_settlement_service'],
    'crates/adventuresim-stdb-module/src/strategic/world_import.rs': [
        'fn require_settlement_service(', 'settlement.economy.has_service(service)'],
    'crates/adventuresim-stdb-module/src/capability.rs': [
        'pub fn evaluate_character(', 'pub fn refresh_character_capability('],
    'crates/adventuresim-stdb-module/src/condition.rs': [
        'pub fn refresh_character_strategic_condition(', 'fn refuse_expired_holy_day_demands('],
    'crates/adventuresim-stdb-module/src/inventory_container.rs': [
        'pub(crate) fn detach_if_nested(', 'merge_empty_container(ctx, parent_id)?',
        'fn require_exact_carried_backing(', 'pub(crate) fn merge_empty_container(',
        '.checked_add(source.quantity)', 'Container stack quantity overflow'],
    'crates/adventuresim-stdb-module/src/inventory_container/object_lookup.rs': [
        'pub(crate) fn object_for_row(', 'Inventory row has duplicate physical object identities'],
    'crates/adventuresim-stdb-module/src/object_custody.rs': [
        'pub(crate) fn carried_location_custody(', 'fn persisted_location_custody(',
        'fn require_unique_backing_object('],
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'fn require_strategic_character_authority(', 'authority.identity != ctx.sender()'],
    'crates/strategic-web/src/routes/settlements/medical.rs': [
        'pub(super) async fn submit_repair(', 'get_active_character(&state, session.character_id_u64())',
        'pub(super) async fn retrieve_repair('],
    'crates/adventuresim-stdb-client/src/submit_item_for_repair_reducer.rs': [
        'pub trait submit_item_for_repair', 'invoke_reducer_with_callback('],
    'crates/adventuresim-stdb-client/src/retrieve_repaired_item_reducer.rs': [
        'pub trait retrieve_repaired_item', 'invoke_reducer_with_callback('],
    'crates/adventuresim-stdb-module/src/item.rs': ['accessor = inventory_item, public'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
repair = sources['crates/adventuresim-stdb-module/src/repair.rs']
section = repair.split('fn submit(', 1)[1].split('fn saved_attachment_targets(', 1)[0]
retrieval = repair.split('fn retrieve(', 1)[1].split('#[reducer]', 1)[0]
for value in ('ctx.sender', 'require_strategic_character_authority', 'require_strategic_gateway'):
    if value in section or value in retrieval:
        raise ValueError('bounded repair authority observation changed')
container = sources['crates/adventuresim-stdb-module/src/inventory_container.rs']
detach = container.split('pub(crate) fn detach_if_nested(', 1)[1].split('pub(crate) fn object_is_nested(', 1)[0]
if detach.index('.delete(object_id)') >= detach.index('merge_empty_container(ctx, parent_id)?'):
    raise ValueError('containment deletion/error order changed')

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
old = ledger['discovery_findings_and_dependencies']['SEGOVB-L12-REPAIR-BULK-ERROR-CLASSIFICATION']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
role = roles[historical['original_index']]
physical = roles[historical['other_physical_p16_index']]
if (historical['checkpoint'] != '059' or old['source_id'] != historical['original_source_id'] or
    old['source_sha256'] != historical['original_source_sha256'] or
    old['source_sha256'] != hashlib.sha256(repair.encode()).hexdigest() or
    old['current_reconciled'] is not False or old['accepted_unique_count_change'] != 0 or
    role['source_id'] != old['source_id'] or role['historical_sha256'] != old['source_sha256'] or
    physical['historical_sha256'] != historical['other_physical_p16_sha256'] or
    physical['historical_sha256'] == old['source_sha256']):
    raise ValueError('historical source relationship mismatch')
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index, expected_hash in [(historical['original_index'], old['source_sha256']),
                                 (historical['other_physical_p16_index'], physical['historical_sha256'])]:
        if archive_hashes[index] != expected_hash or hashlib.sha256(tar.extractfile(expected_hash).read()).hexdigest() != expected_hash:
            raise ValueError('historical archive mismatch')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 2:
    raise ValueError('ledger count mismatch')
for candidate in doc['candidates']:
    matches = [x for x in entries if x['id'] == candidate['id']]
    if (len(matches) != 1 or matches[0]['record'] != name or
        candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
        not candidate['discovery'].startswith('partial')):
        raise ValueError('unqualified candidate')
print('verified thirteen Git-bound sources, transaction documentation and distinct historical role attribution')
