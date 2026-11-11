#!/usr/bin/env python3
"""Verify road/weather evidence and symbolic comparisons; no product execution."""
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
name = 'ASSESSMENT-ROAD-WEATHER-IDENTITY-95E.json'
a = load(name)
h = load('ASSESSMENT-L01-HISTORICAL-1672-1676.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('ROAD-WEATHER-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1672: 325, 1673: 212, 1674: 168, 1675: 402, 1676: 257}
ranges = {1672: [[1, 165], [166, 325]], 1673: [[1, 212]], 1674: [[1, 168]], 1675: [[1, 210], [211, 402]], 1676: [[1, 145], [146, 257]]}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i == 1674 else 'unassessed')
        if i == 1674:
            assert item['prior_current_observations'] == [{'assessment': 'ASSESSMENT-L1-TRIANGLE-COUNTS.json', 'ranges': [[65, 88]]}]
        assert item['new_personal_read_ranges'] == ranges[i] and len(old.splitlines()) == lengths[i]
        assert sha(old) == row['historical_sha256'] and row['status'] == 'full-historical-body-trace-open'
        assert row['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1672-1676.json'
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == c['lines']
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        if i == 1676:
            assert c['byte_identical'] is False and c['current_personal_ranges'] == [[1, 145], [146, 258]]
            assert body.replace(b'            foliage: None,\n', b'', 1) == old
        else:
            assert c['byte_identical'] and body == old and c['current_personal_ranges'] == ranges[i]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
    future, = a['earlier_historical_source_identities']
    assert future['original_index'] == 1677 and future['new_personal_historical_full_read'] is False and future['additional_original_frontier_credit'] == 0
    body = archive.extractfile(future['sha256']).read()
    path = future['source_id'].split(':', 1)[1]
    assert sha(body) == w['roles'][1677]['historical_sha256']
    chain(path, proof['paths'][path], body, proof['root_tree'], nodes)
    oldlock = archive.extractfile(w['roles'][497]['historical_sha256']).read()
    assert w['roles'][497]['status'] == 'lock-metadata-partial'
    assert a['historical_lock_evidence']['inspected_ranges'] == [[1802, 1805], [2800, 2803]]
    lockproof = load('CITY-GROUND-WIRE-HISTORICAL-GIT-PROOFS.json')
    locknodes = dict(shared['tree_objects'])
    for path in lockproof['additional_shared_tree_objects']:
        locknodes.update(load(path)['additional_tree_objects'])
    locknodes.update(lockproof['additional_tree_objects'])
    chain('Cargo.lock', lockproof['paths']['Cargo.lock'], oldlock, lockproof['root_tree'], locknodes)

sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
checksums = {'bevy_ecs-0.19.1': ('a350b2e4c2e47f7903d688affb8a723b31bca63af9789f1da88703f1ba04e354', (1802, 1805), (1785, 1788)), 'bevy_replicon-0.41.1': ('a25105650eb1b27c6ada888e06bb38ff7e326a94c7c5b4d75fb5aee50c5ae45e', (2800, 2803), (2632, 2635))}
native = {}
for package, (checksum, oldrange, newrange) in checksums.items():
    manifest = load(f'native-evidence/{package}/MANIFEST.json')
    raw = (root / f'native-evidence/{package}' / manifest['archive']).read_bytes()
    assert manifest['sha256'] == sha(raw) == checksum and len(raw) == manifest['bytes']
    assert checksum.encode() in b'\n'.join(oldlock.splitlines()[oldrange[0]-1:oldrange[1]])
    assert checksum.encode() in b'\n'.join(sources['Cargo.lock'].splitlines()[newrange[0]-1:newrange[1]])
    with tarfile.open(root / f'native-evidence/{package}' / manifest['archive'], 'r:gz') as archive:
        for item in manifest['inspected_members']:
            body = archive.extractfile(item['path']).read()
            assert sha(body) == item['sha256'] and len(body) == item['bytes']
            native[item['path']] = body

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
for n, indices in [(531, [1673, 1674]), (532, [1675, 1676])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for row in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
path = 'adler/READ-L11-241.json'
receipt = json.loads(records[path])
check = json.loads(records['adler/VERIFY-L11-241.json'])
assert [row['original_index'] for row in receipt['sources']] == list(range(1672, 1678))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-241.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg = 'adler/' + row['registration'].replace('\\', '/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256'] == lookup[reg]['sha256']
for row in receipt.get('same_lens_context', []):
    rp = 'adler/' + row['receipt'].replace('\\', '/').split('/adler/')[-1]
    assert rp in records and row['sha256'] == lookup[rp]['sha256']
prev = load('ASSESSMENT-CITY-TRAFFIC-CONTEXT-95E.json')
assert prev['earlier_historical_source_identities'][0]['original_index'] == 1672 and prev['earlier_historical_source_identities'][0]['additional_original_frontier_credit'] == 0

surfaces = sources['crates/adventuresim-tactical-core/src/city_layout/surfaces.rs']
graph = sources['crates/adventuresim-tactical-core/src/city_layout/graph.rs']
descriptor = sources['crates/adventuresim-tactical-core/src/scene_input/descriptor.rs']
scene = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
environment = sources['crates/adventuresim-tactical-core/src/scene_input/environment.rs']
selector = sources['crates/adventuresim-tactical-client/src/presentation/environment.rs']
netcode = sources['crates/adventuresim-tactical-netcode/src/replication.rs']
assert b'.and_modify(|road| road.half_width = road.half_width.max(half_width_metres))' in current[1672]
assert b'.or_insert_with(|| Road::new(start_metres, end_metres, half_width_metres))' in current[1672]
assert b'(point * NODE_SNAP_PER_METRE).round().as_ivec2()' in current[1672]
assert b'start_metres.distance_squared(end_metres) > 0.0' in surfaces and b'(1.0..=20.0).contains(&half_width_metres)' in surfaces
assert b'(point.x.to_bits(), point.y.to_bits())' in graph and b'pub streets: Vec<CityStreetPatch>' in descriptor and b'serde_json::to_vec(self)?' in scene
assert b'if !active.is_changed()' in current[1675] and b'#[component(immutable)]' in environment
assert b'On<Add, SceneEnvironment>' in selector and b'!environment.is_changed()' in selector
assert b'.replicate::<SceneEnvironment>()' in netcode
assert b'entity.insert(component);' in native['bevy_replicon-0.41.1/src/shared/replication/registry/receive_fns.rs']
assert b'fn default_write_fn()' in native['bevy_replicon-0.41.1/src/shared/replication/registry/receive_fns.rs']
assert b'Triggered when a component is added to an entity that did not already have it.' in native['bevy_ecs-0.19.1/src/lifecycle.rs']
assert b'Immutable components are guaranteed to never have an exclusive reference' in native['bevy_ecs-0.19.1/src/component/mod.rs']
assert b'let signature = weather_occlusion_tree_signature(scene, &trees);' in current[1676]
assert b'transform.transform_point(branch.start)' in current[1676] and b'.to_scale_rotation_translation()' in current[1676]
assert b'presentation.weather_occlusion_cache_key().to_le_bytes()' in current[1676]
assert b'SceneObstacle::Tree => 0.0' in sources['crates/adventuresim-tactical-server/src/scene_obstacles.rs']
assert b'.insert(environment.clone())' in sources['crates/adventuresim-tactical-client/src/presentation/atmosphere.rs']
# Dyadic input coordinates are exact in binary; inequalities stay away from round ties.
small = Fraction(1, 4096)
assert 0 < 1000 * small < Fraction(1, 2)
assert 1000 < 1000 * (1 + small) < Fraction(2001, 2)
assert 1 + small - small == 1 and 1 <= 2 <= 20
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [89, 90, 1394, 1395, 1396, 1397, 1398])
road, weather, rotation = a['conditional_findings']
assert (road['id'], weather['id'], rotation['id']) == ('ADLER-L11-TRAFFIC-NODE-REPRESENTATIVE-IDENTITY', 'ADLER-L11-WEATHER-ENVIRONMENT-REVISION', 'REVIEW-WEATHER-TREE-TRANSFORM-IDENTITY-001')
originalq = load('baseline-059/queue.json')['entries']
assert originalq[90]['contract_id'] == road['id'] and originalq[89]['contract_id'] == weather['id']
assert originalq[1398]['contract_id'] == 'Q-L12-c65931ec59acdb224f70'
assert all(item['accepted_historical_unique_count_change'] == 0 and item['repair'] == 'not started' and item['validation'] == 'not performed' for item in [road, weather, rotation])
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1 for finding in [road, weather, rotation])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified road/weather source, earlier attributions, framework archives and symbolic cases; no product execution.')
