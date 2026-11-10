"""Verify preserved source identities and bounded armor-evidence claims.

This reads evidence as data. It does not run the product, its validator or a
simulation. Run verify_lock_inventory.py and verify_role_worklist.py first.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-ARMOR-EVIDENCE.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, _, blob = header.split()
    if path in tree:
        raise ValueError('duplicate tree path')
    tree[path] = blob

sources = {}
for source in doc['current_sources']:
    path = (root / source['preserved_bytes']).resolve()
    if not path.is_relative_to(root):
        raise ValueError('source escapes checkpoint')
    raw = gzip.decompress(path.read_bytes())
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (source['revision'] != doc['current_revision'] or
            hashlib.sha256(raw).hexdigest() != source['sha256'] or
            len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines'] or
            blob != source['git_blob'] or blob != tree.get(source['path'])):
        raise ValueError('source identity mismatch: ' + source['path'])
    if any(not 1 <= lo <= hi <= source['lines'] for lo, hi in source['ranges']):
        raise ValueError('invalid source range')
    sources[source['path']] = raw.decode()

catalog = json.loads(sources['content/items/catalog.yaml'])
items = {item['id']: item for item in catalog['items']}
placements = items['vambrace']['equipment']['placements']
if [x['id'] for x in placements] != ['left', 'right']:
    raise ValueError('current vambrace placement order differs')
if [[surface['regions'] for surface in x['surface']] for x in placements] != [[['left_forearm']], [['right_forearm']]]:
    raise ValueError('current vambrace regions differ')
if [x['id'] for x in items['brigandine']['equipment']['placements']] != ['worn']:
    raise ValueError('current brigandine placement differs')
armor = sources['crates/adventuresim-core/src/autoresolve/melee_iteration/evidence/armor.rs']
for needle in ('(BodySide::Left, BodyPart::LeftArm, 0_usize',
               '(BodySide::Right, BodyPart::RightArm, 1_usize',
               '&equipment.placements[placement_index]'):
    if needle not in armor:
        raise ValueError('expected positional fixture source missing')
coverage = sources['crates/adventuresim-core/src/combat/coverage.rs']
for needle in ('if region_part != part', 'Region::LeftForearm => (BodyPart::LeftArm',
               'Region::RightForearm => (BodyPart::RightArm'):
    if needle not in coverage:
        raise ValueError('expected region projection source missing')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == doc['contract_id'] for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('armor ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
if len(doc['historical_source_roles']) != 5:
    raise ValueError('historical role count mismatch')
print('verified pinned armor source and current authored order; hypothetical admission remains open')
