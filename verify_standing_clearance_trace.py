"""Verify source evidence and analytical IEEE limits without executing product code."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-STANDING-CLEARANCE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        assert name not in out
        out[name] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip' and item['revision'] == revision
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 17
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
must(sources[p+'interior.rs'], ['pub use exterior_access::StandingClearance;'])
must(sources[p+'interior/exterior_access.rs'], ['Obstruction::new(*solid)?.projection(bottom, top)?', 'bounds.min().metres() - margin', 'pub fn is_clear(&self, start: Position<F>, end: Position<F>) -> bool', '.all(|&rect| !segment_intersects(rect, start, end))', 'let delta = end - start;', 'let a = (min[axis] - start[axis]) / delta[axis];', 'let b = (max[axis] - start[axis]) / delta[axis];', 'if enter > leave {'])
must(sources[p+'interior/geometry.rs'], ['PERSON_RADIUS: f32 = 0.30', 'PERSON_HEIGHT: f32 = 1.8', 'FLOOR_CLEARANCE: f32 = 0.15'])
must(sources[p+'interior/obstruction.rs'], ['let topology = solid.corners()?;', 'if self.top.metres() <= bottom.metres() || self.bottom.metres() >= top.metres()', 'for edge in self.corners.edges()', 'SpatialBounds::from_metres(min, max)'])
must(sources[p+'collision.rs'], ['pub fn from_metres(', 'centre: Position::from_metres(centre)?', 'size: CuboidDimensions::from_metres(size)?', 'yaw_radians: Radians::new(yaw_radians)?'])
must(sources[p+'collision/cuboid.rs'], ['centre.metres() + rotation.quaternion() * (size.metres() * sign * 0.5)', 'RigidRotation::from_quaternion(rotation)?'])
must(sources[p+'spatial_geometry/coordinates.rs'], ['finite(metres, GeometryRole::Position)?', 'pub fn displacement_to(self, other: Self)', 'Displacement::from_metres(other.metres - self.metres)', 'finite(metres, GeometryRole::Displacement)?'])
must(sources[p+'spatial_geometry/error.rs'], ['if !component.is_finite()'])
must(sources[p+'spatial_geometry/bounds.rs'], ['Self::new(Position::from_metres(min)?, Position::from_metres(max)?)', 'if lower > upper'])
must(sources[p+'spatial_geometry/orientation.rs'], ['if !value.is_finite()', 'if !value.is_finite() || !value.is_normalized()'])
must(sources['crates/fabelgeist-geometry/src/cuboid.rs'], ['if !component.is_finite()', 'if component.metres() < 0.0'])
must(sources['crates/fabelgeist-geometry/src/measurement.rs'], ['impl SignedLength', 'if !value.is_finite()'])
must(sources['crates/adventuresim-tactical-client/src/strategic_scene/staging.rs'], ['CONVERSATION_DISTANCE_METRES: f32 = 1.1', 'clearance.is_clear(', 'StandingClearance::new('])
must(sources['crates/adventuresim-tactical-core/src/city_layout/compiled/church.rs'], ['if !StandingClearance::new(', '.is_clear(physical(door)?, physical(street)?)'])
must(sources['crates/adventuresim-tactical-core/src/city_layout/compiled/property/clearance.rs'], ['validate_building_routes(', 'for route in &compound.access', 'if !clearance.is_clear('])

packages = {(x['name'], x['version']): x for x in tomllib.loads(sources['Cargo.lock'])['package']}
assert 'glam 0.32.1' in packages[('bevy_math', '0.19.1')]['dependencies']
dep = doc['native_dependencies'][0]
manifest_path = (root / dep['manifest']).resolve()
assert manifest_path.is_relative_to(root)
manifest = json.loads(manifest_path.read_text())
archive = manifest_path.parent / manifest['archive']
assert hashlib.sha256(archive.read_bytes()).hexdigest() == dep['archive_sha256'] == manifest['sha256'] == packages[('glam', '0.32.1')]['checksum']
with tarfile.open(archive, 'r:gz') as tar:
    entry = next(x for x in manifest['inspected_members'] if x['path'] == dep['source_member'])
    member = tar.extractfile(entry['path']).read()
    assert len(member) == entry['bytes'] and hashlib.sha256(member).hexdigest() == entry['sha256']
    must(member.decode(), ['impl Sub for Vec2', 'x: self.x.sub(rhs.x)', 'y: self.y.sub(rhs.y)'])

# Integer/rational analysis of the recorded IEEE binary32 case only: M is the
# largest finite value; +/- one metre lies below half an ulp at M, while 2M is
# above the round-to-nearest overflow threshold. This does not run product f32.
M = (2**24 - 1) * 2**104
half_ulp = 2**103
assert 1 < half_ulp and 2*M > M + half_ulp
z_min, z_max = Fraction(3, 20), Fraction(17, 20)
assert 0 < z_min < Fraction(1, 2) < z_max < 1
assert Fraction(0) < z_min  # collapsed x leave precedes z entry

h = doc['historical_source']
ledger = json.loads((root / h['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies'][h['id']]
assert old['proposed_id'] == h['proposed_id'] and old['accepted_unique_count_change'] == 0
assert old['source_only_counterexample']['no_execution'] and old['source_sha256'] == h['historical_source_role']['sha256']
role = h['historical_source_role']
r = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][role['index']]
reg = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in reg['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    assert r['source_id'] == role['source_id'] and r['historical_sha256'] == role['sha256'] == hashes[role['index']]
    assert r['historical_assessment'] == role['assessment'] and hashlib.sha256(tar.extractfile(role['sha256']).read()).hexdigest() == role['sha256']
assert hashlib.sha256(sources[p+'interior/exterior_access.rs'].encode()).hexdigest() == role['sha256']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified 17 Git-bound sources, locked native source and original 059 linkage; no product ran')
