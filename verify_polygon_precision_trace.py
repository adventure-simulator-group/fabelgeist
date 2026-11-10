"""Verify polygon source provenance and integer/IEEE reasoning; no product execution."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-POLYGON-PRECISION-95E.json'
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
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 8
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
must(sources[p+'lib.rs'], ['pub mod plan_geometry;'])
geometry = sources[p+'plan_geometry.rs']
must(geometry, ['if !metres.is_finite()', 'Finite nondegenerate counterclockwise convex vertices', 'pub fn from_ordered_vertices(vertices: Vec<P>)', 'vertices.len() < 3 || signed_area(&vertices) == 0.0', 'return Err(PlanGeometryError::DegeneratePolygon);', 'let origin = first.plan_metres().as_dvec2();', '(a.plan_metres().as_dvec2() - origin).perp_dot(b.plan_metres().as_dvec2() - origin)', '.sum::<f64>()', 'PlanHull<P>', 'cross(hull[hull.len() - 2], hull[hull.len() - 1], point)'])
must(sources[p+'collision/footprint.rs'], ['PlanHull::from_points(points)?', 'PlanHull::Area(polygon)'])
must(sources[p+'collision/ground_contact.rs'], ['PlanHull::from_points(points)?', 'GroundContact::Segment'])
must(sources['crates/adventuresim-tactical-core/src/scene_coordinates.rs'], ['pub struct ScenePlanPoint(Vec2)', 'if point.x.is_finite() && point.y.is_finite()', 'impl PlanVertex for ScenePlanPoint'])
must(sources['crates/adventuresim-tactical-core/src/scene_coordinates/polygon.rs'], ['let polygon = PlanPolygon::from_ordered_vertices(vertices)?;', 'Self::from_rigid_projection(vertices)', 'Distinct vertices and', 'positive represented area'])
must(sources['crates/adventuresim-tactical-core/src/city_layout/grounding/surface/floor/region.rs'], ['ScenePlanPolygon::from_ordered_vertices(vertices.clone())?', '&PlanPolygon::from_ordered_vertices(vertices.clone())?'])
packages = {(x['name'], x['version']): x for x in tomllib.loads(sources['Cargo.lock'])['package']}
assert 'glam 0.32.1' in packages[('bevy_math', '0.19.1')]['dependencies']
dep = doc['native_dependencies'][0]
manifest_path = (root / dep['manifest']).resolve()
assert manifest_path.is_relative_to(root)
manifest = json.loads(manifest_path.read_text())
archive = manifest_path.parent / manifest['archive']
assert hashlib.sha256(archive.read_bytes()).hexdigest() == dep['archive_sha256'] == manifest['sha256'] == packages[('glam', '0.32.1')]['checksum']
contents = {}
with tarfile.open(archive, 'r:gz') as tar:
    for item in dep['members']:
        raw = tar.extractfile(item['path']).read()
        assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
        assert all(1 <= a <= b <= len(raw.splitlines()) for a, b in item['personally_inspected_ranges'])
        contents[item['path']] = raw.decode()
must(contents['glam-0.32.1/src/f32/vec2.rs'], ['pub fn as_dvec2(self)', 'crate::DVec2::new(self.x as f64, self.y as f64)'])
must(contents['glam-0.32.1/src/f64/dvec2.rs'], ['pub fn perp_dot(self, rhs: Self) -> f64', '(self.x * rhs.y) - (self.y * rhs.x)', 'impl Sub for DVec2', 'x: self.x.sub(rhs.x)', 'y: self.y.sub(rhs.y)'])

# Exact integer reasoning over f32-representable inputs and binary64 spacing.
# This models the specified rounding boundary; it does not call product floats.
M = 2**60
A, B, C = (M, M), (0, 0), (1, 0)
real_cross = (B[0]-A[0])*(C[1]-A[1]) - (B[1]-A[1])*(C[0]-A[0])
assert real_cross == M > 0 and len({A, B, C}) == 3
lower_binary64_neighbor = M - 2**7
assert M - 1 > (M + lower_binary64_neighbor)//2
# Thus 1-M rounds to -M; the two translated vectors coincide and their
# binary64 products are the same exactly representable power of two.
assert (-M)*(-M) - (-M)*(-M) == 0
h = doc['historical_source']
queue = json.loads((root / h['queue']).read_text())
old = queue['entries'][1575]
assert old['contract_id'] == h['contract_id'] and old['status'] == 'open-evidence-or-disposition'
assert 'ActualPlanPolygon canonicaladmission' in old['first_failed_contract_or_unresolved_question']['required']
role = h['historical_source_role']
r = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles'][role['index']]
reg = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in reg['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    assert r['source_id'] == role['source_id'] and r['historical_sha256'] == role['sha256'] == hashes[role['index']]
    assert r['historical_assessment'] == role['assessment'] and hashlib.sha256(tar.extractfile(role['sha256']).read()).hexdigest() == role['sha256']
assert hashlib.sha256(geometry.encode()).hexdigest() == role['sha256']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified eight Git-bound sources, locked math and analytical finite triangle; no product ran')
