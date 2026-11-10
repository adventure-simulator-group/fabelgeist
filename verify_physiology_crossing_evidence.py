"""Verify source identities and a rational illustrative calculation only."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-PHYSIOLOGY-CROSSING.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-128.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t',1)
    _, _, blob = header.split()
    if path in tree:
        raise ValueError('duplicate source path')
    tree[path] = blob
sources = {}
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
    if any(not 1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']):
        raise ValueError('invalid source range')
    sources[item['path']] = raw.decode()
physiology = sources['crates/adventuresim-core/src/physiology.rs']
disease = sources['crates/adventuresim-core/src/disease.rs']
server = sources['crates/adventuresim-stdb-module/src/disease.rs']
if not all(s in physiology for s in ('.clamp(-1.0, 2.0)', 'if state_at(right).terminal().is_none()',
                                     'combined.add_bounded(administration.effect_at(now))',
                                     'self.get(*meter) >= meter.terminal_loss()')):
    raise ValueError('physiology trace mismatch')
if not all(s in disease for s in ('pub fn structural_minutes(', 'pub fn private_meter_state(', '.clamp(-1.0, 2.0)')):
    raise ValueError('disease source trace mismatch')
if not all(s in server for s in ('let mut structural = vec![from, to]',
                                 'physiology::first_terminal_crossing(&structural',
                                 'disease::private_meter_state(*episode, minute, immunity, &key.key)')):
    raise ValueError('server boundary trace mismatch')
def clamp(value):
    return max(Fraction(-1), min(Fraction(2), value))
def illustrative_value(minute):
    base = Fraction(-2,100)
    a = min(Fraction(2), Fraction(3,10)*minute)
    b = Fraction(1) - Fraction(1,10)*minute
    return clamp(clamp(clamp(base) + a) + b) - 1
if (illustrative_value(0) != Fraction(-1,50) or illustrative_value(7) != 1 or
        illustrative_value(10) != Fraction(49,50)):
    raise ValueError('illustrative rational arithmetic mismatch')
item = history['historical_source_roles'][0]
with tarfile.open(root / item['evidence_archive'],'r:gz') as archive:
    raw = archive.extractfile(item['sha256']).read()
role = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][128]
if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != 1169 or
        item['new_personal_read_ranges'] != [[1,1169]] or role['source_id'] != item['source_id'] or
        role['status'] != 'full-historical-body-trace-open' or
        role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-128.json'):
    raise ValueError('historical role attribution mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == 'REVIEW-PHYSIOLOGY-TERMINAL-CROSSING-001')
if entry['record'] != 'ASSESSMENT-L1-PHYSIOLOGY-CROSSING.json' or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
    raise ValueError('ledger linkage mismatch')
print('verified crossing source and illustrative rational calculation; authored-history reachability remains open')
