"""Verify selected plant-mesh evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-PLANT-MESH-COMPOSITION-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 4 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 9
mesh = sources['crates/adventuresim-plant-generator/src/mesh.rs']
must(mesh, ['pub positions: Vec<[f32; 3]>', 'pub normals: Vec<[f32; 3]>',
            'pub colors: Vec<[f32; 4]>', 'pub indices: Vec<u32>',
            'let base = self.positions.len() as u32;',
            '.extend(source.indices.iter().map(|index| base + index));',
            '(translation + rotation * Vec3::from_array(*p) * scale).to_array()',
            '(rotation * Vec3::from_array(*n)).to_array()',
            '.with_inserted_indices(Indices::U32(self.indices))',
            'self.samples(512, 128, 32)', 'for row in 0..=rows', 'for col in 0..=columns'])
flower = sources['crates/adventuresim-plant-generator/src/flower/geometry.rs']
must(flower, ['.min(lod.samples(8, 5, 3))', '.min(lod.samples(12, 5, 2))',
              '.min(lod.samples(5, 3, 1))', '.min(lod.samples(40, 12, 6))',
              'usize::from(p.stamens).min(8)', 'mesh.surface([1, 2]',
              'mesh.surface([rows, 2]', 'lod.samples(3, 2, 1)',
              'lod.samples(6, 4, 4)', 'Quat::from_rotation_y(angle) * Quat::from_rotation_z(p.head_tilt)',
              '            1.0,'])
mod = sources['crates/adventuresim-plant-generator/src/flower/mod.rs']
must(mod, ['("heads", f32::from(self.heads), 1.0, 8.0)',
           '("leaves", f32::from(self.leaves), 1.0, 12.0)',
           '("leaflets", f32::from(self.leaflets), 1.0, 7.0)',
           '("petals", f32::from(self.petals), 3.0, 40.0)',
           'self.validate()?;', 'Ok(geometry::generate(self, seed, detail))'])
# Source-derived loose integer allowance; no geometry or product function is run.
assert 3*8 + 12*5*8 + 7*8 + 8*(40*18 + 28 + 8*3) == 6736 < 2**32
lod = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/plants/lod.rs']
must(lod, ['species\n                        .generate(PLANT_SEED, lod)',
           'meshes: generated.map(|m| meshes.add(m.into_bevy()))',
           'Mesh3d(specimen.meshes[level.index()].clone())'])
viewer = sources['crates/adventuresim-plant-generator/src/viewer/main.rs']
must(viewer, ['match editor.parameters.generate(options.seed, editor.lod)',
              'Mesh3d(meshes.add(mesh.into_bevy()))'])
recipe = sources['crates/adventuresim-plant-generator/src/viewer/recipe.rs']
must(recipe, ['Self::Flower(p) => p.generate(seed, detail)',
              'Self::Fungus(p) => p.generate(seed, detail)'])
assert len(doc['search_evidence']) == 3
for search in doc['search_evidence']:
    raw = (root / search['path']).read_bytes()
    assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
    assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
append = (root / doc['search_evidence'][0]['path']).read_text()
assert 'crates/adventuresim-plant-generator/src/flower/geometry.rs:66:' in append
assert 'crates/adventuresim-tactical-core/src/city_layout/grounding.rs:285:' in append
conversions = (root / doc['search_evidence'][1]['path']).read_text()
assert 'crates/adventuresim-plant-generator/src/viewer/main.rs:202:' in conversions
assert 'crates/adventuresim-tactical-client/src/presentation/ground_scatter/plants/lod.rs:73:' in conversions
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 5
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert old['evidence']['partial_checkpoints'] == []
assert any(x.get('proposed_id') == 'PL13-226-PLANT-CELL-MESH-COMPOSITION-NATIVE-PENDING'
           for x in old['evidence']['connected_contracts'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][310]
assert prior['receipt'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_operations_traces']
queue = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][2214]
assert queue['contract_id'] == 'PL13-226-PLANT-CELL-MESH-COMPOSITION-NATIVE-PENDING'
assert name in queue['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected plant mesh source, bound and 5 historical records; no product ran')
