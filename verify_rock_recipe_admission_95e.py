#!/usr/bin/env python3
"""Verify historical and current source evidence; no product execution."""
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
a = load('ASSESSMENT-ROCK-RECIPE-ADMISSION-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1589-1591.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('ROCK-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    head, path = line.split('\t', 1)
    mode, kind, hash_ = head.split()
    tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']
raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1589,1590,1591]
assert [x['lines'] for x in h['historical_source_roles']] == [407,141,319]
expected_history = {1589:[[1,225],[226,407]],1590:[[1,141]],1591:[[1,180],[181,319]]}
expected_current = {1589:[[1,115],[220,320],[335,411]],1590:[[1,140]],1591:[[1,180],[181,319]]}
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']
        role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1589-1591.json'
        assert x['new_personal_read_ranges'] == expected_history[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']
        body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i == 1591) and c['current_personal_ranges'] == expected_current[i]
        check_chain(c['path'], proof['paths'][c['path']], original)
        if i != 1591:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']
            assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
        current[i] = body
    assert [x['original_index'] for x in a['earlier_historical_source_identities']] == [1914,1920,1954]
    for x in a['earlier_historical_source_identities']:
        role = w['roles'][x['original_index']]
        body = archive.extractfile(x['sha256']).read()
        path = x['source_id'].split(':', 1)[1]
        assert sha(body) == x['sha256'] == role['historical_sha256'] and len(body) == x['bytes'] and len(body.splitlines()) == x['lines']
        assert oid('blob', body) == x['git_blob'] == tree[path]
        assert x['new_personal_full_read'] is False and x['additional_original_frontier_credit'] == 0
        check_chain(path, proof['paths'][path], body)

sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
core = 'crates/adventuresim-tactical-core/src/'
client = 'crates/adventuresim-tactical-client/src/'
scene = sources[core + 'scene_input.rs']
recipe = sources[core + 'scene_input/rock_recipe.rs']
detail = sources[core + 'scene_input/detail_obstacles.rs']
extract = sources[client + 'presentation/volumetric.rs']
assert b'pub struct RockRecipe' in scene and b'pub collision_radius_cm: u16' in scene
assert b'pub enum SceneObstacle' in scene and b'Rock(RockRecipe)' in scene
assert b'#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]' in scene
assert b'pub const ROCK_RADIUS_METRES: f32 = 0.75' in scene and b'collision_radius_cm: (ROCK_RADIUS_METRES * 100.0) as u16' in recipe
assert b'PositiveLength::from_metres(recipe.collision_radius_metres())?' in detail
assert b'.replicate::<SceneObstacle>()' in sources['crates/adventuresim-tactical-netcode/src/replication.rs']
assert b'Collider::sphere(recipe.collision_radius_metres())' in sources['crates/adventuresim-tactical-server/src/scene_obstacles.rs']
assert b'SceneObstacle::Rock(recipe)' in current[1590] and b'procedural_rock_mesh(recipe)' in current[1590]
assert b'minimum: Vec3::splat(-radius)' in current[1591] and b'maximum: Vec3::splat(radius)' in current[1591]
assert b'!grid.minimum.cmplt(grid.maximum).all()' in extract and b'return None' in extract
assert b'.expect("bounded rock recipe produces a finite client field")' in current[1591] and not (-0.0 < 0.0)
assert b'.add_plugins(materials::TacticalMaterialsPlugin)' in current[1589]
assert b'.add_plugins(ground_scatter::plants::PlantPresentationPlugin)' in sources[client + 'presentation/materials.rs']
assert b'branch.foliage.map(|identity|' in current[1590]

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
for n, indices in [(494,[1588,1589]),(495,[1590,1591]),(638,[1914]),(641,[1920,1921,1922,1923]),(651,list(range(1952,1958)))]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
path='adler/READ-L11-223.json';receipt=json.loads(records[path]);check=json.loads(records['adler/VERIFY-L11-223.json'])
assert [x['original_index'] for x in receipt['sources']] == list(range(1583,1592))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-223.json']['sha256']
for row in receipt['sources']:
    reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
    assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1328,1329,1602))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
p = load('CONTINUATION-PROGRESS-95E.json')
assert len(a['conditional_findings']) == 1
item = a['conditional_findings'][0]
assert item['repair'] == 'not started' and item['validation'] == 'not performed' and item['accepted_historical_unique_count_change'] == 0
assert sum(x['id'] == item['id'] and x['record'] == 'ASSESSMENT-ROCK-RECIPE-ADMISSION-95E.json' for x in ledger) == 1
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
check = load(baseline['latest_main_recheck'])
assert check['observed_revision'] == revision and check['ref'] == 'refs/heads/main'
print('Verified three historical bodies, earlier source identities, producer safeguards and conditional extraction case; no product execution.')
