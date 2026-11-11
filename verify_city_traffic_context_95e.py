#!/usr/bin/env python3
"""Verify traffic source, recovery attribution and symbolic arithmetic only."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-CITY-TRAFFIC-CONTEXT-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1667-1671.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('CITY-TRAFFIC-CONTEXT-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1667: 140, 1668: 404, 1669: 132, 1670: 127, 1671: 175}
ranges = {1667: [[1, 140]], 1668: [[1, 200], [201, 404]], 1669: [[1, 132]], 1670: [[1, 127]], 1671: [[1, 175]]}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == ranges[i]
        assert len(old.splitlines()) == lengths[i] and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1667-1671.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == lengths[i]
        assert c['current_personal_ranges'] == ranges[i] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    future, = a['earlier_historical_source_identities']
    assert future['original_index'] == 1672 and future['new_personal_historical_full_read'] is False and future['additional_original_frontier_credit'] == 0
    body = archive.extractfile(future['sha256']).read()
    path = future['source_id'].split(':', 1)[1]
    assert sha(body) == w['roles'][1672]['historical_sha256']
    chain(path, proof['paths'][path], body, proof['root_tree'], nodes)

sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
manifest = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery']
    records[item['original_path']] = body
for n, indices in [(529, [1668, 1669]), (530, [1670, 1671, 1672])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for row in receipt.get('partial_sequence', []):
        assert row['path'] in records and row['sha256'] == lookup[row['path']]['sha256']
assert b'neutralmodevalid' in records['segovb/READ-L12-530.json']
prev = load('ASSESSMENT-CITY-GROUND-WIRE-ADMISSION-95E.json')
assert {r['original_path'] for r in prev['recovered_historical_records']} >= {'adler/READ-L11-240.json', 'segovb/READ-L12-528.json'}
assert all(item['additional_original_frontier_credit'] == 0 for item in prev['earlier_historical_source_identities'])
oldfinding = load('baseline-059/ledger.json')['discovery_findings_and_dependencies']['ADLER-L11-CITY-TRAFFIC-DISTANCE-FRAME']
assert oldfinding['accepted_unique_count_change'] == 0 and oldfinding['lens12_connected_reconciliation'][0]['source_original_index'] == 1667

streets = sources['crates/adventuresim-tactical-client/src/presentation/vista/streets.rs']
material = sources['crates/adventuresim-tactical-client/src/presentation/vista/streets/material.rs']
mesh = sources['crates/adventuresim-tactical-client/src/presentation/vista/streets/mesh.rs']
shader = sources['assets/shaders/tactical_city_ground.wgsl']
scene = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'const DETAIL_RADIUS_METRES: f32 = 110.0;' in current[1667]
assert b'Vec3::new(centre.x, 0.0, centre.y).distance(position) < DETAIL_RADIUS_METRES' in current[1667]
assert b'let Some(camera) = cameras.iter().next()' in current[1667] and b'masks.retain(|tile, _| wanted.contains(tile))' in current[1667]
assert b'#[serde(skip)]' not in current[1668] and b'self.index.take();' in current[1668]
assert b'index.partition(0..triangles.len(), &bounds);' in current[1669]
assert b'&[0, 0, 0, 255]' in current[1671] and b'TextureFormat::Rgba8Unorm' in current[1671]
assert b'traffic::TrafficMask::neutral(images)' in streets and b'StreamedTrafficTile(*tile)' in streets
assert b'traffic_transform: traffic.transform' in material and b'traffic_mask: traffic.image.clone()' in material
assert b'point.y += patch.lift_metres' in mesh and b'!(-12_000.0..=12_000.0).contains(height)' in scene
assert b'let traffic = textureSample(traffic_mask, traffic_sampler, traffic_uv).rgb;' in shader
assert b'clamp(1.0 - traffic.b + (broken - 0.5) * 0.20, 0.0, 1.0)' in shader
assert b'0.075 + shoulder * 0.91' in shader and b'1.0 - smoothstep(0.02, 0.82, soil_level)' in shader
assert b'((1.0 - smooth_band(0.0, ROAD_SHOULDER_METRES, clearance)) * 255.0)' in current[1671]
assert 121**2 > 110**2 and 121 - 120 == 1 and -12000 <= 120 <= 12000
soil_neutral = Fraction(75, 1000) + Fraction(91, 100)
soil_baked = Fraction(75, 1000)
low, high = Fraction(2, 100), Fraction(82, 100)
assert soil_neutral == Fraction(985, 1000) and soil_neutral > high and low < soil_baked < high
t = (soil_baked - low) / (high - low)
assert 0 < 1 - (3 * t * t - 2 * t * t * t) < 1
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [88, 1389, 1390, 1391, 1392, 1393])
frame, neutral = a['conditional_findings']
assert frame['id'] == 'ADLER-L11-CITY-TRAFFIC-DISTANCE-FRAME'
assert neutral['id'] == 'REVIEW-CITY-TRAFFIC-NEUTRAL-SHOULDER-001'
originalq = load('baseline-059/queue.json')['entries']
assert originalq[88]['contract_id'] == frame['id'] and originalq[1393]['contract_id'] == 'Q-L12-ce3672dd2af98d3371f7'
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [frame, neutral])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [frame, neutral])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified traffic source, prior reviewer judgments and symbolic comparisons; no product execution.')
