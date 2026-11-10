"""Verify objective admission evidence and prior receipts, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-OBJECTIVE-EXPRESSION-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 3 and doc['source_revision'] == revision

def blob(raw):
    return hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()

def preserved(item):
    p = (root / item['preserved_bytes']).resolve()
    assert p.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(p.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    assert blob(raw) == item['git_blob']
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
assert len(sources) == 9
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/strategic/'
case = sources[core + 'case.rs']
must(case, ['#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]\npub struct ObjectiveExpression {',
 'pub alternatives: Vec<ObjectivePath>,', 'pub objectives: Vec<Objective>,',
 'pub fn new(alternatives: Vec<ObjectivePath>) -> Result<Self, ValidationError>',
 'if alternatives.is_empty()', 'if path.objectives.is_empty()',
 'if alternatives.len() > MAX_PATHS', '(leaves <= MAX_LEAVES)',
 'if !ids.insert(&objective.id)', 'ObjectiveRequirement::Defeat { count: 0, .. }',
 '.any(|path| path.iter().all(|p| p.state == EvaluationState::Satisfied))'])
assert 'try_from' not in case.split('pub struct ObjectiveExpression {', 1)[0].split('#[derive', 1)[-1]
# Logical properties of empty collections; this does not execute Rust product code.
assert all([]) and any([all([])])
dev = sources[core + 'developer_quest.rs']
must(dev, ['pub objectives: ObjectiveExpression,', 'serde_json::from_str(json_text)',
 'validate_recursive_bounds(definition, &mut diagnostics);',
 'for path in &mut materialized.objectives.alternatives {',
 'for objective in &mut path.objectives {', 'objectives: materialized.objectives,',
 'if let Err(errors) = qg::validate(&generated)'])
validation = sources[core + 'quest_generation/validation.rs']
must(validation, ['pub fn validate(case: &GeneratedCase)', 'validate_track_trails(case, &mut errors);',
 '.flat_map(|path| &path.objectives)', '"objective {} has no concrete owning producer"'])
for text in (dev, validation):
    assert 'ObjectiveExpression::new(' not in text
    assert 'alternatives.is_empty()' not in text and 'objectives.is_empty()' not in text
must(sources[core + 'quest_generation/model.rs'], ['pub objectives: ObjectiveExpression,'])
must(sources[core + 'quest_generation/assembly.rs'], ['ObjectiveExpression::new(paths)',
 'objectives: ObjectiveExpression {', 'objectives: vec![objective],'])
authority = sources[module + 'authority_model.rs']
must(authority, ['pub objective_expression_json: String,', 'qg::validate(&manifest)',
 'adventuresim_core::developer_quest::compile(developer)', 'if regenerated != manifest'])
boot = sources[module + 'mission_bootstrap.rs']
must(boot, ['require_development_gateway(ctx)?;', 'require_strategic_character_authority(ctx, character_id)?;',
 'dq::parse_definition_json(&definition_json)', 'let generated = dq::compile(&developer_context)',
 'objective_expression_json: serde_json::to_string(&generated.objectives)',
 'count: required_enemy_kills.max(1),'])
custody = sources[module + 'custody_objectives.rs']
must(custody, ['let objectives: adventuresim_core::case::ObjectiveExpression =',
 'if objectives != validated.manifest.objectives',
 'serde_json::from_str(&case.objective_expression_json)',
 'let evaluation = expression.evaluate(&core_case_id, party_id, &facts);',
 'case.resolution_status =', 'CaseStatus::Resolved', 'ContractStatus::ReadyToReport'])
web = sources['crates/strategic-web/src/routes/developer_quests.rs']
must(web, ['if !development_enabled(&state).await',
 'let definition: DeveloperQuestDefinition = match serde_json::from_value(request.definition)',
 'if let Err(diagnostics) = dq::compile(&preview)', '"spawn_developer_quest",'])

native = doc['native_source']
manifest = json.loads((root / native['package_manifest']).read_text())
archive = (root / native['package_manifest']).parent / manifest['archive']
assert hashlib.sha256(archive.read_bytes()).hexdigest() == manifest['sha256']
entry = next(x for x in manifest['inspected_members'] if x['path'] == native['archive_member'])
with tarfile.open(archive) as tar:
    raw = tar.extractfile(native['archive_member']).read()
assert len(raw) == native['bytes'] == entry['bytes']
assert hashlib.sha256(raw).hexdigest() == native['sha256'] == entry['sha256']
serde = raw.decode()
must(serde, ['pub(super) fn deserialize(', 'quote!(#this_value)',
 'let mut result = quote!(#struct_path { #(#result),* });', '_serde::#private::Ok(#result)'])
assert all(1 <= a <= b <= len(raw.splitlines()) for a, b in native['ranges'])

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
prior_receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw)
    assert entry['git_blob'] == item['git_blob']
    old = json.loads(raw)
    assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0
    assert old['current_source_reconciled'] is False
    prior_receipts[item['original_path']] = item
assert len(prior_receipts) == 2
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
delegated = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
for index, n in ((244, 142), (248, 145)):
    item = prior_receipts[f'segovb/READ-L13-PARALLEL-{n}.json']
    assert delegated[index]['receipt'] == item['original_path']
    assert delegated[index]['receipt_sha256'] == item['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
ref = doc['historical_source_roles'][0]
role = roles[ref['original_index']]
assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
assert role['historical_assessment'] == ref['historical_assessment']
for i in range(887, 891):
    assert roles[i]['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-887-890.json'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name and x['lens'] == 3
               for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert base['lens_status'] == ['incomplete'] * 13
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified nine Git-bound sources, native serde source and two prior receipts; no product ran')
