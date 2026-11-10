"""Verify preserved iteration evidence, not reachability or product behavior.

Run verify_lock_inventory.py first to reconstruct the two cited Git trees.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-ITERATION-TIMEOUT.json').read_text())
queue = json.loads((root / 'baseline-059/queue.json').read_text())
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())


def tree(name):
    entries = {}
    for line in (root / name).read_text().splitlines():
        head, path = line.split('\t', 1)
        _, _, oid = head.split()
        if path in entries:
            raise ValueError('duplicate path')
        entries[path] = oid
    return entries


trees = {
    doc['current_revision']: tree('CURRENT-TREE.tsv'),
    'ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04': tree('lock-evidence/selected-tree.tsv'),
}


def evidence_path(name):
    target = (root / name).resolve()
    if not target.is_relative_to(root):
        raise ValueError('evidence path escapes checkpoint')
    return target


for key in ('current_sources', 'reported_selected_sources'):
    for item in doc[key]:
        raw = gzip.decompress(evidence_path(item['preserved_bytes']).read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != item['sha256']
                or len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines']
                or blob != item['git_blob'] or blob != trees[item['revision']].get(item['path'])):
            raise ValueError('source byte or tree mismatch: ' + item['path'])
        if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
            raise ValueError('invalid source range: ' + item['path'])

bundle = doc['historical']['receipt_bundle']
if len(bundle) != 9:
    raise ValueError('historical receipt population mismatch')
for item in bundle:
    raw = gzip.decompress(evidence_path(item['preserved_bytes']).read_bytes())
    if hashlib.sha256(raw).hexdigest() != item['sha256']:
        raise ValueError('historical receipt hash mismatch')
receipt = json.loads(gzip.decompress(evidence_path(bundle[0]['preserved_bytes']).read_bytes()))
registrations = {item['original_index']: item for item in bundle[1:]}
for item in receipt['sources']:
    saved = registrations[item['original_index']]
    if saved['sha256'] != item['registration_sha256']:
        raise ValueError('historical registration link mismatch')
if (queue['entries'][118]['contract_id'] != doc['contract_id']
        or queue['entries'][118]['exact_source_identities_and_assessed_ranges']
        != doc['historical']['source_identities']):
    raise ValueError('historical queue linkage mismatch')
if sum(item['id'] == doc['contract_id'] for item in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('connected ledger link mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('connected ledger count mismatch')
print('verified iteration sources, historical receipt and eight registrations; scope remains qualified')
