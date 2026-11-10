"""Verify wheel and historical source evidence, not product behavior."""
import gzip
import hashlib
import json
import math
import struct
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
wheel = json.loads((root / 'ASSESSMENT-L1-WHEEL-CUTOFF.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-90-99.json').read_text())

def tree_index(path):
    index = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in index:
            raise ValueError('duplicate tree path')
        index[name] = blob
    return index

trees = {wheel['current_revision']: tree_index('CURRENT-TREE.tsv'),
         wheel['reported_selected_revision']: tree_index('lock-evidence/selected-tree.tsv')}
sources = {}
for key in ('current_sources', 'reported_selected_sources'):
    for item in wheel[key]:
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

current = wheel['current_revision']
old = wheel['reported_selected_revision']
body = sources[(current, 'crates/adventuresim-tactical-client/src/ui/wheel.rs')]
page = sources[(current, 'crates/adventuresim-stdb-module/static/tactical-hud.js')]
old_ui = sources[(old, 'crates/adventuresim-tactical-client/src/ui.rs')]
for text, needles in [(body, ('remaining -= amount;', '(amount * ARC_QUANTUM).round() / ARC_QUANTUM',
                              'const ARC_QUANTUM: f32 = 1000.0;',
                              'const MIN_VISIBLE_INCAPACITATION_SEGMENT: f32 = 0.005;')),
                      (page, ('const end = angle + arc.share * Math.PI * 2;', 'angle = end;')),
                      (old_ui, ('let end = cursor + amount * std::f32::consts::TAU;',
                                'remaining -= amount;'))]:
    if any(needle not in text for needle in needles):
        raise ValueError('expected source trace missing')

manifest = json.loads((root / wheel['native_archive']).read_text())
archive = (root / wheel['native_archive']).parent / manifest['archive']
raw = archive.read_bytes()
if hashlib.sha256(raw).hexdigest() != manifest['sha256'] or len(raw) != manifest['bytes']:
    raise ValueError('native archive identity mismatch')
lock = sources[(current, 'Cargo.lock')]
if ('name = "bevy_color"\nversion = "0.19.1"' not in lock or
        'checksum = "' + manifest['sha256'] + '"' not in lock):
    raise ValueError('native archive is not pinned by source lockfile')
with tarfile.open(archive, 'r:gz') as tar:
    for member in manifest['inspected_members']:
        data = tar.extractfile(member['path']).read()
        if len(data) != member['bytes'] or hashlib.sha256(data).hexdigest() != member['sha256']:
            raise ValueError('native archive member mismatch')
    srgba = tar.extractfile('bevy_color-0.19.1/src/srgba.rs').read().decode()
    if 'format!("#{r:02X}{g:02X}{b:02X}{a:02X}")' not in srgba:
        raise ValueError('color alpha formatter source missing')

# Evidence-only arithmetic with IEEE binary32 operations. Values are away from
# rounding ties; this does not call the Rust function, serializer, or browser.
def f32(value):
    return struct.unpack('!f', struct.pack('!f', value))[0]
remaining = f32(1.0)
shares = []
for value in (f32(0.3336), f32(0.3336), f32(0.3328)):
    amount = min(value, remaining)
    remaining = f32(remaining - amount)
    scaled = f32(amount * f32(1000.0))
    shares.append(f32(f32(math.floor(scaled + 0.5)) / f32(1000.0)))
if not (all(value >= 0.005 for value in shares) and sum(shares) > 1.0009):
    raise ValueError('arithmetic counterexample no longer reproduced')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
if [x['original_index'] for x in history['historical_source_roles']] != [90, 92, 94, 98, 99]:
    raise ValueError('unexpected historical roles')
for item in history['historical_source_roles']:
    row = roles[item['original_index']]
    if (row['source_id'] != item['source_id'] or row['historical_sha256'] != item['sha256'] or
            row['status'] != 'full-historical-body-trace-open' or
            row.get('historical_assessment') != 'ASSESSMENT-L01-HISTORICAL-90-99.json'):
        raise ValueError('historical source linkage mismatch')
if sum(x['id'] == wheel['contract_id'] and x['record'] == 'ASSESSMENT-L1-WHEEL-CUTOFF.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('wheel ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
for path in ('crates/adventuresim-tactical-client/src/ui.rs',
             'crates/adventuresim-tactical-client/src/ui/wheel.rs',
             'crates/adventuresim-tactical-client/src/ui/combat_state.rs'):
    row = next(x for x in baseline['source_delta'] if x['path'] == path)
    if not row['lens_status'][0].startswith('partial'):
        raise ValueError('UI delta disposition mismatch')
print('verified wheel sources, pinned color archive and conditional arithmetic; product behavior remains unverified')
