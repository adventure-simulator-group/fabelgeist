"""Verify surface-cut source and recovery evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-SURFACE-CUT-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision


def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in result and kind in ('blob', 'commit')
        result[filename] = oid
    return result


def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw


def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)


current = tree('main-advance-95e/current-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == current[item['path']]
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 10
p = 'crates/adventuresim-character-creator/src/'
must(sources[p + 'lib.rs'], ['pub mod surface_cut;', 'pub mod device_underlayer;'])
cut = sources[p + 'surface_cut.rs']
must(cut, ['pub struct SourcePoint', 'pub triangle: usize,', 'pub weights: [f32; 3],',
    'pub struct SurfaceCut', 'pub points: Vec<SourcePoint>,',
    'pub fn source_table(&self, faces: &[[u32; 3]], uv_faces: &[[u32; 3]])',
    'table.extend(faces[point.triangle]);', 'table.extend(uv_faces[point.triangle]);',
    'table.extend(point.weights.map(f32::to_bits));', 'pub fn new(',
    'retained_polygons(face.map(|v| positions[v as usize]), include, subtract)',
    'assembly::assemble(positions, faces, uv_faces, fragments).surface',
    'let mut remaining = vec![vec![[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]];',
    '(polygon, outside)'])
assembly = sources[p + 'surface_cut/assembly.rs']
must(assembly, ['for fragment in triangulation::conforming(polygons, &shared.points(face))',
    'if area_squared(p) <= MINIMUM_TRIANGLE_AREA_SQUARED',
    '(weight > 0).then_some((face[i], uv_faces[triangle][i], weight))',
    '.push(SourcePoint { triangle, weights });'])
triangulation = sources[p + 'surface_cut/triangulation.rs']
must(triangulation, ['boundary.push(a);', 'triangulate(boundary, polygon_index, &mut result);',
    'if let [a, b, c] = boundary.as_slice()', 'corners: [*a, *b, *c]'])
must(sources[p + 'surface_cut/edge_splits.rs'], ['let &[mut low, mut high] = axes.as_slice() else {',
    'continue;'])
plan = sources[p + 'device_underlayer/plan.rs']
must(plan, ['pub fn new(', 'design.validate()?;',
    'SurfaceCut::new(body.positions, body.faces, uv_faces, &include, &subtract)',
    'Ok(Self::from_cut(&cut, body, uv_faces))', 'pub fn from_cut(cut: &SurfaceCut, body: &Wearer<\'_>, uv_faces: &[[u32; 3]])',
    'body.faces[s.triangle].map(|v| body.positions[v as usize])',
    'let table = cut.source_table(body.faces, uv_faces);'])
must(sources[p + 'device_underlayer.rs'], ['pub mod plan;', 'pub async fn fit_async(',
    'let device_body = body.upload(gpu)?;', 'CutPlan::new(design, placement, body, domain.uv_faces'])
must(sources[p + 'armor_frames.rs'], ['pub struct Wearer<\'a>',
    'pub faces: &\'a [[u32; 3]],', 'pub positions: &\'a [[f32; 3]],'])
must(sources[p + 'underlayer_armor.rs'], ['pub struct UnderlayerBody<\'a>',
    'let fitted = device_underlayer::fit_async('])
must(sources[p + 'underlayer_equipment.rs'], ['faces: &character.mesh.faces,',
    'uv_faces: &character.mesh.texcoord_faces,'])
# Evidence-only symbolic bounds and barycentric correspondence, not product code.
assert len([]) == 0 and 0 >= len([])
original, rebound, weights = [0, 1, 2], [1, 2, 0], [1, 0, 0]
assert sum(i * w for i, w in zip(original, weights)) == 0
assert sum(i * w for i, w in zip(rebound, weights)) == 1
assert 0.25 > 1e-20
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entries = {item['path']: item for item in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) == item['bytes']
    receipts[item['original_path'].rsplit('/', 1)[-1]] = (item, json.loads(raw))
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']['lens13']
old = ledger['delegated_connected_contracts_pending_alias']
assert old[173]['receipt_sha256'] == receipts['READ-L13-PARALLEL-092.json'][0]['sha256']
assert old[173]['contract']['contract'] == doc['historical_records'][0]['id']
assert old[222]['receipt_sha256'] == receipts['READ-L13-PARALLEL-123.json'][0]['sha256']
assert 835 in old[222]['contract']['source_originals']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
reference = doc['historical_source_roles'][0]
role = work['roles'][reference['original_index']]
assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256']) is False
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
entries = base['newly_assessed_ledger']['entries']
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
    assert candidate['discovery'].startswith('partial')
    assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
print('verified ten Git-bound sources, two original receipts and symbolic correspondence; no product ran')
