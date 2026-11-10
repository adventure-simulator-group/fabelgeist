"""Verify combat source evidence and numeric bounds, not a product build.

Run verify_lock_inventory.py and verify_role_worklist.py first; they reconstruct
the source tree and validate historical identities and body-read ranges.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-COMBAT-ADMISSION.json').read_text())
tree = {}
for line in (root / 'CURRENT-TREE.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, _, oid = header.split()
    if path in tree:
        raise ValueError('duplicate tree path')
    tree[path] = oid
for source in doc['current_sources']:
    path = (root / source['preserved_bytes']).resolve()
    if not path.is_relative_to(root):
        raise ValueError('source escapes checkpoint')
    raw = gzip.decompress(path.read_bytes())
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (source['revision'] != doc['current_revision']
            or hashlib.sha256(raw).hexdigest() != source['sha256']
            or len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines']
            or blob != source['git_blob'] or blob != tree.get(source['path'])):
        raise ValueError('combat source mismatch: ' + source['path'])
    if any(not 1 <= lo <= hi <= source['lines'] for lo, hi in source['ranges']):
        raise ValueError('combat source range mismatch')

# Exact integer comparisons: 10^-100 is between binary64's minimum normal
# and half binary32's minimum subnormal. This does not execute a float parser.
if not (2**150 < 10**100 < 2**1022):
    raise ValueError('numeric bound mismatch')
if len(doc['historical_source_roles']) != 6:
    raise ValueError('historical read population mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == doc['contract_id'] for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('combat ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
print('verified combat sources and exact numeric bounds; compiler and reachability remain open')
