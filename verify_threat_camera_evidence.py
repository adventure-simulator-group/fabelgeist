"""Verify pinned threat and camera source evidence without running the product.

Run verify_lock_inventory.py and verify_role_worklist.py first. Catalog files
are decoded only as static JSON source data, not through product parsing.
"""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
threat = json.loads((root / 'ASSESSMENT-L1-THREAT-IDENTITY.json').read_text())
camera = json.loads((root / 'ASSESSMENT-L01-CAMERA-ROOT.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, blob = header.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {threat['current_revision']: tree('CURRENT-TREE.tsv'),
         camera['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (threat, camera):
    for key in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(key, []):
            file = (root / item['preserved_bytes']).resolve()
            if not file.is_relative_to(root):
                raise ValueError('source escapes checkpoint')
            raw = gzip.decompress(file.read_bytes())
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            if (hashlib.sha256(raw).hexdigest() != item['sha256'] or
                    len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
                    blob != item['git_blob'] or blob != trees[item['revision']].get(item['path'])):
                raise ValueError('source identity mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()

current = threat['current_revision']
input_paths = threat['static_catalog_inventory']['inputs']
tree_inputs = sorted(name for name in trees[current] if name.startswith('content/quests/')
                     and name.count('/') == 2 and name.endswith('.yaml'))
if input_paths != tree_inputs:
    raise ValueError('quest source document inventory mismatch')
ids = [monster['id'] for path in input_paths
       for monster in json.loads(sources[(current, path)]).get('monsters', [])]
example = threat['static_catalog_inventory']['example']
if (len(ids) != threat['static_catalog_inventory']['monster_entries'] or example in ids
        or threat['static_catalog_inventory']['example_present']
        or not 1 <= len(example) <= 63
        or any(not (c.isascii() and (c.islower() or c.isdigit() or c in '_-.:' )) for c in example)):
    raise ValueError('static threat example mismatch')
body = sources[(current, 'crates/adventuresim-core/src/bestiary.rs')]
for needle in ('pub fn try_new(value: &str) -> Result<Self, UnknownThreatId>',
               '.expect("threat ID was validated against startup catalog")',
               'pub struct ReportDescription(ThreatId)', 'pub struct EvidenceKind(ThreatId)'):
    if needle not in body:
        raise ValueError('expected threat admission/lookup source missing')
root_source = sources[(current, 'crates/adventuresim-tactical-client/src/camera.rs')]
for needle in ('pub(crate) fn animation_capture_camera_offset(',
               'rotation.quaternion() * Vec3::Z * CameraRigConfig::default().lowered.distance.metres()'):
    if needle not in root_source:
        raise ValueError('camera native calculation source missing')
source = next(item for item in camera['current_sources'] if item['path'] == camera['path'])
if camera['personally_inspected_current_ranges'] != [[1, source['lines']]]:
    raise ValueError('camera body read range mismatch')
baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
if sum(x['id'] == threat['contract_id'] and x['record'] == 'ASSESSMENT-L1-THREAT-IDENTITY.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('threat ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
entry = next(x for x in baseline['source_delta'] if x['path'] == camera['path'])
if not entry['lens_status'][0].startswith('assessed:') or 'ASSESSMENT-L01-CAMERA-ROOT.json' not in entry['lens_status'][0]:
    raise ValueError('camera delta disposition mismatch')
if len(threat['historical_source_roles']) != 5:
    raise ValueError('historical role count mismatch')
print('verified threat source, static catalog inventory and scoped camera body; runtime remains unobserved')
