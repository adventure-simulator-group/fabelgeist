"""Verify captured source and pinned parser archives, without executing either.

Run verify_lock_inventory.py, verify_role_worklist.py and the Rust native
verifier first. No CLI, macro expansion, product build, test or runtime runs.
"""
import gzip
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
weapon = json.loads((root / 'ASSESSMENT-L1-WEAPON-LENGTH.json').read_text())
capture = json.loads((root / 'ASSESSMENT-L1-CAPTURE-ORBIT.json').read_text())

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {weapon['current_revision']: tree('CURRENT-TREE.tsv'),
         capture['reported_selected_revision']: tree('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (weapon, capture):
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
                raise ValueError('source mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()

cur = weapon['current_revision']
model = sources[(cur, 'crates/adventuresim-core/src/autoresolve/model.rs')]
producer = sources[(cur, 'crates/adventuresim-core/src/autoresolve/authored_weapon.rs')]
viewer = sources[(cur, 'crates/adventuresim-tactical-client/src/animation_viewer_main.rs')]
inspection = sources[(cur, 'crates/adventuresim-tactical-client/src/animation_viewer/inspection.rs')]
for data, needles in [(model, ('pub melee_reach: f32', 'pub grip_to_tip_m: f32')),
                      (producer, ('melee_reach: *reach_m', 'striking_head_length_m,')),
                      (viewer, ('camera_orbit_degrees: f32', 'args.camera_orbit_degrees')),
                      (inspection, ('"orbit_degrees": orbit_degrees',
                                    'Quat::from_rotation_y(orbit_degrees.to_radians())'))]:
    if any(needle not in data for needle in needles):
        raise ValueError('expected bounded source expression missing')

lock = tomllib.loads((root / 'lock-evidence/current-Cargo.lock').read_text())
packages = {(item['name'], item['version']): item for item in lock['package']}
contents = {}
for item in capture['native_evidence']:
    manifest = json.loads((root / item['manifest']).read_text())
    archive_path = (root / item['archive']).resolve()
    if not archive_path.is_relative_to(root):
        raise ValueError('native archive escapes checkpoint')
    raw = archive_path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if (digest != item['sha256'] or digest != manifest['sha256'] or
            digest != packages[(item['package'], item['version'])]['checksum'] or
            len(raw) != manifest['bytes']):
        raise ValueError('native archive or lock mismatch')
    with tarfile.open(archive_path, 'r:gz') as tar:
        for member in manifest['inspected_members']:
            data = tar.extractfile(member['path']).read()
            if hashlib.sha256(data).hexdigest() != member['sha256'] or len(data) != member['bytes']:
                raise ValueError('native archive member mismatch')
            contents[member['path']] = data.decode()

if 'Parse::from_str' not in contents['clap_builder-4.6.6/src/builder/value_parser.rs']:
    raise ValueError('default parser fallback missing')
if 'clap::value_parser!(#inner_type)' not in contents['clap_derive-4.6.4/src/item.rs']:
    raise ValueError('derive default parser missing')
if 'Number::from_f32(f).map_or(Value::Null, Value::Number)' not in contents['serde_json-1.0.151/src/value/from.rs']:
    raise ValueError('nonfinite JSON mapping source missing')
if '$crate::to_value(&$other).unwrap()' not in contents['serde_json-1.0.151/src/macros.rs']:
    raise ValueError('JSON expression mapping source missing')
rust = gzip.decompress((root / capture['rust_float_source']).read_bytes()).decode()
if "'inf', '-inf', '+infinity', 'NaN'" not in rust:
    raise ValueError('installed float parser documentation missing')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
for doc, name in ((weapon, 'ASSESSMENT-L1-WEAPON-LENGTH.json'),
                  (capture, 'ASSESSMENT-L1-CAPTURE-ORBIT.json')):
    if sum(x['id'] == doc['contract_id'] and x['record'] == name
           for x in baseline['repair_ledger']['entries']) != 1:
        raise ValueError('ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
if len(weapon['historical_source_roles']) != 4:
    raise ValueError('historical role count mismatch')
entry = next(x for x in baseline['source_delta']
             if x['path'] == 'crates/adventuresim-tactical-client/src/animation_viewer_main.rs')
if not entry['lens_status'][0].startswith('partial'):
    raise ValueError('capture delta disposition mismatch')
print('verified weapon and capture source plus pinned parser archives; actual CLI and runtime remain open')
