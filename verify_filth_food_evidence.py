"""Verify preserved source and historical identities, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
filth = json.loads((root / 'ASSESSMENT-L1-TRAVEL-FILTH-REMAINDER.json').read_text())
food = json.loads((root / 'ASSESSMENT-L1-MICROBIAL-AGGREGATES.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-106-110.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, name = line.split('\t', 1)
    _, _, blob = header.split()
    if name in tree:
        raise ValueError('duplicate current path')
    tree[name] = blob
sources = {}
for doc in (filth, food):
    for item in doc['current_sources']:
        path = (root / item['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                blob != tree.get(item['path']) or item['revision'] != doc['current_revision']):
            raise ValueError('current source identity mismatch')
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid source range')
        sources[item['path']] = raw.decode()
core_filth = sources['crates/adventuresim-core/src/filth.rs']
module_filth = sources['crates/adventuresim-stdb-module/src/filth.rs']
calendar = sources['crates/adventuresim-world-schema/src/calendar.rs']
if not all(s in core_filth for s in ('TRAVEL_DIRT_PER_DAY: f32 = 8.0', 'numerator % minutes_per_day', 'saturating_mul(8)')):
    raise ValueError('filth accrual trace mismatch')
if not all(s in module_filth for s in ('pub remainder_numerator: u16', 'remainder_numerator: 0',
                                       '(MINUTES_PER_DAY - remainder).div_ceil(8).max(1)')):
    raise ValueError('filth persistence trace mismatch')
if 'MINUTES_PER_DAY: u64 = 24 * 60' not in calendar or not (24 * 60 < 1441 <= 65535):
    raise ValueError('conditional remainder arithmetic mismatch')
core_food = sources['crates/adventuresim-core/src/food.rs']
cooking = sources['crates/adventuresim-stdb-module/src/food/cooking.rs']
if not all(s in core_food for s in ('!total_load.is_finite()', 'final_mass_kg <= 0.0',
                                    'return 0.0;', 'return MAX_CONTAMINATION;',
                                    'total_mass_kg > 25.0', 'cooking_duration_minutes(method, safety_minutes, total_mass_kg)?')):
    raise ValueError('food policy trace mismatch')
if not all(s in cooking for s in ('v.is_finite() && v >= 0.0', 'food::microbial_concentration(loads.iter().sum(), mass)',
                                  'food::cooking_duration_minutes_for_check(method, &safety, mass, check)')):
    raise ValueError('cooking preflight trace mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in history['historical_source_roles']:
        raw = archive.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-106-110.json'):
            raise ValueError('historical role identity mismatch')
if [item['original_index'] for item in history['historical_source_roles']] != [106,107,109,110]:
    raise ValueError('historical population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
for identifier, record in [('REVIEW-TRAVEL-FILTH-REMAINDER-001','ASSESSMENT-L1-TRAVEL-FILTH-REMAINDER.json'),
                           ('REVIEW-MICROBIAL-INVALID-AGGREGATE-001','ASSESSMENT-L1-MICROBIAL-AGGREGATES.json')]:
    entry = next(item for item in baseline['repair_ledger']['entries'] if item['id'] == identifier)
    if entry['record'] != record or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
        raise ValueError('ledger linkage mismatch')
print('verified filth and food source identities; invalid-state reachability remains open')
