"""Verify spiral source provenance and analytical count bounds; no product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-SPIRAL-ADMISSION-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        assert name not in result
        result[name] = oid
    return result

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
    assert all(1 <= a <= b <= item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 16
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
flight = sources[p+'spiral_stairs/flight.rs']
must(flight, ['MAXIMUM_RISER_METRES: f32 = 0.19', 'pub fn compile_flight(stair: Stair, storey_height: f32)', 'let required = required_treads(base_height_metres, rise_metres, storey_height);', 'let extras = tread_count.checked_sub(required)?;', 'pub fn required_treads(base: f32, rise: f32, storey_height: f32) -> u16', 'u16::from(base > 0.0)', '.ceil() as u16)', '.sum::<u16>()', '(1..=(top / storey_height).floor() as u16)', 'for tread in 1..=count'])
must(sources[p+'model/layout.rs'], ['#[serde(rename_all = "snake_case", tag = "kind")]', 'pub enum Stair', 'tread_count: u16'])
must(sources[p+'model/editor.rs'], ['pub struct EditableBuildingAssembly', 'pub storey_height_metres: f32', 'pub stairs: Vec<Stair>'])
must(sources[p+'model/player_build.rs'], ['#[derive(Clone, Debug, Serialize, Deserialize)]', 'pub struct PlayerBuildDocument', 'pub assembly: EditableBuildingAssembly', 'if self.schema_version != PLAYER_BUILD_DOCUMENT_SCHEMA_VERSION'])
must(sources[p+'viewer/run.rs'], ['serde_json::from_slice::<PlayerBuildDocument>(&bytes)', 'if let Some(document) = &player_build_document', 'setup_player_build_scene(world, document);'])
must(sources[p+'viewer/player_build_scene.rs'], ['for stair in document.assembly.stairs.iter().copied()', 'spawn_stair(', 'setup_player_build_scene(world, &document);'])
must(sources[p+'viewer/walk_rendering.rs'], ['spiral_stairs::compile_flight(', '.expect("spiral descriptor compiles a flight")'])
must(sources[p+'main.rs'], ['requires = "editor"', 'player_build_document: Option<PathBuf>', 'args.player_build_document'])
must(sources[p+'generator/tower_derivation.rs'], ['crate::spiral_stairs::required_treads(', 'tread_count:'])
must(sources[p+'spiral_stairs/keep_core.rs'], ['tread_count: required_treads(', 'compile_flight(stair, program.storey_height_metres)'])
must(sources[p+'spiral_stairs.rs'], ['compile_flight(*stair, plan.storey_height_metres)', 'filter_map(|(index, stair)|'])
must(sources[p+'model/building_program.rs'], ['pub storey_height_metres: f32', 'rejects unsupported inputs'])

# Integer/rational source-shaped reasoning, not execution of f32 or product code.
# The represented f32 0.19 lies strictly between 0.18 and 0.20.
assert Fraction(1,1)/Fraction(1,5) == 5
assert Fraction(1,1)/Fraction(9,50) < 6
assert Fraction(100000,1)/Fraction(1,5) > 65535
assert 1+100000 < 200000 and 1+65535 == 65536
assert (1+65535) % (2**16) == 0
h = doc['historical_source']
ledger = json.loads((root / h['ledger']).read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][145]
assert old['receipt'] == h['receipt'] and old['receipt_sha256'] == h['receipt_sha256']
assert old['contract']['contract'] == 'Stair semantic physical retention/admission and native flight propagation'
assert old['contract']['status'] == 'Necessary connected discovery pending; no accepted new unique finding count'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
reg = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
hashes = {x['original']: x['sha256'] for x in reg['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for item in h['historical_source_roles']:
        role = roles[item['index']]
        path = item['source_id'].split(':',1)[1]
        assert role['source_id'] == item['source_id'] and role['historical_sha256'] == item['sha256'] == hashes[item['index']]
        assert role['historical_assessment'] == item['assessment'] and hashlib.sha256(tar.extractfile(item['sha256']).read()).hexdigest() == item['sha256']
        assert hashlib.sha256(sources[path].encode()).hexdigest() == item['sha256']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
entries = baseline['newly_assessed_ledger']['entries']
assert len(doc['candidates']) == 2 and progress['new_current_ledger_candidates'] == len(entries)
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
    assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
assert progress['queue_records_semantically_reconciled'] == 0
print('verified 16 Git-bound sources, recovered stair contract and count arithmetic; no product ran')
