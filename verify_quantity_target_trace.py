"""Verify preserved target-authority evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-QUANTITY-TARGET-95E.json'
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
if len(sources) != 6:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'pub fn set_inventory_quantity_target(', 'crate::character::require_living_character(ctx, character_id)?',
        'if party.leader_id != character_id', 'ctx.db.inventory_quantity_target().id().update(row)',
        'ctx.db.inventory_quantity_target().insert(row)', 'fn require_strategic_character_authority('],
    'crates/adventuresim-stdb-module/src/character.rs': [
        'pub fn require_living_character(', 'if !character.alive'],
    'crates/adventuresim-stdb-client/src/set_inventory_quantity_target_reducer.rs': [
        'pub trait set_inventory_quantity_target', 'invoke_reducer_with_callback('],
    'crates/strategic-web/src/routes/settlements/party/inventory_medical.rs': [
        'session.character_id_u64()', 'execute_or_request_party_action(',
        '.call("set_inventory_quantity_target", &args)'],
    'crates/adventuresim-stdb-module/src/strategic/approved_action.rs': [
        'set_inventory_quantity_target(ctx, leader_id, true, item_id, quantity)'],
    'crates/adventuresim-stdb-module/src/alcohol.rs': [
        'fn target_for(', 'let reserve = if settled',
        'target_for(ctx, owner, party_scope, item_id)', 'total.saturating_sub(reserve)',
        'available_stacks(ctx, character_id, settled, allow_medical, false)'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
authority = sources['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
setter = authority.split('pub fn set_inventory_quantity_target(', 1)[1].split('/// Member contribution entitlement', 1)[0]
if any(value in setter for value in ('ctx.sender', 'require_strategic_character_authority', 'require_strategic_gateway')):
    raise ValueError('bounded caller-authority observation changed')

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
old = ledger['segovb_continuation']['current_connected_dispositions'][132]
if (historical['checkpoint'] != '059' or old['scope_revision'] != historical['scope_revision'] or
    old['latest_current_target_reconciled'] is not False or
    old['disposition']['id'] != historical['disposition_id'] or
    old['disposition']['accepted_unique_count_change'] != 0):
    raise ValueError('historical disposition qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
archive_manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in archive_manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index in historical['matching_authority_model_roles']:
        role = roles[index]
        sha = historical['authority_model_sha256']
        if (role['historical_sha256'] != sha or archive_hashes[index] != sha or
            hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha or
            sha != hashlib.sha256(authority.encode()).hexdigest()):
            raise ValueError('historical source relationship mismatch')
if len(historical['matching_authority_model_roles']) != 2:
    raise ValueError('incorrect historical role count')
if roles[268]['historical_sha256'] == hashlib.sha256(sources['crates/adventuresim-stdb-module/src/strategic/approved_action.rs'].encode()).hexdigest():
    raise ValueError('distinct historical approved-action body claim changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or
    candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
    not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified six Git-bound sources, native documentation and separately attributed actor-admission record')
