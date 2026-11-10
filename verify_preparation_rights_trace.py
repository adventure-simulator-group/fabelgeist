"""Verify bounded preparation source evidence and prior attribution, without product execution."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-PREPARATION-RIGHTS-95E.json'
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
assert len(sources) == 7
core = 'crates/adventuresim-core/src/'
module = 'crates/adventuresim-stdb-module/src/'
herbalism = sources[core + 'herbalism.rs']
must(herbalism, ['pub fn preparation_rights_question(', 'RightsSubject::Character(actor)',
 'RightsResource::Object(object_id)', 'RightsOperation::Alter', 'RightsJurisdiction::Place(place)',
 'question_digest: preparation_rights_question_digest(question)',
 'evidence_revision: RightsRevision(revision)', 'pub struct PreparationPlanAuthority {',
 'pub rights: PrivateRightsDecision<()>', 'authority.rights.kind() == RightsDecisionKind::Allowed',
 'if time.permits_completion_effects()', 'PreparationPlanEffect::CommitPreparation'])
requirements = herbalism.split('pub fn build_preparation_plan(', 1)[1].split('.into_iter()', 1)[0]
assert 'authority.rights.kind()' in requirements and '.provenance()' not in requirements
assert 'question_digest' not in requirements
rights = sources[core + 'rights.rs']
must(rights, ['pub struct DecisionProvenance {', 'pub evidence_revision: RightsRevision',
 'pub question_digest: [u8; 32]', 'pub struct PrivateRightsDecision',
 'pub const fn provenance(&self)', 'RightsResource::Object(object) =>',
 'self.frame(&object.get().to_le_bytes())'])
action = sources[core + 'strategic_action.rs']
must(action, ['pub struct ActionCoordinates<', 'pub const fn target(&self)',
 'pub fn build_plan<', 'filter(|check| !check.satisfied)',
 'TimeOutcome::Completed', 'pub fn validate_commit<',
 'planned.coordinates != replanned.coordinates',
 'planned.requirements != replanned.requirements',
 'planned.calculation != replanned.calculation'])
forage = sources[core + 'foraging.rs']
must(forage, ['pub struct ForageLicenseDecision {', 'pub fn try_new(',
 'decision.provenance().question_digest != forage_license_question_digest(question)'])
adapter = sources[module + 'food/preparation_authority.rs']
must(adapter, ['lot.material_revision != expected_revision',
 'lot.inventory_item_id == Some(inventory_item_id)',
 'object.id != material_object_id', 'require_actor_carried_object(ctx, actor, &object)?',
 'let coordinates = ActionCoordinates::try_new(',
 'herbalism::preparation_rights_question(actor_id, object_id, authority.place.clone())',
 'herbalism::decide_preparation_rights(', 'authority.lot.material_revision,',
 'herbalism::build_preparation_plan(', 'frame(&object.id.to_le_bytes())',
 'frame(&lot.material_revision.to_le_bytes())'])
must(sources[module + 'object_custody.rs'], ['matches_carried_inventory(actor.id, actor.party_id.as_deref())'])
reducer = sources[module + 'food/ingredient_preparation.rs']
must(reducer, ['pub fn prepare_ingredient_lot(', 'if request_id != canonical_request',
 'let fresh = load_preparation_authority(', 'let replanned = build_preparation_planner(',
 'strategic_action::validate_commit(', 'material::validate_material_commit(',
 'let post = load_preparation_authority(', 'ctx.db.food_lot().id().update(lot)'])

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
item = doc['recovered_historical_receipts'][0]
raw = preserved(item)
entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
old = json.loads(raw)
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][277]
assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
assert prior['contract']['proposed_id'] == 'PL13-170-MATERIAL-PREPARATION-CONTAINMENT-NATIVE-PENDING'
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
print('verified seven Git-bound sources and separately attributed prior receipt; no product ran')
