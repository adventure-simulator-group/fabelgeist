"""Verify bounded world-runtime source evidence; do not execute the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
assessment = json.loads((root / 'ASSESSMENT-L01-WORLD-RUNTIME-95E.json').read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if assessment['source_revision'] != revision or len(assessment['current_sources']) != 8:
    raise ValueError('unexpected current source scope')
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, _, oid = header.split()
    if path in tree:
        raise ValueError('duplicate tree entry')
    tree[path] = oid

sources = {}
for source in assessment['current_sources']:
    path = source['path']
    saved = (root / source['preserved_bytes']).resolve()
    if not saved.is_relative_to(root) or source['revision'] != revision or source['content_encoding'] != 'gzip':
        raise ValueError('invalid source reference')
    raw = gzip.decompress(saved.read_bytes())
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != source['bytes'] or len(raw.splitlines()) != source['lines'] or
            hashlib.sha256(raw).hexdigest() != source['sha256'] or
            blob != source['git_blob'] or blob != tree.get(path)):
        raise ValueError('source identity mismatch: ' + path)
    if path in sources:
        raise ValueError('duplicate source')
    for first, last in source['ranges']:
        if not 1 <= first <= last <= source['lines']:
            raise ValueError('invalid source range')
    sources[path] = raw.decode()
markers = {
    'scripts/world_runtime_release.py': ['SCHEMA = 2', 'MEMBER_DESTINATIONS = {',
        'def validate_lock(', 'def install(', 'regional-roads-v1.json'],
    'scripts/init_world_runtime.py': ['previous = runtime.read_lock(marker)',
        'replace or replace_downloaded_release'],
    'scripts/just_tasks.py': ['def verified_world_identity(', 'world-runtime-release.lock.json'],
    'justfile': ['init-world-runtime:', 'replace-world-runtime:'],
    'crates/adventuresim-terrain/src/road_pack.rs': ['pub struct RoadManifest',
        'pub fn load(', 'SourcePackageDigest::from_hex(terrain.digest())?',
        'self.source.as_str() != terrain.digest()'],
    'crates/adventuresim-terrain/src/lib.rs': ['pub package_sha256: String',
        'pub fn digest(&self) -> &str', 'valid_digest(&manifest.package_sha256)'],
    'crates/adventuresim-terrain/src/digest.rs': ['SourcePackageDigest::from_hex(value).is_ok()'],
    'crates/adventuresim-world-schema/src/source_package.rs': ['pub struct SourcePackageDigest(String)',
        'Geometry content and placement'],
}
for path, needles in markers.items():
    if any(needle not in sources[path] for needle in needles):
        raise ValueError('source marker missing: ' + path)
lock = json.loads((root / 'main-advance-95e/current-world-runtime-release.lock.json').read_text())
expected_destinations = {
    'world-1544.json': 'target/world-1544.json',
    'strategic-map/regional-roads-v1.json': 'target/strategic-map/regional-roads-v1.json',
    'strategic-map/regional-roads-v1.pack': 'target/strategic-map/regional-roads-v1.pack',
    'strategic-map/terrain-routing-v3.json': 'target/strategic-map/terrain-routing-v3.json',
    'strategic-map/terrain-routing-v3.pack': 'target/strategic-map/terrain-routing-v3.pack',
    'strategic-map/STRATEGIC_MAP_DATA_LICENSE.md': 'target/strategic-map/STRATEGIC_MAP_DATA_LICENSE.md',
    'WORLD_RUNTIME_DATA_NOTICE.md': 'target/WORLD_RUNTIME_DATA_NOTICE.md',
}
actual_destinations = {item['path']: item['destination'] for item in lock['files']}
if lock['schema'] != 2 or actual_destinations != expected_destinations:
    raise ValueError('lock identity comparison mismatch')
for path, destination in expected_destinations.items():
    if f'"{path}": "{destination}"' not in sources['scripts/world_runtime_release.py']:
        raise ValueError('source member mapping mismatch')
candidate = assessment['ledger_candidate']
related = candidate['historical_related_candidate']
ledger = json.loads((root / related['ledger']).read_text())
original = ledger['discovery_findings_and_dependencies'][related['id']]
if original['id'] != related['id'] or related['checkpoint'] != '059':
    raise ValueError('historical relation mismatch')
entries = baseline['newly_assessed_ledger']['entries']
matches = [entry for entry in entries if entry['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-WORLD-RUNTIME-95E.json' or
        progress['new_current_ledger_candidates'] != len(entries) or candidate['validation'] != 'not performed'):
    raise ValueError('candidate ledger mismatch')
print('verified eight revision-bound source snapshots, lock comparison and qualified historical relation')
