"""Verify preserved source identities and bounded arithmetic, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
weights = json.loads((root / 'ASSESSMENT-L1-WEAPON-SKILL-WEIGHTS.json').read_text())
sigils = json.loads((root / 'ASSESSMENT-L1-SIGIL-POSITIONS.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-100-104.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, _, blob = header.split()
    if path in tree:
        raise ValueError('duplicate current path')
    tree[path] = blob
sources = {}
for doc in (weights, sigils):
    for item in doc['current_sources']:
        path = (root / item['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw) != item['bytes'] or
                len(raw.splitlines()) != item['lines'] or blob != item['git_blob'] or
                blob != tree.get(item['path']) or item['revision'] != doc['current_revision']):
            raise ValueError('source identity mismatch')
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid source read range')
        sources[item['path']] = raw.decode()
equipment = sources['crates/adventuresim-core/src/equipment.rs']
validation = sources['crates/adventuresim-core/src/item_catalog_validation.rs']
if not all(s in equipment for s in ('pub sword: f32', 'v.is_finite() && v >= 0.0',
                                    '.map(|(skill, weight)| check(skill) * weight)', '/ total')):
    raise ValueError('weight trace mismatch')
if not all(s in validation for s in ('(total - 1.0).abs() > 0.000_1',
                                     'weights must be finite, non-negative, and sum to 1')):
    raise ValueError('catalog trace mismatch')
# Exact integer f32 maximum; doubling it exceeds the finite type range.
f32_max = (2**24 - 1) * 2**104
if not (f32_max > 0 and 2 * f32_max > f32_max):
    raise ValueError('counterexample arithmetic mismatch')
clue = sources['crates/adventuresim-puzzles/src/ordered_sigils.rs']
fey = sources['crates/adventuresim-core/src/errantry.rs']
backend = sources['crates/adventuresim-stdb-module/src/strategic/challenges.rs']
route = sources['crates/strategic-web/src/routes/challenges.rs']
if not all(s in clue for s in ('Exact { sigil: Sigil, position: u8 }',
                               'usize::from(*position) >= ORDERED_SIGIL_COUNT', 'puzzle.validate()?')):
    raise ValueError('clue admission trace mismatch')
if 'FEY_EXACT[sigil(value)][usize::from(position)]' not in fey or '[[&str; 5]; 5]' not in fey:
    raise ValueError('fey lookup trace mismatch')
if 'puzzle.validate().ok()?;' not in backend or 'serde_json::from_str(&challenge.puzzle_projection_json)' not in route:
    raise ValueError('projection trace mismatch')
if not (0 <= 4 < 5 <= 255 and 255 + 1 > 255):
    raise ValueError('position arithmetic mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in history['historical_source_roles']:
        raw = archive.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-100-104.json'):
            raise ValueError('historical attribution mismatch')
if [item['original_index'] for item in history['historical_source_roles']] != [100,104]:
    raise ValueError('historical population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
for identifier, filename in [('REVIEW-WEAPON-SKILL-WEIGHT-AGGREGATE-001','ASSESSMENT-L1-WEAPON-SKILL-WEIGHTS.json'),
                             ('REVIEW-ORDERED-SIGIL-POSITIONS-001','ASSESSMENT-L1-SIGIL-POSITIONS.json')]:
    entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == identifier)
    if entry['record'] != filename or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
        raise ValueError('ledger linkage mismatch')
print('verified weight and sigil source identities; reachability and product behavior remain open')
