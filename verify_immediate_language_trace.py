"""Verify preserved source and provenance for the immediate-language audit only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-IMMEDIATE-LANGUAGE-95E.json'
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
if len(sources) != 14:
    raise ValueError('incorrect source count')

activities = sources['crates/adventuresim-stdb-module/src/time/activities.rs']
action = activities.split('pub fn perform_immediate_activity(', 1)[1].split('const ACTIVITY_MINUTE_SCALE', 1)[0]
for marker in ('require_strategic_character_authority(ctx, character_id)?',
               '!(60..=MINUTES_PER_DAY).contains(&requested_minutes)',
               'let elapsed = settled.elapsed;', 'character_time.minutes = interval_end;',
               'immediate_activity_schedule(activity, effective_minutes, organization_id)',
               'ctx.db.character_skills().character_id().update(skills);'):
    if marker not in action:
        raise ValueError('immediate activity marker missing')
training_call = action.split('let excess = apply_training(', 1)[1].split(')?;', 1)[0]
if '&effective_schedule' not in training_call or 'MINUTES_PER_DAY' not in training_call:
    raise ValueError('immediate training basis changed')
schedule = sources['crates/adventuresim-stdb-module/src/time/schedule.rs']
training = schedule.split('fn apply_training(', 1)[1].split('fn total_socializing_receipt_minutes(', 1)[0]
for marker in ('elapsed: u64', 'let exposure = elapsed as f32 / 60.0 * (2.0 / 3.0);',
               'settlement.languages.east_central_bp', 'skills.oral_languages.direct_mut(language)',
               'attributes.instinct', 'apply_schedule_training(', 'apply_religion_training('):
    if marker not in training:
        raise ValueError('shared training or ambient marker missing')
skill = sources['crates/adventuresim-core/src/skill.rs']
for marker in ('(1.0 + 0.5 * (attribute.clamp(0.0, MAX_CHECK) - 2.5)).max(0.0)',
               'let gain = real_hours * aptitude_training_multiplier(aptitude);',
               'gain.min((aptitude * 1_000.0 - current).max(0.0))',
               '*stored_effective_hours = current + accepted;'):
    if marker not in skill:
        raise ValueError('language gain or cap marker missing')
world = sources['crates/adventuresim-world-schema/src/language.rs']
initial = sources['crates/adventuresim-world-schema/src/language/initialization.rs']
if 'pub struct OralLanguageHours' not in world or 'let mut hours = OralLanguageHours::default();' not in initial or '*hours.direct_mut(german)' not in initial:
    raise ValueError('language initialization marker missing')
bootstrap = sources['crates/adventuresim-stdb-module/src/strategic/mission_bootstrap.rs']
if '"riverdale",' not in bootstrap or '_ => adventuresim_world_schema::SettlementLanguageProfile {' not in bootstrap or 'east_central_bp: 1_000,' not in bootstrap:
    raise ValueError('demo language profile marker missing')
character = sources['crates/adventuresim-stdb-module/src/character.rs']
for marker in ('instinct: generated_attributes.map_or(2.0, |a| a.instinct)',
               'initial_character_languages(start_settlement.languages, id, npc)',
               'let persisted_oral_languages = life_skills.map_or(oral_languages, |simulated| simulated.oral);',
               'oral_languages: persisted_oral_languages'):
    if marker not in character:
        raise ValueError('character initialization qualification marker missing')
route = sources['crates/strategic-web/src/routes/settlements/party/training_activity.rs']
if '"perform_immediate_activity"' not in route or 'session.character_id_u64() != Some(character_id)' not in route:
    raise ValueError('browser route marker missing')
calendar = sources['crates/adventuresim-world-schema/src/calendar.rs']
core_time = sources['crates/adventuresim-core/src/strategic_time.rs']
if 'pub const MINUTES_PER_DAY: u64 = 24 * 60;' not in calendar or 'days * hours_per_day' not in core_time:
    raise ValueError('daily normalization marker missing')
core_schedule = sources['crates/adventuresim-core/src/strategic_schedule.rs']
if 'training_hours_increment(elapsed_minutes, minutes)' not in core_schedule:
    raise ValueError('schedule training marker missing')
for path, marker in [
    ('crates/adventuresim-stdb-module/src/time/settlement_rest.rs', 'let training_elapsed = elapsed;'),
    ('crates/adventuresim-stdb-module/src/time/stationary.rs', 'let training_elapsed = elapsed.saturating_sub(convalescing);'),
    ('crates/adventuresim-stdb-module/src/time/camp_rest.rs', 'apply_training(ctx, member_id, &mut skills, &allowed, downtime, activities)?'),
]:
    if marker not in sources[path]:
        raise ValueError('other training caller marker missing')

historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies']
if historical['checkpoint'] != '059' or len(historical['contract_ids']) != 3:
    raise ValueError('incorrect historical scope')
for identity in historical['contract_ids']:
    if old[identity]['accepted_unique_count_change'] != 0:
        raise ValueError('historical unique-count qualification changed')
segovb = old['SEGOVB-L12-COND-IMMEDIATE-ACTIVITY-AMBIENT-LANGUAGE-SPAN']
if segovb['current_reconciled'] is not False or segovb['runtime_validation'] != 'Not run' or segovb['source_sha256'] != historical['matching_schedule_sha256']:
    raise ValueError('historical source attribution changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
identities = [(historical['matching_activity_role_index'], historical['matching_activity_sha256']),
              (historical['matching_schedule_role_index'], historical['matching_schedule_sha256']),
              (historical['different_physical_activity_role_index'], historical['different_physical_activity_sha256']),
              (historical['different_physical_schedule_role_index'], historical['different_physical_schedule_sha256'])]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index, sha in identities:
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
if (historical['matching_activity_sha256'] != hashlib.sha256(activities.encode()).hexdigest() or
    historical['matching_schedule_sha256'] != hashlib.sha256(schedule.encode()).hexdigest() or
    historical['different_physical_activity_sha256'] == historical['matching_activity_sha256'] or
    historical['different_physical_schedule_sha256'] == historical['matching_schedule_sha256']):
    raise ValueError('historical current/different distinction changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [item for item in entries if item['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
    candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified fourteen Git-bound sources and the separately attributed immediate-language records')
