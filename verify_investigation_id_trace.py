"""Verify investigation ID source evidence and prior attribution without running the product."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-INVESTIGATION-ID-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 3 and doc['source_revision'] == revision

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
assert len(sources) == 3
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
investigation = sources[core + 'investigation.rs']
macro = investigation.split('macro_rules! stable_id {', 1)[1].split('stable_id!(CaseId);', 1)[0]
must(macro, ['Serialize, Deserialize', 'pub struct $name(String)',
 'if value.is_empty()', 'value.len() > 256', 'b.is_ascii_alphanumeric()',
 "matches!(b, b'-' | b'_' | b':' | b'.')", 'Err(ValidationError::InvalidId)'])
assert 'try_from' not in macro and 'deserialize_with' not in macro
must(investigation, ['stable_id!(CaseId);', 'stable_id!(EventId);',
 'stable_id!(PropositionId);', 'pub struct PipelineInput {', 'pub case_id: CaseId',
 'pub event_id: EventId', 'pub proposition: AtomicProposition',
 'pub fn process_report(', 'case_id: input.case_id,', 'pub fn compound_id('])
projection = sources[core + 'quest_generation/projection.rs']
must(projection, ['pub fn generated_testimony_pipeline(',
 'case_id: CaseId::new(&generated.canonical_case_id)?',
 'event_id: EventId::new(', 'PropositionId::new(draft.proposition_id.clone())?'])
claims = sources[module + 'investigation/claims.rs']
must(claims, ['pub(crate) fn stage_investigation_claim(', 'require_actor(ctx, character_id)?;',
 'if pipeline_json.len() > 8_192', 'serde_json::from_str(&pipeline_json)',
 'process_investigation_pipeline(pipeline)?', 'case_id: claim.case_id.as_str().into()',
 'pub(crate) fn persist_generated_testimony(', 'validate_quest_generation_authority(&authority)?',
 'validated_authority.manifest != *generated', 'generated_testimony_pipeline(',
 'serde_json::to_string(&pipeline)'])

for native in doc['native_sources']:
    manifest_path = root / native['package_manifest']
    manifest = json.loads(manifest_path.read_text())
    archive = manifest_path.parent / manifest['archive']
    raw_archive = archive.read_bytes()
    assert hashlib.sha256(raw_archive).hexdigest() == manifest['sha256']
    with tarfile.open(archive) as ar:
        raw = ar.extractfile(native['archive_member']).read()
        assert hashlib.sha256(raw).hexdigest() == native['sha256'] and len(raw) == native['bytes']
        assert all(1 <= a <= b <= len(raw.splitlines()) for a, b in native['ranges'])
        if 'entry_member' in native:
            entry = ar.extractfile(native['entry_member']).read().decode()
            must(entry, ['Data::Struct(Style::Tuple, fields) | Data::Struct(Style::Newtype, fields)',
                         'tuple::deserialize(params, fields, &cont.attrs, TupleForm::Tuple)'])
        body = raw.decode()
        if 'tuple.rs' in native['archive_member']:
            must(body, ['Some(deserialize_newtype_struct(&type_path, params, &fields[0]))',
                        'let mut result = quote!(#type_path(__field0));',
                        'let __field0: #field_ty = #value;'])
        else:
            must(body, ['fn deserialize_newtype_struct<V>', 'visitor.visit_newtype_struct(self)',
                        "pub fn from_str<'a, T>"])

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
item = doc['recovered_historical_receipts'][0]
raw = preserved(item)
entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
old = json.loads(raw)
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][279]
assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
assert prior['contract']['proposed_id'] == 'PL13-172-INVESTIGATION-NUMERIC-ADAPTER-AND-NATIVE-PENDING'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
ref = doc['historical_source_roles'][0]
role = roles[ref['original_index']]
assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified three Git-bound sources, pinned decoding source and prior receipt; no product ran')
