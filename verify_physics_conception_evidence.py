"""Verify recorded source identities and arithmetic without running the product."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
con = json.loads((root / 'ASSESSMENT-L1-CONCEPTION-STATE.json').read_text())
phys = json.loads((root / 'ASSESSMENT-L01-PHYSICS-CONFIG.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate source tree path')
        result[name] = blob
    return result

trees = {con['current_revision']: tree('CURRENT-TREE.tsv'),
         phys['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (con, phys):
    for key in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(key, []):
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

current = con['current_revision']
courtship = sources[(current, 'crates/adventuresim-core/src/courtship.rs')]
reproduction = sources[(current, 'crates/adventuresim-stdb-module/src/relationship/reproduction.rs')]
plugin = sources[(current, 'crates/adventuresim-tactical-core/src/physics/plugin.rs')]
for body, needles in [(courtship, ('pub conserved_joint_minutes: u8',
                                  'u64::from(state.conserved_joint_minutes) % CONCEPTION_QUANTUM_MINUTES',
                                  'ordinal: state.next_trial_ordinal.saturating_add(index)')),
                      (reproduction, ('conserved_joint_minutes: accrual.conserved_joint_minutes',
                                      'format!("conception-trial:{pair_id}:{}", trial.ordinal)')),
                      (plugin, ('Duration::from_secs_f64(1.0 / 30.0)',
                                '.run_if(any_presentation_body_simulates)',
                                'Collider::sphere(0.25)'))]:
    if any(needle not in body for needle in needles):
        raise ValueError('expected source trace missing')

manifest = json.loads((root / phys['native_archive']).read_text())
archive = (root / phys['native_archive']).parent / manifest['archive']
data = archive.read_bytes()
if hashlib.sha256(data).hexdigest() != manifest['sha256'] or len(data) != manifest['bytes']:
    raise ValueError('native archive mismatch')
lock = sources[(current, 'Cargo.lock')]
if ('name = "avian3d"\nversion = "0.7.0"' not in lock or
        'checksum = "' + manifest['sha256'] + '"' not in lock):
    raise ValueError('native archive not pinned by current lockfile')
with tarfile.open(archive, 'r:gz') as tar:
    for member in manifest['inspected_members']:
        body = tar.extractfile(member['path']).read()
        if len(body) != member['bytes'] or hashlib.sha256(body).hexdigest() != member['sha256']:
            raise ValueError('native archive member mismatch')
    collider = tar.extractfile('avian3d-0.7.0/src/collision/collider/parry/mod.rs').read().decode()
    position = tar.extractfile('avian3d-0.7.0/src/physics_transform/transform.rs').read().decode()
    if 'pub fn sphere(radius: Scalar)' not in collider or 'pub fn from_xyz(x: Scalar, y: Scalar, z: Scalar)' not in position:
        raise ValueError('native API source missing')

# Arithmetic inspection only: no planner, database, simulation or game call.
quantum = 60
carried = 60 % quantum
crossings = 0 if 0 < quantum - carried else 1
if carried != 0 or crossings != 0:
    raise ValueError('remainder counterexample mismatch')
maximum = (1 << 64) - 1
ordinals = [min(maximum, maximum + index) for index in range(2)]
if ordinals != [maximum, maximum]:
    raise ValueError('saturated ordinal counterexample mismatch')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
if [x['original_index'] for x in con['historical_source_roles']] != [91, 95]:
    raise ValueError('unexpected historical roles')
for item in con['historical_source_roles']:
    row = roles[item['original_index']]
    if (row['source_id'] != item['source_id'] or row['historical_sha256'] != item['sha256'] or
            row['status'] != 'full-historical-body-trace-open' or
            row.get('historical_assessment') != 'ASSESSMENT-L1-CONCEPTION-STATE.json'):
        raise ValueError('historical role linkage mismatch')
if sum(x['id'] == con['contract_id'] and x['record'] == 'ASSESSMENT-L1-CONCEPTION-STATE.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('conception ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
row = next(x for x in baseline['source_delta'] if x['path'] == phys['path'])
if not row['lens_status'][0].startswith('partial'):
    raise ValueError('physics delta disposition mismatch')
print('verified conception and physics source evidence; input reachability and product behavior remain open')
