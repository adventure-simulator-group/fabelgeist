"""Verify precision and mesh-count evidence only; do not run the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
precision = json.loads((root / 'ASSESSMENT-L1-CONTACT-PRECISION.json').read_text())
counts = json.loads((root / 'ASSESSMENT-L1-TRIANGLE-COUNTS.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {precision['current_revision']: tree('CURRENT-TREE.tsv'),
         counts['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (precision, counts):
    for key in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(key, []):
            path = (root / item['preserved_bytes']).resolve()
            if not path.is_relative_to(root):
                raise ValueError('source escapes checkpoint')
            raw = gzip.decompress(path.read_bytes())
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            if (hashlib.sha256(raw).hexdigest() != item['sha256'] or
                    len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
                    blob != item['git_blob'] or blob != trees[item['revision']].get(item['path'])):
                raise ValueError('source identity mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()

current = precision['current_revision']
owner = sources[(current, 'crates/adventuresim-core/src/combat/weapon_contact.rs')]
catalog = sources[(current, 'crates/adventuresim-core/src/item_catalog.rs')]
combat = sources[(current, 'crates/adventuresim-core/src/combat.rs')]
geometry = sources[(current, 'crates/adventuresim-core/src/equipment/parametric_weapon.rs')]
for body, needles in [(owner, ('-> ContactPrecision', 'value.is_finite() && value >= 0.0')),
                      (catalog, ('.precision_for_design(&design)', '.value();')),
                      (combat, ('ContactPrecision::new(if shortened_contact || unarmed',
                                'attacker_equip.weapon_precision()')),
                      (geometry, ('pub precision: f32', 'self.precision * effective_catalog_precision / base'))]:
    if any(needle not in body for needle in needles):
        raise ValueError('expected precision trace source missing')
facade = sources[(current, 'crates/adventuresim-tactical-client/src/presentation/mod.rs')]
census = sources[(current, 'crates/adventuresim-tactical-client/src/tactical_scene_viewer/triangle_census.rs')]
if ('fn mesh_triangle_count(mesh: &Mesh) -> usize' not in facade or
        'let triangles_per_tuft = triangle_count.0;' not in census or
        'resident_tufts * triangles_per_tuft' not in census):
    raise ValueError('expected count-source distinction missing')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
for doc, name in ((precision, 'ASSESSMENT-L1-CONTACT-PRECISION.json'),
                  (counts, 'ASSESSMENT-L1-TRIANGLE-COUNTS.json')):
    if sum(x['id'] == doc['contract_id'] and x['record'] == name
           for x in baseline['repair_ledger']['entries']) != 1:
        raise ValueError('ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
entry = next(x for x in baseline['source_delta']
             if x['path'] == 'crates/adventuresim-tactical-client/src/presentation/mod.rs')
if not entry['lens_status'][0].startswith('partial'):
    raise ValueError('presentation delta disposition mismatch')
if len(precision['historical_source_roles']) != 1:
    raise ValueError('historical role count mismatch')
print('verified contact precision and triangle count source identities; owner policy and runtime remain open')
