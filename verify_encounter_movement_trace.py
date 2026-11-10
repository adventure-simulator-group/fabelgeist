"""Verify pinned encounter-rule evidence and arithmetic, without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ENCOUNTER-MOVEMENT-95E.json'
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
if len(sources) != 12:
    raise ValueError('incorrect source count')
markers = {
    'AGENTS.md': ['Combat performance depends on total incapacitation, not fatigue directly.',
                  'Neither fatigue nor total incapacitation adds a movement-speed penalty',
                  'physical mass and limb effects remain separate.'],
    'crates/adventuresim-stdb-module/src/strategic/encounters.rs': [
        'fn current_party_fatigue_percent(', 'stats.calories_used.max(0.0) / capacity',
        'IncapacitationStatus::Ready => 1.0', 'IncapacitationStatus::Staggered => 0.5',
        'IncapacitationStatus::Incapacitated => 0.0',
        'body_burden + personal_burden + party_burden', 'party_encumbrance_remaining_basis_points(',
        'current_party_fatigue_percent(ctx, &member_ids)', 'encounter.party_speed_m_per_minute <= encounter.enemy_speed_m_per_minute'],
    'crates/adventuresim-core/src/encounter.rs': [
        'PARTY_WALKING_SPEED_M_PER_MINUTE: u32 = 83',
        'whole_bps.saturating_sub(u32::from(fatigue_percent) * 50)',
        'EncounterTerrain::Road => whole_bps', 'u64::from(fatigue)', 'u64::from(encumbrance)',
        '(scaled / u64::from(BASIS_POINTS_PER_WHOLE).pow(4)).max(1) as u32',
        'party_speed > archetype.enemy_speed_m_per_minute()',
        'awareness == Awareness::Both && run_is_eligible(party_speed, archetype)'],
    'crates/adventuresim-core/src/morale.rs': [
        'total if total >= 1.0 => IncapacitationStatus::Incapacitated',
        'total if total > 0.5 => IncapacitationStatus::Staggered',
        '((fatigue_ratio - 0.5).max(0.0) / 0.5).powi(2)'],
    'crates/adventuresim-core/src/essential.rs': [
        'CALORIES_PER_ENDURANCE: f32 = 1000.0', 'fn fatigue_by_parts(', 'self.calories_used_today()'],
    'crates/adventuresim-stdb-module/src/capability.rs': [
        'BodyPart::Chest => self.chest_health', 'fn calories_used_today(&self) -> f32', 'self.calories_used'],
    'crates/adventuresim-core/src/attribute.rs': [
        'Self::Endurance => BodyPart::Chest', 'let health = body.body_part_health(attr.body_part())', 'raw * health'],
    'crates/adventuresim-core/src/equipment.rs': [
        'LOWER_MUSCLE_MASS_PER_LEG_STRENGTH: f32 = 5.0',
        'WEIGHT_CAPACITY_PER_LOWER_MUSCLE_MASS: f32 = 30.0',
        '(1.0 - burden_kg / capacity_kg).clamp(0.0, 1.0)'],
    'crates/adventuresim-world-schema/src/basis_points.rs': [
        'BASIS_POINTS_PER_WHOLE: u16 = 10_000',
        'Self::new((value * f32::from(BASIS_POINTS_PER_WHOLE)).round() as u16)'],
    'crates/adventuresim-stdb-module/src/condition.rs': [
        'let fatigue_ratio = stats.fatigue_by_parts(&attributes, &limbs)',
        'fatigue: fatigue_incapacitation(fatigue_ratio)', 'stats.calories_used += elapsed_minutes as f32'],
    'crates/adventuresim-stdb-module/src/strategic/challenges.rs': [
        'fn materialize_narrative_combat(', 'let encounter = build_strategic_encounter(',
        'current_party_fatigue_percent(ctx, &living_party_member_ids(ctx, &party.id))'],
    'crates/strategic-web/src/templates/settlement/travel.rs': [
        'encounter.run_ineligibility.as_deref()', '"Cannot run: " (reason)'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
# Exact integer arithmetic over the explicitly stated hypothetical inputs. It is
# evidence bookkeeping, not a call to the product or a claim of reachability.
whole = 10_000
road = logistics = whole
speed = lambda fatigue, remaining: (83 * (whole - fatigue * 50) * remaining * logistics * road) // whole**4
if (speed(0, 8000), speed(15, 8000), speed(0, 6000)) != (66, 61, 49):
    raise ValueError('counterexample arithmetic changed')
if (300 * 100 != 15 * 2000 or 90 * 5 != 450 or 90 * 5 != 225 * 2):
    raise ValueError('counterexample input relationships changed')

historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies']['SEGOVB-L12-ENCOUNTER-UNCOMMITTED-ROW-READMISSION']
if (historical['checkpoint'] != '059' or old['source_original_index'] != historical['matching_encounters_role_index'] or
    old['source_sha256'] != historical['matching_encounters_sha256'] or old['accepted_unique_count_change'] != 0 or
    old['current_reconciled'] is not False or old['runtime_validation'] != 'Not run'):
    raise ValueError('historical record qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
identities = [(historical['matching_encounters_role_index'], historical['matching_encounters_sha256']),
              (historical['different_physical_p16_role_index'], historical['different_physical_p16_sha256'])]
identities += [(index, historical['matching_core_sha256']) for index in historical['matching_core_role_indexes']]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for index, sha in identities:
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
if (historical['matching_encounters_sha256'] != hashlib.sha256(sources['crates/adventuresim-stdb-module/src/strategic/encounters.rs'].encode()).hexdigest() or
    historical['matching_core_sha256'] != hashlib.sha256(sources['crates/adventuresim-core/src/encounter.rs'].encode()).hexdigest() or
    historical['different_physical_p16_sha256'] == historical['matching_encounters_sha256']):
    raise ValueError('historical current/different distinction changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
    candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified twelve Git-bound movement sources, source arithmetic and separately attributed older encounter record')
