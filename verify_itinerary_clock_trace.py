"""Verify preserved itinerary evidence and one integer fixed-point derivation.

This does not call the forecast, build product code, or run a product test.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-ITINERARY-CLOCK-95E.json').read_text())
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
if len(sources) != 4:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-core/src/strategic_time.rs': ['let start = 12 * 60 - walking_minutes / 2;',
        'Some((start.max(absolute_minute), end))', 'if segments.len() >= MAX_ITINERARY_SEGMENTS',
        'let available = walk_end.elapsed_since(absolute);', 'if duration == 0 {',
        'absolute = absolute.saturating_add_minutes(1);'],
    'crates/adventuresim-world-schema/src/calendar.rs': ['pub const MAX: Self = Self::new(u64::MAX);',
        'Self::new(self.minutes.saturating_add(elapsed_minutes))', 'Self::new(self.day_index() * MINUTES_PER_DAY)'],
    'crates/strategic-web/src/routes/travel.rs': ['StrategicMinute::new(row.minutes.minutes)',
        'destination.forecast_minutes()', 'forecast_itinerary('],
    'crates/adventuresim-stdb-module/src/strategic/journey_camp.rs': ['journey.departure_minute.wrapping_day_offset(elapsed)',
        'if itinerary.truncated {', 'forecast_itinerary('],
}
for path, needles in markers.items():
    if any(needle not in sources[path] for needle in needles):
        raise ValueError('missing source marker: ' + path)
# Symbolically check one selected day's integer saturation, without running Rust.
maximum = 2**64 - 1
day_start = (maximum // 1440) * 1440
walking_start = min(maximum, day_start + 480)
walking_end = min(maximum, day_start + 960)
next_start = min(maximum, walking_start + 1440)
next_end = min(maximum, walking_end + 1440)
selected_start = max(next_start, maximum)
available = max(0, next_end - maximum)
duration = min(available, 1)
next_absolute = min(maximum, maximum + 1)
if (selected_start, next_end, duration, next_absolute) != (maximum, maximum, 0, maximum):
    raise ValueError('fixed-point arithmetic mismatch')
candidate = doc['candidate']
related = candidate['historical_related_candidate']
history = json.loads((root / related['ledger']).read_text())['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][306]['contract']
if history['proposed_id'] != related['id'] or related['checkpoint'] != '059':
    raise ValueError('historical relation mismatch')
entries = baseline['newly_assessed_ledger']['entries']
matches = [entry for entry in entries if entry['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-ITINERARY-CLOCK-95E.json' or
        progress['new_current_ledger_candidates'] != len(entries) or
        candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
        not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified four source identities, a symbolic clock fixed point and historical relation')
