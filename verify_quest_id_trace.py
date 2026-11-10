"""Verify preserved quest-ID source evidence, without running product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-QUEST-ID-95E.json').read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert doc['source_revision'] == revision

def tree(name):
    entries = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in entries:
            raise ValueError('duplicate tree path')
        entries[path] = oid
    return entries

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for source in doc['current_sources']:
    saved = (root / source['preserved_bytes']).resolve()
    if not saved.is_relative_to(root) or source['revision'] != revision or source['content_encoding'] != 'gzip':
        raise ValueError('invalid source reference')
    raw = gzip.decompress(saved.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines'] or
            hashlib.sha256(raw).hexdigest() != source['sha256'] or oid != source['git_blob'] or
            current.get(source['path']) != oid or previous.get(source['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + source['path'])
    for first, last in source['ranges']:
        if not 1 <= first <= last <= source['lines']:
            raise ValueError('invalid read range')
    if source['path'] in sources:
        raise ValueError('duplicate source')
    sources[source['path']] = raw.decode()
if len(sources) != 7:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-core/src/quest_generation/model.rs': [
        'pub struct $name(pub String);', 'pub fn try_new(value: impl Into<String>)',
        'pub const fn from_static(value: &str)', 'assert!(!source.is_empty() && source.len() <= 63)',
        'byte.is_ascii_lowercase() || byte.is_ascii_digit()', 'Self::try_new(&value).map_err',
        'serializer.serialize_str(self.as_str())', 'id_type!(TrackSegmentId);'],
    'crates/adventuresim-core/src/quest_generation/assembly.rs': [
        'SiteId::new(scoped_id(', 'validate(&manifest).map_err'],
    'crates/adventuresim-core/src/quest_generation/solver.rs': [
        'SiteKind::try_new(&authored_site.id)', 'Circumstance::try_new(&authored.id)'],
    'crates/adventuresim-core/src/quest_generation/validation.rs': [
        'pub fn validate(case: &GeneratedCase)', 'validate_track_trails(case, &mut errors)'],
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'serde_json::from_str(&authority.manifest_json)', 'qg::validate(&manifest)',
        'if regenerated != manifest'],
    'crates/adventuresim-stdb-module/src/investigation/projections.rs': [
        'validate_quest_generation_authority(&authority)', 'generated_authority_view('],
    'crates/adventuresim-stdb-module/src/investigation/capabilities.rs': [
        'generated_authority_reducer(ctx, root)', 'serde_json::from_str::<adventuresim_core::quest_generation::GeneratedCase>'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)

for evidence_key in ('native_source', 'serde_json_source'):
    ev = doc[evidence_key]
    manifest_path = (root / ev['package_manifest']).resolve()
    if not manifest_path.is_relative_to(root):
        raise ValueError('unsafe archive path')
    manifest = json.loads(manifest_path.read_text())
    archive = manifest_path.parent / manifest['archive']
    archive_raw = archive.read_bytes()
    if len(archive_raw) != manifest['bytes'] or hashlib.sha256(archive_raw).hexdigest() != manifest['sha256']:
        raise ValueError('archive mismatch')
    with tarfile.open(archive, 'r:gz') as tar:
        raw = tar.extractfile(ev['archive_member']).read()
        if 'entry_member' in ev:
            entry = tar.extractfile(ev['entry_member']).read().decode()
            if 'Data::Struct(Style::Newtype, fields)' not in entry or 'tuple::deserialize(' not in entry:
                raise ValueError('derive dispatch missing')
    if len(raw) != ev['bytes'] or hashlib.sha256(raw).hexdigest() != ev['sha256']:
        raise ValueError('native member mismatch')
    if not any(x['path'] == ev['archive_member'] and x['sha256'] == ev['sha256'] for x in manifest['inspected_members']):
        raise ValueError('native member not in evidence manifest')
    for first, last in ev['ranges']:
        if not 1 <= first <= last <= len(raw.splitlines()):
            raise ValueError('invalid native read range')
    content = raw.decode()
    if evidence_key == 'native_source':
        if 'quote!(#type_path(__field0))' not in content or 'deserialize_newtype_struct' not in content:
            raise ValueError('newtype construction marker missing')
    elif 'visitor.visit_newtype_struct(self)' not in content or 'pub fn from_str' not in content:
        raise ValueError('JSON visitor marker missing')

entries = baseline['newly_assessed_ledger']['entries']
if len(doc['candidates']) != 2 or progress['new_current_ledger_candidates'] != len(entries):
    raise ValueError('ledger count mismatch')
for candidate in doc['candidates']:
    matches = [entry for entry in entries if entry['id'] == candidate['id']]
    if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-QUEST-ID-95E.json' or
            candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
            not candidate['discovery'].startswith('partial')):
        raise ValueError('unqualified candidate')
historical = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-131.json').read_text())
if historical['historical_source_roles'][0]['sha256'] != hashlib.sha256(sources['crates/adventuresim-core/src/quest_generation/model.rs'].encode()).hexdigest():
    raise ValueError('historical source identity mismatch')
print('verified seven Git-bound source snapshots, two native archives and qualified candidate links')
