#!/usr/bin/env python3
"""Verify source, original records and symbolic clearance arithmetic."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
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
name = 'ASSESSMENT-LITTER-CAPTURE-CLEARANCE-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1706-1712.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('LITTER-CLEARANCE-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1706:39,1707:497,1708:225,1709:232,1710:177,1711:64,1712:160}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1, lengths[i]]]
        assert len(old.splitlines()) == lengths[i] and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1706-1712.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert c['current_personal_ranges'] == [[1, c['lines']]] and c['byte_identical'] and body == old
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
select = current[1708]
assert b'let nearest = obstacles.iter().min_by' in select
assert b'distance(obstacle.position.metres().xz()) - obstacle.radius.metres()' in select
assert b'if obstacle_clearance < 1.25' in select
assert b'SceneObstacle::Tree => 1.8' in select and b'SceneObstacle::Rock(recipe) => recipe.collision_radius_metres()' in select
scene = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
recipe = sources['crates/adventuresim-tactical-core/src/scene_input/rock_recipe.rs']
setup = sources['crates/adventuresim-tactical-client/src/tactical_scene_viewer/setup_obstacles.rs']
litter = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/litter.rs']
pairs = sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/litter/capture.rs']
assert b'pub const ROCK_RADIUS_METRES: f32 = 0.75;' in scene and b'pub const TREE_TRUNK_RADIUS_METRES: f32 = 0.35;' in scene
assert b'collision_radius_cm: (ROCK_RADIUS_METRES * 100.0) as u16' in recipe
assert b'Collider::cylinder(TREE_TRUNK_RADIUS_METRES, TREE_TRUNK_HEIGHT_METRES)' in setup
assert b'leaf_anchors.push(placement.root)' in litter and b'twig_anchors.push(placement.root)' in litter
assert b'GroundLitterCapturePair::closest(' in litter and b'.take(maximum_pairs)' in pairs
# Pure rational geometry; no fixture, product function, scene or renderer is run.
rock_distance, rock_radius = Fraction(9,4), Fraction(3,4)
tree_distance, tree_review_radius, threshold = Fraction(5,2), Fraction(9,5), Fraction(5,4)
assert rock_distance < tree_distance
assert rock_distance - rock_radius == Fraction(3,2) >= threshold
assert tree_distance - tree_review_radius == Fraction(7,10) < threshold

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(546,[1705,1706,1707]), (547,[1708,1709]), (548,[1710,1711,1712])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
prior = json.loads(records['segovb/READ-L12-547.json'])
assert 'nearest obstacle by centre distance' in str(prior['rows'][0]['function_contract'])
assert 'minimum surfaceclearance policy' in str(prior['rows'][0]['function_contract'])
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-LITTER-CAPTURE-OBSTACLE-CLEARANCE-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified seven historical roles, earlier reviewer evidence and conditional clearance arithmetic; no product execution.')
