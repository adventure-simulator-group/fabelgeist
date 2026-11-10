"""Verify rate-source identities and constants, not simulation behavior.

Run verify_lock_inventory.py and verify_role_worklist.py first to reconstruct the
two trees and validate the historical source identities and read ranges.
"""
import gzip
import hashlib
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-COMBAT-STEP-RATES.json').read_text())


def tree(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in result:
            raise ValueError('duplicate tree path')
        result[path] = oid
    return result


trees = {
    doc['current_revision']: tree('CURRENT-TREE.tsv'),
    doc['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv'),
}
sources = {}
for key in ('current_sources', 'reported_selected_sources'):
    for source in doc[key]:
        path = (root / source['preserved_bytes']).resolve()
        if not path.is_relative_to(root):
            raise ValueError('source escapes checkpoint')
        raw = gzip.decompress(path.read_bytes())
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        if (hashlib.sha256(raw).hexdigest() != source['sha256']
                or len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines']
                or blob != source['git_blob'] or blob != trees[source['revision']].get(source['path'])):
            raise ValueError('rate source mismatch: ' + source['path'])
        if any(not 1 <= lo <= hi <= source['lines'] for lo, hi in source['ranges']):
            raise ValueError('invalid rate source range')
        sources[(key, source['path'])] = raw.decode()

joint = 'crates/adventuresim-core/src/autoresolve/joint_melee.rs'
net = 'crates/adventuresim-tactical-netcode/src/lib.rs'
yaml = 'content/tactical/combat.yaml'
if sources[('current_sources', joint)] != sources[('reported_selected_sources', joint)]:
    raise ValueError('strategic source differs between cited revisions')
if not re.search(r'TACTICAL_SERVER_STEPS_PER_SECOND:\s*f32\s*=\s*64\.0', sources[('current_sources', joint)]):
    raise ValueError('strategic rate not found')
if not re.search(r'TACTICAL_TICK_HZ:\s*u32\s*=\s*30', sources[('current_sources', net)]):
    raise ValueError('current tactical rate not found')
if not re.search(r'FIXED_TIMESTEP_HZ:\s*f64\s*=\s*64\.0', sources[('reported_selected_sources', net)]):
    raise ValueError('selected tactical rate not found')
if not re.search(r'autoresolve:\s*\n\s*combat_round_seconds:\s*1\.0', sources[('current_sources', yaml)]):
    raise ValueError('authored round value not found')
audit = sources[('current_sources', 'crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/audit.rs')]
causal = sources[('current_sources', 'crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/causal/autoresolve.rs')]
if '1.0_f32 / 64.0 + 1.0e-6' not in audit or 'maximum_movement_segment_seconds' not in audit:
    raise ValueError('movement acceptance threshold not found')
if 'maximum_movement_segment_seconds' not in causal or '.max(elapsed_measurement)' not in causal:
    raise ValueError('movement segment report source not found')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == doc['contract_id'] for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('rate ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
if len(doc['historical_source_roles']) != 6:
    raise ValueError('historical role count mismatch')
print('verified pinned rate-source constants and identities; rate policy remains open')
