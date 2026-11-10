"""Verify belfry source provenance and the conditional zero-edge trace; no product execution."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-BELFRY-SHELL-95E.json'
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
assert len(sources) == 15
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
must(sources[p+'lib.rs'], ['pub mod spatial_geometry;', 'SmallChurchPlan', 'compile_building_lod'])
must(sources[p+'generator/small_church.rs'], ['#[derive(Clone, Debug, Serialize, Deserialize)]', 'pub struct SmallChurchPlan', 'pub belfry_stage: SpatialBounds<Architectural>'])
must(sources[p+'model/plan.rs'], ['#[derive(Clone, Debug, Serialize, Deserialize)]', 'pub small_church: Option<crate::SmallChurchPlan>'])
must(sources[p+'spatial_geometry/bounds.rs'], ['Zero extents are permitted.', 'if lower > upper', 'Self::new(value.min, value.max)'])
must(sources[p+'generator/small_church/fittings.rs'], ['d.bell_centre().x - 1.05', 'd.bell_centre().y - 1.05', 'd.bell_centre().x + 1.05', 'd.bell_centre().y + 1.05'])
validation = sources[p+'generator/small_church/validation.rs']
must(validation, ['pub(crate) fn audit_small_church', 'church.public_route', 'small_church_wrong_authority'])
assert 'belfry_stage' not in validation
must(sources[p+'lod/compilation.rs'], ['from the accepted semantic plan', 'if plan.small_church.is_some()', 'return small_church::compile(plan, level, excluded);'])
must(sources[p+'lod/small_church.rs'], ['if lod.level == BuildingLodLevel::Shell', 'let tangent = (end - start).normalize();', 'let outward = Vec3::new(tangent.y, 0.0, -tangent.x);', 'BuildingLodMaterial::InteriorTimber).push_quad', '.dot(tangent)', 'mesh.remap_vertices();'])
must(sources[p+'lod.rs'], ['pub(crate) fn push_quad', 'geometric_normal.dot(normal) < 0.0', 'position,\n                    normal,\n                    uv,'])
must(sources[p+'lod/vertex_remap.rs'], ['seams[index] |= basis.is_none();', 'if determinant == 0.0 || !determinant.is_finite()', 'vertices.push(vertex);'])
must(sources['crates/adventuresim-tactical-client/src/presentation/buildings.rs'], ['None => compile_building_lod(plan, BuildingLodLevel::Shell)?', 'let mut mesh = recipe_mesh(batch, origin);'])
must(sources['crates/adventuresim-tactical-client/src/presentation/recipe_mesh.rs'], ['vertex.normal.to_array()', 'vertex.uv.to_array()', 'BuildingLodMaterial::InteriorTimber', 'mesh.generate_tangents()'])
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
        assert all(1 <= a <= b <= len(raw.splitlines()) for a,b in item['personally_inspected_ranges'])
        contents[item['path']] = raw.decode()
must(contents['glam-0.32.1/src/f32/vec2.rs'], ['pub fn length_recip(self) -> f32', 'self.length().recip()', 'pub fn normalize(self) -> Self', 'let normalized = self.mul(self.length_recip());', 'glam_assert!(normalized.is_finite());'])
must(contents['glam-0.32.1/src/macros.rs'], ['all(debug_assertions, feature = "debug-glam-assert")', 'feature = "glam-assert"', 'assert!($($arg)*);'])
# Exact source-shaped zero edge and fixed producer footprint, without product f32.
minimum = (Fraction(0), Fraction(0)); maximum = (Fraction(0), Fraction(2))
assert (maximum[0]-minimum[0], minimum[1]-minimum[1]) == (0, 0)
assert Fraction(105,100) - (-Fraction(105,100)) == Fraction(21,10)
h = doc['historical_source']
ledger = json.loads((root / h['ledger']).read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][122]['contract']
assert old['contract_id'] == h['contract_id'] and old['status'].startswith('Conditional admission contract')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
reg = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in reg['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for item in h['historical_source_roles']:
        r = roles[item['index']]
        path = item['source_id'].split(':',1)[1]
        assert r['source_id'] == item['source_id'] and r['historical_sha256'] == item['sha256'] == hashes[item['index']]
        assert r['historical_assessment'] == item['assessment'] and hashlib.sha256(tar.extractfile(item['sha256']).read()).hexdigest() == item['sha256']
        assert hashlib.sha256(sources[path].encode()).hexdigest() == item['sha256']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified 15 Git-bound sources, locked native source and conditional 059 linkage; no product ran')
