"""Verify custody/dialogue evidence, source conditions and historical limits."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-CUSTODY-DIALOGUE-OBJECTIVES-95E.json'
doc = load(name)
ids = ['SEGOVB-L12-CUSTODY-OBJECT-PRIVATE-READMISSION',
       'SEGOVB-L12-DIALOGUE-OBJECTIVE-PRIVATE-KEY',
       'REVIEW-CUSTODY-RETRY-FACT-PAYLOAD-001',
       'REVIEW-DEVELOPER-OBJECTIVE-OPTIONS-001']
assert doc['source_revision'] == rev and doc['lenses'] == [3,12]
assert [x['id'] for x in doc['candidates']] == ids
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1); tree[path] = meta.split()[2]
contents = {}
for row in doc['current_sources']:
    raw = gzip.decompress((root / row['preserved_bytes']).read_bytes())
    assert sha(raw) == row['sha256'] and blob(raw) == row['git_blob'] == tree[row['path']]
    assert len(raw) == row['bytes'] and len(raw.splitlines()) == row['lines']
    assert row['revision'] == rev and all(1 <= lo <= hi <= row['lines'] for lo,hi in row['ranges'])
    contents[row['path']] = raw
custody = contents['crates/adventuresim-stdb-module/src/strategic/custody_objectives.rs']
seed = custody.split(b'fn seed_case_custody(',1)[1].split(b'fn party_strategic_minute(',1)[0]
assert b'asset_id.as_str()' in seed and b'subject_id.as_str()' in seed
assert b'kinds.insert(object_id.to_owned(), kind)' in seed and b'transition_case_custody(' in seed
transition = custody.split(b'fn transition_case_custody(',1)[1].split(b'fn seed_case_custody(',1)[0]
assert b'let object = custody_object(object_kind, object_id)?' in transition
assert b'validate_custody_fact_retry_attribution(' in transition
retry = transition.split(b'if let Some(existing)',1)[1].split(b'let party_key',1)[0]
assert b'existing.holder_id == holder_id' in retry and b'existing.version == version' in retry
assert b'receipt.case_id.as_str(), receipt.party_id.as_str()' in retry
assert b'fact_json' not in retry and b'Ok(false)' in retry
adapter = custody.split(b'pub(crate) fn record_asset_returned_or_exchanged(',1)[1].split(b'pub(crate) fn record_subject_rescued_or_released(',1)[0]
assert b'if exchange' in adapter and b'OutcomeFactKind::AssetExchanged' in adapter
assert b'OutcomeFactKind::AssetReturned' in adapter and b'CustodyHolderKind::Npc' in adapter
case = contents['crates/adventuresim-core/src/case.rs']
assert b'id!(AssetId, "asset:")' in case and b'id!(SubjectId, "subject:")' in case
assert b'pub enum CustodyObject' in case and b'pub fn apply_custody(' in case
bindings = contents['crates/adventuresim-stdb-module/src/strategic/dialogue_bindings.rs']
assert b'objective_id: &str' in bindings and b'producer.objective_id.as_str() == objective_id' in bindings
assert b'objective.id.as_str(),' in bindings and b'objective.id.as_str().to_string(),' in bindings
schema = contents['crates/adventuresim-stdb-module/src/strategic/dialogue_schema.rs']
assert b'pub struct DialogueInvestigationBinding' in schema and b'pub objective_id: String' in schema
effects = contents['crates/adventuresim-stdb-module/src/strategic/dialogue_effects.rs']
assert b'binding.action_family != dialogue_investigation_action_id(action)' in effects
assert b'|| !binding.consumed_by.is_empty()' in effects and b'current.version) != binding.expected_custody_version' in effects
assert b'R::Return {' in effects and b'R::Exchange {' in effects
model = contents['crates/adventuresim-core/src/quest_generation/model.rs']
assert b'pub struct GeneratedDialogueProducer' in model and b'pub objective_id: ObjectiveId' in model
assert b'pub enum GeneratedDialogueAction {\n    Expose,\n    ReturnAsset,\n}' in model
developer = contents['crates/adventuresim-core/src/developer_quest.rs']
assert b'"objective_requirements":' in developer and b'"PresentProof"' in developer
assert b'if let Err(errors) = qg::validate(&generated)' in developer
validation = contents['crates/adventuresim-core/src/quest_generation/validation.rs']
objective = validation.split(b'for objective in case\n        .objectives',1)[1].split(b'let expected_finale',1)[0]
assert b'ObjectiveRequirement::PresentProof' not in objective
assert b'_ => false,' in objective and b'has no concrete owning producer' in objective
web = contents['crates/strategic-web/src/routes/developer_quests.rs']
assert b'let mut schema = dq::schema_json(' in web and b'if let Err(diagnostics) = dq::compile(&preview)' in web
editor = contents['crates/strategic-web/static/developer-quest-editor.js']
assert b'for (const optionValue of schema.options.objective_requirements)' in editor
assert b'schema.constructors.variants.objective_requirement[select.value]' in editor

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records=doc['recovered_historical_records']; assert len(records)==28
by_path={}
for row in records:
    raw=gzip.decompress((root / row['preserved_bytes']).read_bytes()); original=transport[row['original_path']]
    assert row['sha256']==original['sha256'] and row['git_blob']==original['git_blob']
    assert row['bytes']==original['bytes'] and sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if not row['redacted_local_path_occurrences']:
        assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    else:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    by_path[row['original_path']]=raw
for number, indices in ((287,[1208,1209]),(288,[1210,1211]),(289,[1212]),(290,[1213]),(291,[1214])):
    name059=f'segovb/READ-L12-{number}.json'; receipt=json.loads(by_path[name059])
    assert receipt['lens']==12 and [r['original_index'] for r in receipt['rows']]==indices
    assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
    assert receipt['implementation_gate_complete'] is False
    assert sha(by_path[receipt['plan']])==receipt['plan_sha256']
    for row in receipt['rows']:
        assert sha(by_path[row['registration']])==row['registration_sha256']
        assert row['fresh_personal_body_reads']==row['assessed_ranges'] and row['other_lens_verdict_transfer'] is False
    check=json.loads(by_path[f'segovb/VERIFY-L12-{number}.json'])
    assert check['receipt_sha256']==sha(by_path[name059])
    assert check['state_sha256']==sha(by_path[f'segovb/STATE-L12-{number}.json'])
row=json.loads(by_path['segovb/READ-L12-291.json'])['rows'][0]
assert row['assessed_ranges']==[[1,13]]
locator=row['function_contract']['source_locator_qualification']
assert locator['actual_EOF']==14 and locator['unassessed']==[[14,14]]
assert locator['historical_locator_proof_read'] is False
assert sha(by_path[locator['path']])==locator['sha256']

histname='ASSESSMENT-L01-HISTORICAL-1209-1214.json'
history=load(histname)['historical_source_roles']
assert [r['original_index'] for r in history]==list(range(1209,1215))
roles=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read()
        assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']]
        assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
assert history[-1]['lines']==14
q=load('RECONCILIATION-059.json')['entries']
for index, contract in [(229,ids[0]),(230,ids[1]),(978,'Q-L12-fba0b8a6504c5caf8eb8'),(980,'Q-L12-9e75bc934bd0149f11ca'),(981,'Q-L12-c18648844bc0927c68da'),(982,'Q-L12-5f12eb046c5eaf28a2d5')]:
    assert q[index]['contract_id']==contract and q[index]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches=[r for r in ledger if r['id']==identity]; assert len(matches)==1 and matches[0]['record']==name
progress=load('CONTINUATION-PROGRESS-95E.json')
assert name in progress['post_transition_bounded_string_traces'] and name in progress['post_transition_bounded_admission_traces']
assert progress['lenses_complete']==0 and progress['queue_records_semantically_reconciled']==0
assert progress['new_current_ledger_candidates']==len(ledger)
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified custody/dialogue trace and earlier locator limit; no product ran')
