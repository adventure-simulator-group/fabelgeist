"""Verify preserved source and provenance for the planned-exposure audit only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-PLANNED-EXPOSURE-95E.json'
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
if len(sources) != 16:
    raise ValueError('incorrect source count')

adv = sources['crates/adventuresim-stdb-module/src/time/advancement.rs']
ordinary_move = adv.split('pub fn advance_character_time(', 1)[1].split('fn advance_character_time_in_plan(', 1)[0]
planned_move = adv.split('fn advance_character_time_in_plan(', 1)[1].split('/// Actual strategic movement', 1)[0]
ordinary_wait = adv.split('pub fn advance_character_wait_time(', 1)[1].split('pub fn advance_character_wait_time_in_plan(', 1)[0]
planned_wait = adv.split('pub fn advance_character_wait_time_in_plan(', 1)[1]
for body in (ordinary_move, ordinary_wait):
    if body.count('apply_weather_exposure(') != 1 or '.saturating_add_minutes(' not in body:
        raise ValueError('ordinary weather or clock marker missing')
for body in (planned_move, planned_wait):
    if 'apply_weather_exposure(' in body or '.saturating_add_minutes(' not in body:
        raise ValueError('planned weather or clock marker changed')
if 'advance_character_time_in_plan(ctx, character_id, chunk, plan)?' not in adv:
    raise ValueError('planned travel wrapper marker missing')

journey = sources['crates/adventuresim-stdb-module/src/strategic/journey_camp.rs']
move = journey.split('fn advance_party_movement(', 1)[1].split('pub(crate) fn advance_party_wilderness_elapsed(', 1)[0]
for marker in ('plan_party_disease_interval(', 'preview_travel_time_in_plan(',
               'advance_travel_time_in_plan(', 'advance_party_wilderness_elapsed('):
    if marker not in move:
        raise ValueError('party movement marker missing')
if 'apply_weather_exposure(' in move:
    raise ValueError('party movement now applies weather explicitly')
encounters = sources['crates/adventuresim-stdb-module/src/strategic/encounters.rs']
if 'advance_party_movement(ctx, party_id, traveler_ids, requested_leg_minutes)?' not in encounters:
    raise ValueError('encounter movement caller marker missing')
travel = sources['crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs']
if travel.count('advance_party_movement_until_encounter(') < 3:
    raise ValueError('party journey entry points changed')

surgery = sources['crates/adventuresim-stdb-module/src/surgery.rs']
align = surgery.split('fn align_and_advance(', 1)[1].split('fn duration_minutes(', 1)[0]
treat = surgery.split('pub fn treat_limb(', 1)[1].split('#[cfg(test)]', 1)[0]
for marker in ('plan_party_disease_interval(', 'advance_character_wait_time_in_plan(',
               'advance_character_wait_time(ctx, id, catchup)?'):
    if marker not in align:
        raise ValueError('treatment alignment marker missing')
for marker in ('require_strategic_character_authority(ctx, actor_id)?',
               'SurgeryProcedure::Bandage if injury.cut_damage > 0.0 && !injury.bandaged => 0.0',
               'if !align_and_advance(ctx, actor_id, patient_id, duration)?',
               'treatment_action_receipt()'):
    if marker not in treat:
        raise ValueError('treatment reducer marker missing')
if 'apply_weather_exposure(' in align or 'apply_weather_exposure(' in treat:
    raise ValueError('treatment now applies weather explicitly')
core_surgery = sources['crates/adventuresim-core/src/surgery.rs']
for marker in ('SurgeryProcedure::Bandage => 30.0', '(base - skill.max(0.0) * 5.0).max(10.0).ceil() as u64'):
    if marker not in core_surgery:
        raise ValueError('procedure duration marker missing')
actor = sources['crates/adventuresim-stdb-module/src/world_actor.rs']
core_action = sources['crates/adventuresim-core/src/strategic_action.rs']
if 'if actor_id == patient_id {' not in actor or 'ContextualActionDecision::Allowed(ContextualActionReason::SelfAction)' not in core_action:
    raise ValueError('self action marker missing')
medical = sources['crates/strategic-web/src/routes/settlements/medical.rs']
if '"treat_limb"' not in medical or 'json!(patient_id)' not in medical:
    raise ValueError('settlement treatment route marker missing')

condition = sources['crates/adventuresim-stdb-module/src/condition.rs']
for marker in ('pub fn apply_weather_exposure(', 'survival::advance_exposure(',
               'ctx.db\n        .character_exposure()', 'thermal_incapacitation(exposure.thermal_strain)'):
    if marker not in condition:
        raise ValueError('exposure owner or projection marker missing')
survival = sources['crates/adventuresim-core/src/survival.rs']
for marker in ('if shelter == ExposureShelter::Indoor {', 'return strain - strain.signum();',
               'pub const COLD_STAGGER_STRAIN: i32 = -5_000;',
               'pub const COLD_INCAPACITATION_STRAIN: i32 = -9_000;',
               'state.frostbite_progress_minutes = state.frostbite_progress_minutes.saturating_add(1);'):
    if marker not in survival:
        raise ValueError('survival state marker missing')
camp = sources['crates/adventuresim-stdb-module/src/time/camp_rest.rs']
if 'apply_weather_exposure(' not in camp or 'ExposureShelter::Field(shelter)' not in camp:
    raise ValueError('separate camp exposure marker missing')

# Inspect the textual assertion's selected ranges; this is not a product test.
test_source = sources['crates/adventuresim-stdb-module/src/time/tests/clock_and_advancement.rs']
time_mod = sources['crates/adventuresim-stdb-module/src/time/mod.rs']
schedule = sources['crates/adventuresim-stdb-module/src/time/schedule.rs']
if ('fn every_authoritative_clock_commit_has_one_exposure_application()' not in test_source or
    'body.matches("apply_weather_exposure(").count()' not in test_source or
    time_mod.index('include_str!("advancement.rs")') > time_mod.index('include_str!("schedule.rs")')):
    raise ValueError('textual assertion composition changed')
combined = adv + schedule
travel_selection = combined.split('pub fn advance_character_time', 1)[1].split('pub fn preview_travel_time', 1)[0]
wait_selection = combined.split('pub fn advance_character_wait_time', 1)[1].split('fn default_schedule', 1)[0]
if (travel_selection.count('apply_weather_exposure(') != 1 or 'fn advance_character_time_in_plan(' not in travel_selection or
    wait_selection.count('apply_weather_exposure(') != 1 or 'pub fn advance_character_wait_time_in_plan(' not in wait_selection):
    raise ValueError('textual assertion scope changed')

historical = doc['historical_source']
queue = json.loads((root / historical['queue']).read_text())
questions = {item['contract_id']: item for item in queue['entries']}
if historical['checkpoint'] != '059' or len(historical['contract_ids']) != 3:
    raise ValueError('incorrect historical scope')
for identity in historical['contract_ids']:
    if questions[identity]['status'] != 'open-evidence-or-disposition' or questions[identity]['body_or_population_credit'] != 0:
        raise ValueError('historical question status changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
identities = [(historical['matching_advancement_role_index'], historical['matching_advancement_sha256']),
              (historical['different_physical_p16_role_index'], historical['different_physical_p16_sha256']),
              (historical['matching_journey_role_index'], historical['matching_journey_sha256']),
              (historical['matching_surgery_role_index'], historical['matching_surgery_sha256'])]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index, sha in identities:
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
if (historical['matching_advancement_sha256'] != hashlib.sha256(adv.encode()).hexdigest() or
    historical['matching_journey_sha256'] != hashlib.sha256(journey.encode()).hexdigest() or
    historical['matching_surgery_sha256'] != hashlib.sha256(surgery.encode()).hexdigest() or
    historical['different_physical_p16_sha256'] == historical['matching_advancement_sha256']):
    raise ValueError('historical current/different distinction changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [item for item in entries if item['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
    candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified sixteen Git-bound source files and separately attributed planned-weather questions')
