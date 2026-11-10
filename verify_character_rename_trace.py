"""Verify preserved rename source evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-CHARACTER-RENAME-95E.json').read_text())
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
    saved = (root / item['preserved_bytes']).resolve()
    if not saved.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe or incorrect source reference')
    data = gzip.decompress(saved.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    if (len(data) != item['bytes'] or len(data.splitlines()) != item['lines'] or
        hashlib.sha256(data).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + item['path'])
    if item['path'] in sources:
        raise ValueError('duplicate source')
    for first, last in item['ranges']:
        if not 1 <= first <= last <= item['lines']:
            raise ValueError('invalid source range')
    sources[item['path']] = data.decode()
if len(sources) != 12:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/character/name_reducers.rs': [
        '#[reducer]', 'pub fn update_character', 'require_living_character(ctx, id)?',
        'assign_authored_character_name(ctx, CharacterId::new(id), name)'],
    'crates/adventuresim-stdb-module/src/character.rs': [
        'include!("character/name_reducers.rs")', 'pub fn require_living_character',
        'if !character.alive', 'authority.identity == ctx.sender()',
        'require_strategic_character_authority(ctx, character_id)?'],
    'crates/adventuresim-stdb-module/src/character/name_identity.rs': [
        'pub(crate) fn assign_authored_character_name', 'identity.surname_id = character_hereditary_surname',
        'character.name = display.into_string()', 'character_name_identity().character_id().update(row)'],
    'crates/adventuresim-stdb-module/src/character/name_types.rs': [
        'pub(crate) struct CharacterId(u64)', 'pub struct NameIdentityJson', 'serde_json::from_str(value)'],
    'crates/adventuresim-world-schema/src/person_names.rs': [
        'pub fn authored(', 'GivenNameResolution::Authored { full_name }',
        'identity.validate()?', 'Self::MAX_CHARS', 'value.chars().any(char::is_control)'],
    'crates/adventuresim-world-schema/src/person_names/identity_validation.rs': [
        'pub fn validate(&self)', 'RenderedPersonalName::new(full_name.clone())?', '.surnames'],
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'pub(crate) fn require_strategic_character_authority', 'authority.identity != ctx.sender()',
        'sender_owns_simulation_character(ctx, character_id)'],
    'crates/adventuresim-stdb-client/src/update_character_reducer.rs': [
        'pub trait update_character', 'invoke_reducer_with_callback(UpdateCharacterArgs { id, name }'],
    'scripts/check_semantic_authorities.py': [
        'client.call("register_strategic_gateway", None, 0)',
        'client.call("update_character", character_id, "Renamed Candidate")'],
    'Cargo.lock': ['name = "spacetimedb"', 'version = "2.6.1"',
        'b6eff5017248ff2e35e03564df947fdf3e963f7198be6ae1d9e894c2efb2ea15'],
    'Cargo.toml': ['spacetimedb = "=2.6.1"'],
    'crates/adventuresim-stdb-module/src/lib.rs': ['pub mod character;', 'pub use character::*;'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
reducer = sources['crates/adventuresim-stdb-module/src/character/name_reducers.rs']
if 'ctx.sender' in reducer or 'require_strategic_character_authority' in reducer:
    raise ValueError('rename reducer no longer matches bounded observation')

native = doc['native_source']
manifest_path = (root / native['package_manifest']).resolve()
if not manifest_path.is_relative_to(root):
    raise ValueError('unsafe native path')
manifest = json.loads(manifest_path.read_text())
archive = manifest_path.parent / manifest['archive']
raw = archive.read_bytes()
if len(raw) != manifest['bytes'] or hashlib.sha256(raw).hexdigest() != manifest['sha256']:
    raise ValueError('native archive mismatch')
if manifest['sha256'] not in sources['Cargo.lock']:
    raise ValueError('native archive not pinned by lockfile')
with tarfile.open(archive, 'r:gz') as tar:
    member = tar.extractfile(native['archive_member']).read()
if len(member) != native['bytes'] or hashlib.sha256(member).hexdigest() != native['sha256']:
    raise ValueError('native member mismatch')
text = member.decode()
if 'that can be invoked remotely by [clients]' not in text or 'caller `Identity`' not in text:
    raise ValueError('native documentation markers missing')
for first, last in native['ranges']:
    if not 1 <= first <= last <= len(member.splitlines()):
        raise ValueError('invalid native read range')

historical = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-178-184.json').read_text())
hashes = {item['original_index']: item['sha256'] for item in historical['historical_source_roles']}
for index, path in [(179, 'crates/adventuresim-stdb-module/src/character/name_identity.rs'),
                    (180, 'crates/adventuresim-stdb-module/src/character/name_reducers.rs'),
                    (181, 'crates/adventuresim-stdb-module/src/character/name_types.rs')]:
    if hashes[index] != hashlib.sha256(sources[path].encode()).hexdigest():
        raise ValueError('historical body identity mismatch')

candidate = doc['candidate']
entries = baseline['newly_assessed_ledger']['entries']
matches = [item for item in entries if item['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-CHARACTER-RENAME-95E.json' or
    progress['new_current_ledger_candidates'] != len(entries) or
    candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
    not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate or ledger mismatch')
print('verified 12 Git-bound sources, pinned native archive and qualified rename candidate')
