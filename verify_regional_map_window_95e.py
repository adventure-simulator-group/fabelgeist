#!/usr/bin/env python3
"""Verify regional map source arithmetic and historical evidence, without product execution."""
import difflib
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda value: hashlib.sha256(value).hexdigest()

def oid(kind, value):
    return hashlib.sha1(kind.encode() + b' ' + str(len(value)).encode() + b'\0' + value).hexdigest()

def entries(value):
    out, pos = {}, 0
    while pos < len(value):
        end = value.index(b'\0', pos)
        mode, name = value[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), value[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(value)
    return out

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
a = load('ASSESSMENT-REGIONAL-MAP-WINDOW-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1613-1618.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('REGIONAL-MAP-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, hash_ = header.split()
    tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']; raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == list(range(1613,1619))
assert [x['lines'] for x in h['historical_source_roles']] == [31,87,143,189,284,115]
current, historic = {}, {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1613-1618.json'
        assert x['prior_role_status'] == 'unassessed' and x['new_personal_read_ranges'] == [[1,x['lines']]]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i != 1617) and c['current_personal_ranges'] == [[1,c['lines']]]
        check_chain(c['path'], proof['paths'][c['path']], original)
        if i == 1617:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']
            assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
            assert b'-            && crate::strategic_scene::status::waiting_pipelines() == 0' in diff
            assert b'+            && crate::presentation::pipeline_readiness::waiting_pipelines() == 0' in diff
        current[i], historic[i] = body, original
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1619,1620,1621,1622]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]; body = archive.extractfile(x['sha256']).read(); path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] and x['new_personal_historical_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)
sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/presentation/'
core = 'crates/adventuresim-tactical-core/src/'
web = 'crates/strategic-web/'
camera, city, installation = current[1616], current[1617], current[1618]
protocol = sources[base+'regional_map/protocol.rs']; canvas = sources[base+'canvas_view.rs']
map_ = sources[core+'regional_map.rs']; terrain = sources[core+'regional_terrain.rs']
length = sources['crates/fabelgeist-geometry/src/length.rs']; surface = sources[base+'regional_map/surface.rs']
status = sources[base+'regional_map/status.rs']; mesh = sources[base+'vista/regional.rs']
assert all(x in length for x in [b'if !value.is_finite()',b'if value < 0.0',b'if value == 0.0',b'Ok(Self(value))'])
assert b'MAX_MAP_SPAN_METRES: f32 = 8_192_000.0' in map_ and b'(MIN_MAP_SPAN_METRES..=MAX_MAP_SPAN_METRES).contains(&metres)' in map_
assert b'#[serde(try_from = "MapRouteWire"' in map_ and b'(2..=MAX_MAP_ROUTE_POINTS).contains(&points.len())' in map_
assert all(x in protocol for x in [b'rect.width > 0',b'rect.height > 0',b'rect.full_width >= rect.width',b'rect.full_height >= rect.height',b'rect.offset_x.is_finite()',b'rect.offset_y.is_finite()',b'pixels\n            .is_finite()'])
assert b'full_size: UVec2::new(self.full_width.max(1), self.full_height.max(1))' in canvas
assert all(x in camera for x in [b'WINDOW_COVERAGE_MARGIN: f64 = 2.0',b'let width = f64::from(self.span.metres()) * aspect;',b'let needed = east_extent.max(north_extent) * WINDOW_COVERAGE_MARGIN;',b'.unwrap_or(RegionalTerrainScale::Continent)'])
assert b'REGIONAL_TERRAIN_SIDE: usize = 65' in terrain and b'Self::Continent => 128_000.0' in terrain
assert (1000 / 500) * 8192000 == 16384000 and 64 * 128000 == 8192000
assert (1000 / 500) * 8192000 * 2 == 32768000 > 64 * 128000
assert b'(index % REGIONAL_TERRAIN_SIDE) as f32 * spacing - center * spacing' in mesh
assert b'if indices.is_empty()' in mesh and b'return None;' in mesh and b'Some(mesh.with_computed_area_weighted_normals())' in mesh
assert b'coverage: match mesh' in surface and b'SurfaceCoverage::Drawn' in surface
assert b'surface.source == pose.source' in status and b'matches!(surface.coverage, SurfaceCoverage::Drawn { .. })' in status
assert b'!state.covered ? "Terrain unavailable here"' in sources[web+'static/regional-map-view.js']
assert b'"zoom-out":()=>zoom(1.25)' in sources[web+'static/regional-map-gestures.js']
assert b'INITIAL_MAP_SPAN_METRES: f32 = 60_000.0' in sources[web+'src/strategic_map/producer.rs']
assert b'residency.pending.replace(*city)' not in installation and b'state.city.pending.replace(*city)' in installation
assert b'if installation.is_err()' in installation and b'entity.despawn()' in installation
assert b'CityAssemblyPublished' in city and b'city_gpu_ready(PresentationOwner::RegionalMap)' in city

manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for x in a['recovered_historical_records']:
    original = lookup[x['original_path']]
    assert (x['sha256'], x['bytes'], x['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    body = contents(x['preserved_bytes'])
    assert sha(body) == x['published_sha256'] and len(body) == x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body) == x['sha256'] and oid('blob', body) == x['git_blob'])
    records[x['original_path']] = body
for n, indices in [(508,[1613,1614,1615]),(509,[1616,1617]),(510,[1618,1619,1620])]:
    path=f'segovb/READ-L12-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert not receipt.get('qualified_supporting_same_lens_reuse',[]) and not receipt.get('partial_sequence',[])
path='adler/READ-L11-231.json'; receipt=json.loads(records[path]); check=json.loads(records['adler/VERIFY-L11-231.json'])
assert [x['original_index'] for x in receipt['sources']] == list(range(1613,1623))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-231.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\','/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1342,1343,1344,1345))
baseline = load('BASELINE-95E-20261110.json'); ledger = baseline['newly_assessed_ledger']['entries']
assert [x['id'] for x in a['conditional_findings']] == ['REVIEW-REGIONAL-MAP-WINDOW-FOOTPRINT-001']
for item in a['conditional_findings']:
    assert item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed'
    assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-REGIONAL-MAP-WINDOW-95E.json' for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == revision
print('Verified six historical bodies, earlier reader attribution and bounded map-window source arithmetic; no product execution.')
