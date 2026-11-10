"""Check preserved training source and a symbolic magnitude comparison only."""
import gzip
import hashlib
import json
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-TRAINING-FINITE-95E.json').read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert doc['source_revision'] == revision

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
    if not saved.is_relative_to(root) or item['revision'] != revision or item['content_encoding'] != 'gzip':
        raise ValueError('invalid source reference')
    raw = gzip.decompress(saved.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
            hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
            current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('source mismatch: ' + item['path'])
    if any(not 1 <= first <= last <= item['lines'] for first, last in item['ranges']):
        raise ValueError('invalid source range')
    if item['path'] in sources:
        raise ValueError('duplicate source')
    sources[item['path']] = raw.decode()
if len(sources) != 8:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-core/src/skill.rs': ['pub fn hours_for_rank(self, rank: f32)',
        'f32::INFINITY', 'let effective_gain = real_hours * aptitude_training_multiplier(aptitude);',
        '*stored_effective_hours = current + accepted;', 'excess_effective_hours: effective_gain - accepted',
        'direct + transferred', 'let hours = if hours.is_finite()'],
    'crates/adventuresim-core/src/strategic_schedule.rs': ['pub fn apply_schedule_training(',
        'training_hours_increment(elapsed_minutes, minutes)', 'pub fn apply_curriculum_training('],
    'crates/adventuresim-core/src/strategic_time.rs': ['pub fn training_hours_increment(elapsed_minutes: u64, daily_minutes: u16)'],
    'crates/adventuresim-core/src/activity.rs': ['pub const ACTIVITY_TRAINING_RATE: f32 = 0.25;'],
    'crates/adventuresim-stdb-module/src/time/schedule.rs': ['fn apply_training(', 'skills.knife_hours = hours.knife;'],
    'crates/adventuresim-stdb-module/src/continuity.rs': ['curriculum_real_hours(', 'apply_direct_training(skill, stored, real_hours, &attributes)'],
    'crates/adventuresim-stdb-module/src/character.rs': ['pub struct CharacterSkills', 'pub knife_hours: f32'],
    'crates/adventuresim-stdb-module/src/capability.rs': ['impl PlayerSkills for CharacterSkills', 'Skill::Knife => self.knife_hours'],
}
for path, needles in markers.items():
    if any(needle not in sources[path] for needle in needles):
        raise ValueError('missing source marker: ' + path)
# Exact magnitude bounds, not an execution of any product operation.
binary32_max = (Fraction(2) - Fraction(1, 2**23)) * 2**127
if binary32_max * Fraction(9, 4) <= binary32_max:
    raise ValueError('binary32 magnitude comparison failed')
maximum_basic_increment = Fraction(2**64 - 1, 1440) * Fraction(2**16 - 1, 60)
if maximum_basic_increment * Fraction(9, 4) >= binary32_max:
    raise ValueError('basic schedule bound comparison failed')
candidate = doc['candidate']
related = candidate['historical_related_candidate']
historical = json.loads((root / related['ledger']).read_text())['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][304]['contract']
if historical['proposed_id'] != related['id'] or related['checkpoint'] != '059':
    raise ValueError('historical candidate mismatch')
entries = baseline['newly_assessed_ledger']['entries']
matches = [entry for entry in entries if entry['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-TRAINING-FINITE-95E.json' or
        progress['new_current_ledger_candidates'] != len(entries) or candidate['repair'] != 'not started' or
        candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified current candidate')
print('verified eight current source snapshots, symbolic magnitudes and historical relation')
