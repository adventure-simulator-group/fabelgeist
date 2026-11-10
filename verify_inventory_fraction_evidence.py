"""Verify historical and current source identities, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-INVENTORY-FRACTION.json').read_text())
history = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-112-116.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t',1)
    _, _, blob = header.split()
    if path in tree:
        raise ValueError('duplicate current path')
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
core = sources['crates/adventuresim-core/src/inventory_measurement.rs']
store = sources['crates/adventuresim-stdb-module/src/inventory_amount.rs']
rest = sources['crates/strategic-web/src/routes/settlements/rest_preview.rs']
merchant = sources['crates/strategic-web/src/templates/settlement/trade/merchant.rs']
if not all(s in core for s in ('pub struct ConsumableFractionMicros(u32)', 'MICROS_PER_WHOLE: u32 = 1_000_000',
                               'fraction_micros > Self::MICROS_PER_WHOLE', 'assert!(parts > 0')):
    raise ValueError('fraction owner trace mismatch')
if not all(s in store for s in ('pub remaining_fraction_micros: u32',
                                'remaining_fraction_micros: ConsumableFractionMicros::WHOLE.get()',
                                'persisted consumable fraction must not exceed one whole',
                                'remaining_fraction_micros: state.remaining_fraction_micros')):
    raise ValueError('fraction storage trace mismatch')
if 'public consumable fraction must not exceed one whole' not in rest or 'public consumable fraction must not exceed one whole' not in merchant:
    raise ValueError('fraction presentation trace mismatch')
if not (1_000_001 > 1_000_000 and 1_000_001 < 2**32 and 1_000_000 // 25 > 0):
    raise ValueError('fraction arithmetic mismatch')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in history['historical_source_roles']:
        raw = archive.extractfile(item['sha256']).read()
        role = roles[item['original_index']]
        if (hashlib.sha256(raw).hexdigest() != item['sha256'] or len(raw.splitlines()) != item['lines'] or
                item['new_personal_read_ranges'] != [[1,item['lines']]] or role['source_id'] != item['source_id'] or
                role['status'] != 'full-historical-body-trace-open' or
                role['historical_assessment'] != 'ASSESSMENT-L01-HISTORICAL-112-116.json'):
            raise ValueError('historical role attribution mismatch')
if [item['original_index'] for item in history['historical_source_roles']] != [112,113,116]:
    raise ValueError('historical role population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
entry = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == 'REVIEW-CONSUMABLE-FRACTION-STATE-001')
if entry['record'] != 'ASSESSMENT-L1-INVENTORY-FRACTION.json' or entry['repair'] != 'not started' or entry['validation'] != 'not performed':
    raise ValueError('ledger linkage mismatch')
print('verified fraction and historical source identities; invalid-state reachability remains open')
