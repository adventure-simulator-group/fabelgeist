"""Verify movement-source identities and a conditional arithmetic observation.

This reads preserved evidence only. It does not run the product, its build,
validators, tests, reports or simulations. Verify the tree and roles first.
"""
import gzip
import hashlib
import json
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
doc = json.loads((root / 'ASSESSMENT-L1-MELEE-MOVEMENT.json').read_text())
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

movement = sources['crates/adventuresim-core/src/autoresolve/melee_movement.rs']
for needle in ('pub displacement_metres: f32', 'pub distance_after_metres: f32',
               'displacement_metres: accelerating_displacement + constant_displacement'):
    if needle not in movement:
        raise ValueError('expected movement producer source missing')
timeline = sources['crates/adventuresim-core/src/autoresolve/timeline.rs']
for needle in ('pub movement_displacement_metres: Option<f32>',
               'const STEPS_PER_SECOND: f32 = 64.0', 'Self::tick_at(time_seconds)'):
    if needle not in timeline:
        raise ValueError('expected timeline source missing')
joint = sources['crates/adventuresim-core/src/autoresolve/joint_melee.rs']
for needle in ('TACTICAL_SERVER_STEPS_PER_SECOND: f32 = 64.0', '.round()',
               'parameters.combat_round_seconds.seconds() / step_count as f32'):
    if needle not in joint:
        raise ValueError('expected strategic window calculation missing')
audit = sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/audit.rs']
if '1.0_f32 / 64.0 + 1.0e-6' not in audit:
    raise ValueError('expected movement report threshold missing')
# This checks an exact mathematical margin, not binary32 execution or reachability.
round_seconds = Fraction(2, 100)
scaled = round_seconds * 64
if not (Fraction(1, 2) < scaled < Fraction(3, 2)):
    raise ValueError('hypothetical window rounding interval mismatch')
if not round_seconds > Fraction(1, 64) + Fraction(1, 1_000_000):
    raise ValueError('hypothetical bound margin mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
for ident in (doc['contract_id'], 'NEW-ITERATION-MOVEMENT-WINDOW-001'):
    if sum(x['id'] == ident and x['record'] == 'ASSESSMENT-L1-MELEE-MOVEMENT.json'
           for x in baseline['repair_ledger']['entries']) != 1:
        raise ValueError('movement ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
if len(doc['historical_source_roles']) != 6:
    raise ValueError('historical role count mismatch')
print('verified movement source identities and mathematical margin; configuration and runtime remain open')
