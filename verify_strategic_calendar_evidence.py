"""Verify source identities and a numeric boundary without running the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-STRATEGIC-CALENDAR.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {doc['current_revision']: tree('CURRENT-TREE.tsv'),
         doc['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for key in ('current_sources', 'reported_selected_sources'):
    for item in doc[key]:
        path = (root / item['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                blob != trees[item['revision']].get(item['path'])):
            raise ValueError('source identity mismatch: ' + item['path'])
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid source range')
        sources[(item['revision'], item['path'])] = raw.decode()

cur = doc['current_revision']
calendar = sources[(cur, 'crates/adventuresim-world-schema/src/calendar.rs')]
js = sources[(cur, 'crates/strategic-web/static/strategic-calendar.js')]
travel = sources[(cur, 'crates/strategic-web/static/travel-planner.js')]
template = sources[(cur, 'crates/strategic-web/src/templates/settlement/travel.rs')]
clock = sources[(cur, 'crates/strategic-web/src/routes/clock.rs')]
for body, needles in [(calendar, ('pub struct StrategicMinute', 'minutes: u64',
                                 'pub const fn new(minutes: u64)', 'self.minutes % MINUTES_PER_DAY')),
                      (js, ('Math.floor(Number(minute))', 'modulo(')),
                      (travel, ('departure: Number(planner.dataset.departureMinute) || 0',
                                'renderTimeRail(planner, departure, elapsedTotal)')),
                      (template, ('data-departure-minute=', 'preview_departure_minute.get()')),
                      (clock, ('character_minutes: u64', 'official_minutes: u64', 'Json(CurrentTime'))]:
    if any(needle not in body for needle in needles):
        raise ValueError('expected source trace missing')

# Binary64 integer conversion evidence only; no JavaScript or product execution.
minute = (1 << 53) + 1
rounded = int(float(minute))
if rounded != minute - 1 or minute % 1440 != 33 or rounded % 1440 != 32:
    raise ValueError('numeric counterexample mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == doc['contract_id'] and x['record'] == 'ASSESSMENT-L1-STRATEGIC-CALENDAR.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('calendar ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
for path in ('crates/strategic-web/UNIFIED_RENDERER.md',
             'crates/strategic-web/src/templates/layout/shell.rs'):
    row = next(x for x in baseline['source_delta'] if x['path'] == path)
    if not row['lens_status'][0].startswith('partial'):
        raise ValueError('delta disposition mismatch')
print('verified calendar sources and conditional binary64 arithmetic; supported reachability remains open')
