"""Verify hair-field source and historical receipt evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-HAIR-FIELD-95E.json'
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
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob']
    assert current[item['path']] == previous[item['path']] == oid
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 8
p = 'crates/adventuresim-character-creator/src/'
must(sources[p + 'lib.rs'], ['pub mod hair;'])
must(sources[p + 'hair.rs'], ['pub use groom::{BodySurface, GroomedHair, PlacedHair};'])
groom = sources[p + 'hair/groom.rs']
must(groom, ['pub struct BodySurface<\'a>', 'pub positions: &\'a [[f32; 3]]',
    'pub faces: &\'a [[u32; 3]]', 'pub fn place(',
    'BodyField::sample(gpu, body, self.carried_bounds(placement))?',
    'draping.place(gpu, &self.on_gpu, &field)?'])
field = sources[p + 'hair/gpu/field.rs']
must(field, ['const TRIANGLE_BITS: u32 = 20;', 'NearMesh::upload(context, body, [lo, hi])?',
    'let normals = vertex_normals(&positions, body.faces);', '.filter(|face| {',
    'triangles: faces.len() as u32,', '"count" => grid.mesh.triangles',
    '"triangle_bits" => TRIANGLE_BITS', '(near << params.triangle_bits) | t',
    '(mark & ((1u << params.triangle_bits) - 1u)) + 1u',
    'let t = nearest[k];', 'closest(p, t - 1u)'])
assert field.index('let normals = vertex_normals(&positions, body.faces);') < field.index('.filter(|face| {')
must(sources[p + 'hair_preview.rs'], ['faces: character.mesh.faces.clone()',
    'let body = BodySurface {', 'hair.place(gpu, &placement, &body)'])
must(sources[p + 'hair/gpu/place/mod.rs'], ['let carried = self.carry(gpu, groomed, field)?',
    'carry.insert("head_shape", field.buffer.clone());',
    'gpu.groomer().apply(&placed, &[shrinkwrap], &field.buffer)?'])
must(sources['crates/fabelgeist-hair/src/surface.rs'], ['pub fn vertex_normals(',
    'let [a, b, c] = t.map(|i| positions[i as usize]);'])
must(sources['crates/fabelgeist-compute/src/kernel/mod.rs'], ['pub fn dispatch_folded(',
    'self.dispatch(kernel, parameters, kernel.groups_folded(items))'])
# Evidence-only integer reasoning for one nonzero winning mark.
bits = 20
ordinal = 1 << bits
proximity = 1
mark = (proximity << bits) | ordinal
assert mark != 0 and ((mark & ((1 << bits) - 1)) + 1) == 1
assert ((1 << bits) - 1) < (1 << bits) and ordinal >= (1 << bits)
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entries = {item['path']: item for item in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) == item['bytes']
    receipts[item['original_path'].rsplit('/', 1)[-1]] = json.loads(raw)
adoption = receipts['audit11-segovb-173-adoption-root.json']
earlier = receipts['audit11-segovb-173-800-complete.json']
assert adoption['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert adoption['new_unique_delta'] == earlier['accepted_unique_count_change'] == 0
assert earlier['original'] == 800 and earlier['lens'] == 'type-granularity'
assert earlier['sha256'] == doc['historical_source_roles'][0]['historical_sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
assert ledger['audit_progress'][68]['receipt'] == doc['historical_records'][0]['id']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
role = work['roles'][800]; historical = doc['historical_source_roles'][0]
assert role['source_id'] == historical['source_id'] and role['historical_sha256'] == historical['historical_sha256']
assert historical['byte_identical'] == (historical['historical_sha256'] == historical['current_sha256'])
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial')
assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified eight Git-bound sources, two original receipts and packed-ordinal reasoning; no product ran')
